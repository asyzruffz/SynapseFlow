use std::time::Duration;

use sha2::{Digest, Sha256};

use crate::{DomainError, DomainResult, ModelReference};

use crate::execution::{
    FrameEnvelope, FrameMessageType, FrameProtocolVersion, FrameSequence, FrameTarget,
    RemainingDeadline, SessionId, ShardId, StreamId, TensorDescriptor, TensorDtype,
    MAX_TENSOR_BYTES, MAX_TENSOR_RANK,
};

const MAGIC: [u8; 4] = *b"SYNF";
const PREFIX_BYTES: usize = 16;
const MAX_ENVELOPE_BYTES: usize = 2_048;
const MAX_MODEL_REFERENCE_BYTES: usize = 255;
const MAX_TRACE_ID_BYTES: usize = 128;
const MAX_EXTENSION_VALUE_BYTES: usize = 1_024;
const MAX_DEADLINE_MILLIS: u64 = 24 * 60 * 60 * 1_000;

/// Compression algorithm declared by an activation-frame packet.
///
/// Version 1 supports only canonical uncompressed bytes. A future compressed
/// variant must receive a new protocol capability and its own bounded decoder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameCompression {
    None,
}

impl FrameCompression {
    const NONE_TAG: u8 = 0;

    const fn wire_tag(self) -> u8 {
        match self {
            Self::None => Self::NONE_TAG,
        }
    }

    fn from_wire_tag(value: u8) -> DomainResult<Self> {
        if value == Self::NONE_TAG {
            Ok(Self::None)
        } else {
            Err(DomainError::FrameInvalid)
        }
    }
}

/// Safe observability identifier that cannot carry prompt or activation data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SafeTraceId(String);

impl SafeTraceId {
    pub fn new(value: String) -> DomainResult<Self> {
        let valid = (16..=MAX_TRACE_ID_BYTES).contains(&value.len())
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'));
        if !valid {
            return Err(DomainError::FrameInvalid);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Bounded additive header field preserved by the protocol codec.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrameExtension {
    tag: u8,
    value: Vec<u8>,
}

impl FrameExtension {
    pub fn new(tag: u8, value: Vec<u8>) -> DomainResult<Self> {
        if tag == 0 || value.len() > MAX_EXTENSION_VALUE_BYTES {
            return Err(DomainError::FrameBoundsExceeded);
        }
        Ok(Self { tag, value })
    }

    pub const fn tag(&self) -> u8 {
        self.tag
    }

    pub fn value(&self) -> &[u8] {
        &self.value
    }
}

/// A fully validated packet decoded from canonical activation-frame bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedFrame {
    pub envelope: FrameEnvelope,
    pub compression: FrameCompression,
    pub payload: Vec<u8>,
    payload_sha256: [u8; 32],
    pub trace_id: Option<SafeTraceId>,
    extensions: Vec<FrameExtension>,
}

impl DecodedFrame {
    pub const fn payload_sha256(&self) -> &[u8; 32] {
        &self.payload_sha256
    }

    pub fn extensions(&self) -> &[FrameExtension] {
        &self.extensions
    }
}

/// Deterministic encoder and bounded decoder for activation-frame protocol v1.
pub struct FrameCodec;

impl FrameCodec {
    /// Encodes a frame using the protocol-v1 canonical binary schema.
    pub fn encode(
        envelope: &FrameEnvelope,
        payload: &[u8],
        compression: FrameCompression,
        trace_id: Option<&SafeTraceId>,
    ) -> DomainResult<Vec<u8>> {
        Self::encode_with_extensions(envelope, payload, compression, trace_id, &[])
    }

    /// Encodes a frame with bounded additive header fields.
    pub fn encode_with_extensions(
        envelope: &FrameEnvelope,
        payload: &[u8],
        compression: FrameCompression,
        trace_id: Option<&SafeTraceId>,
        extensions: &[FrameExtension],
    ) -> DomainResult<Vec<u8>> {
        if !envelope.protocol_version.is_supported() {
            return Err(DomainError::ProtocolUnsupported);
        }
        validate_payload_shape(envelope, payload.len())?;
        if compression != FrameCompression::None {
            return Err(DomainError::FrameInvalid);
        }

        let payload_len =
            u32::try_from(payload.len()).map_err(|_| DomainError::FrameBoundsExceeded)?;
        let model = envelope.target.model.as_str().as_bytes();
        if model.len() > MAX_MODEL_REFERENCE_BYTES {
            return Err(DomainError::FrameBoundsExceeded);
        }
        let session = envelope.session_id.as_str().as_bytes();
        let shard = envelope.target.shard.as_str().as_bytes();
        let trace = trace_id.map(SafeTraceId::as_str).unwrap_or("").as_bytes();
        let extension_len = extensions_len(extensions)?;
        let header_len = header_len(
            envelope,
            model.len(),
            session.len(),
            shard.len(),
            trace.len(),
            extension_len,
        )?;
        let total_len = PREFIX_BYTES
            .checked_add(header_len)
            .and_then(|value| value.checked_add(payload.len()))
            .ok_or(DomainError::FrameBoundsExceeded)?;
        if header_len > MAX_ENVELOPE_BYTES
            || total_len > MAX_ENVELOPE_BYTES + MAX_TENSOR_BYTES as usize
        {
            return Err(DomainError::FrameBoundsExceeded);
        }

        let mut result = Vec::with_capacity(total_len);
        result.extend_from_slice(&MAGIC);
        push_u16(&mut result, envelope.protocol_version.value());
        result.push(message_type_tag(envelope.message_type));
        result.push(compression.wire_tag());
        push_u32(
            &mut result,
            u32::try_from(header_len).map_err(|_| DomainError::FrameBoundsExceeded)?,
        );
        push_u32(&mut result, payload_len);
        push_short_bytes(&mut result, session)?;
        push_u64(&mut result, envelope.stream_id.value());
        push_u64(&mut result, envelope.sequence.value());
        push_short_bytes(&mut result, model)?;
        push_short_bytes(&mut result, shard)?;
        push_u64(&mut result, deadline_millis(envelope.remaining_deadline())?);
        push_trace(&mut result, trace_id)?;
        push_tensor(&mut result, envelope.tensor.as_ref())?;
        result.extend_from_slice(&Sha256::digest(payload));
        push_extensions(&mut result, extensions)?;
        result.extend_from_slice(payload);
        Ok(result)
    }

    /// Decodes only supported protocol-v1 packets after validating all bounds.
    pub fn decode(bytes: &[u8]) -> DomainResult<DecodedFrame> {
        if bytes.len() < PREFIX_BYTES
            || bytes.len() > MAX_ENVELOPE_BYTES + MAX_TENSOR_BYTES as usize
        {
            return Err(DomainError::FrameBoundsExceeded);
        }
        if bytes[..4] != MAGIC {
            return Err(DomainError::FrameInvalid);
        }

        let version = FrameProtocolVersion::new(read_u16(bytes, 4)?)?;
        if !version.is_supported() {
            return Err(DomainError::ProtocolUnsupported);
        }
        let message_type = message_type_from_tag(*bytes.get(6).ok_or(DomainError::FrameInvalid)?)?;
        let compression =
            FrameCompression::from_wire_tag(*bytes.get(7).ok_or(DomainError::FrameInvalid)?)?;
        let header_len =
            usize::try_from(read_u32(bytes, 8)?).map_err(|_| DomainError::FrameBoundsExceeded)?;
        let payload_len =
            usize::try_from(read_u32(bytes, 12)?).map_err(|_| DomainError::FrameBoundsExceeded)?;
        if header_len > MAX_ENVELOPE_BYTES || payload_len > MAX_TENSOR_BYTES as usize {
            return Err(DomainError::FrameBoundsExceeded);
        }
        let header_end = PREFIX_BYTES
            .checked_add(header_len)
            .ok_or(DomainError::FrameBoundsExceeded)?;
        let frame_end = header_end
            .checked_add(payload_len)
            .ok_or(DomainError::FrameBoundsExceeded)?;
        if frame_end != bytes.len() || header_len < 32 {
            return Err(DomainError::FrameInvalid);
        }

        let mut reader = HeaderReader::new(&bytes[PREFIX_BYTES..header_end]);
        let session_id = SessionId::new(reader.read_short_string(128)?)?;
        let stream_id = StreamId::new(reader.read_u64()?)?;
        let sequence = FrameSequence::from_wire(reader.read_u64()?);
        let model = ModelReference::parse(reader.read_short_string(MAX_MODEL_REFERENCE_BYTES)?)?;
        let shard = ShardId::new(reader.read_short_string(128)?)?;
        let deadline_millis = reader.read_u64()?;
        if deadline_millis == 0 || deadline_millis > MAX_DEADLINE_MILLIS {
            return Err(DomainError::DeadlineExceeded);
        }
        let trace_id = reader.read_trace_id()?;
        let tensor = reader.read_tensor()?;
        let hash = reader.read_hash()?;
        let extensions = reader.read_extensions()?;

        let envelope = FrameEnvelope::new(
            version,
            message_type,
            session_id,
            stream_id,
            sequence,
            FrameTarget { model, shard },
            tensor,
            RemainingDeadline::new(Duration::from_millis(deadline_millis))?,
        )?;
        let payload = &bytes[header_end..frame_end];
        validate_payload_shape(&envelope, payload.len())?;
        if Sha256::digest(payload).as_slice() != hash {
            return Err(DomainError::FrameIntegrity);
        }

        Ok(DecodedFrame {
            envelope,
            compression,
            payload: payload.to_vec(),
            payload_sha256: hash,
            trace_id,
            extensions,
        })
    }
}

fn header_len(
    envelope: &FrameEnvelope,
    model_len: usize,
    session_len: usize,
    shard_len: usize,
    trace_len: usize,
    extension_len: usize,
) -> DomainResult<usize> {
    let tensor_len = envelope
        .tensor
        .as_ref()
        .map_or(1, |tensor| 3 + tensor.dimensions.len() * 4);
    let trace_field_len = if trace_len == 0 { 1 } else { 2 + trace_len };
    1usize
        .checked_add(session_len)
        .and_then(|value| {
            value.checked_add(
                16 + 1
                    + model_len
                    + 1
                    + shard_len
                    + 8
                    + trace_field_len
                    + tensor_len
                    + 32
                    + extension_len,
            )
        })
        .ok_or(DomainError::FrameBoundsExceeded)
}

fn validate_payload_shape(envelope: &FrameEnvelope, payload_len: usize) -> DomainResult<()> {
    let expected = envelope
        .tensor
        .as_ref()
        .map(TensorDescriptor::byte_len)
        .unwrap_or(0);
    if u64::try_from(payload_len).ok() != Some(expected) {
        return Err(DomainError::FrameBoundsExceeded);
    }
    Ok(())
}

fn deadline_millis(deadline: RemainingDeadline) -> DomainResult<u64> {
    let millis = u64::try_from(deadline.duration().as_millis())
        .map_err(|_| DomainError::DeadlineExceeded)?;
    if millis == 0 || millis > MAX_DEADLINE_MILLIS {
        return Err(DomainError::DeadlineExceeded);
    }
    Ok(millis)
}

fn message_type_tag(message_type: FrameMessageType) -> u8 {
    match message_type {
        FrameMessageType::Data => 0,
        FrameMessageType::Ack => 1,
        FrameMessageType::Nack => 2,
        FrameMessageType::Cancel => 3,
        FrameMessageType::Heartbeat => 4,
        FrameMessageType::Error => 5,
    }
}

fn message_type_from_tag(value: u8) -> DomainResult<FrameMessageType> {
    match value {
        0 => Ok(FrameMessageType::Data),
        1 => Ok(FrameMessageType::Ack),
        2 => Ok(FrameMessageType::Nack),
        3 => Ok(FrameMessageType::Cancel),
        4 => Ok(FrameMessageType::Heartbeat),
        5 => Ok(FrameMessageType::Error),
        _ => Err(DomainError::FrameInvalid),
    }
}

fn push_tensor(result: &mut Vec<u8>, tensor: Option<&TensorDescriptor>) -> DomainResult<()> {
    let Some(tensor) = tensor else {
        result.push(0);
        return Ok(());
    };
    result.push(1);
    result.push(match tensor.dtype {
        TensorDtype::F32 => 1,
        TensorDtype::U32 => 2,
    });
    result
        .push(u8::try_from(tensor.dimensions.len()).map_err(|_| DomainError::FrameBoundsExceeded)?);
    for dimension in &tensor.dimensions {
        push_u32(result, *dimension);
    }
    Ok(())
}

fn push_trace(result: &mut Vec<u8>, trace_id: Option<&SafeTraceId>) -> DomainResult<()> {
    match trace_id {
        Some(trace_id) => {
            result.push(1);
            push_short_bytes(result, trace_id.as_str().as_bytes())?;
        }
        None => result.push(0),
    }
    Ok(())
}

fn extensions_len(extensions: &[FrameExtension]) -> DomainResult<usize> {
    extensions.iter().try_fold(0usize, |total, extension| {
        let value_len =
            u16::try_from(extension.value.len()).map_err(|_| DomainError::FrameBoundsExceeded)?;
        total
            .checked_add(3 + usize::from(value_len))
            .ok_or(DomainError::FrameBoundsExceeded)
    })
}

fn push_extensions(result: &mut Vec<u8>, extensions: &[FrameExtension]) -> DomainResult<()> {
    for extension in extensions {
        let value_len =
            u16::try_from(extension.value.len()).map_err(|_| DomainError::FrameBoundsExceeded)?;
        result.push(extension.tag);
        push_u16(result, value_len);
        result.extend_from_slice(&extension.value);
    }
    Ok(())
}

fn push_short_bytes(result: &mut Vec<u8>, value: &[u8]) -> DomainResult<()> {
    result.push(u8::try_from(value.len()).map_err(|_| DomainError::FrameBoundsExceeded)?);
    result.extend_from_slice(value);
    Ok(())
}

fn push_u16(result: &mut Vec<u8>, value: u16) {
    result.extend_from_slice(&value.to_be_bytes());
}

fn push_u32(result: &mut Vec<u8>, value: u32) {
    result.extend_from_slice(&value.to_be_bytes());
}

fn push_u64(result: &mut Vec<u8>, value: u64) {
    result.extend_from_slice(&value.to_be_bytes());
}

fn read_u16(bytes: &[u8], offset: usize) -> DomainResult<u16> {
    let values: [u8; 2] = bytes
        .get(offset..offset + 2)
        .ok_or(DomainError::FrameInvalid)?
        .try_into()
        .map_err(|_| DomainError::FrameInvalid)?;
    Ok(u16::from_be_bytes(values))
}

fn read_u32(bytes: &[u8], offset: usize) -> DomainResult<u32> {
    let values: [u8; 4] = bytes
        .get(offset..offset + 4)
        .ok_or(DomainError::FrameInvalid)?
        .try_into()
        .map_err(|_| DomainError::FrameInvalid)?;
    Ok(u32::from_be_bytes(values))
}

struct HeaderReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> HeaderReader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn read_u64(&mut self) -> DomainResult<u64> {
        let bytes: [u8; 8] = self
            .read_exact(8)?
            .try_into()
            .map_err(|_| DomainError::FrameInvalid)?;
        Ok(u64::from_be_bytes(bytes))
    }

    fn read_short_string(&mut self, maximum: usize) -> DomainResult<String> {
        let length = usize::from(
            *self
                .read_exact(1)?
                .first()
                .ok_or(DomainError::FrameInvalid)?,
        );
        if length > maximum {
            return Err(DomainError::FrameBoundsExceeded);
        }
        let value = self.read_exact(length)?;
        String::from_utf8(value.to_vec()).map_err(|_| DomainError::FrameInvalid)
    }

    fn read_trace_id(&mut self) -> DomainResult<Option<SafeTraceId>> {
        match *self
            .read_exact(1)?
            .first()
            .ok_or(DomainError::FrameInvalid)?
        {
            0 => Ok(None),
            1 => Ok(Some(SafeTraceId::new(
                self.read_short_string(MAX_TRACE_ID_BYTES)?,
            )?)),
            _ => Err(DomainError::FrameInvalid),
        }
    }

    fn read_tensor(&mut self) -> DomainResult<Option<TensorDescriptor>> {
        match *self
            .read_exact(1)?
            .first()
            .ok_or(DomainError::FrameInvalid)?
        {
            0 => return Ok(None),
            1 => {}
            _ => return Err(DomainError::FrameInvalid),
        }
        let dtype = match *self
            .read_exact(1)?
            .first()
            .ok_or(DomainError::FrameInvalid)?
        {
            1 => TensorDtype::F32,
            2 => TensorDtype::U32,
            _ => return Err(DomainError::FrameDtypeUnsupported),
        };
        let rank = usize::from(
            *self
                .read_exact(1)?
                .first()
                .ok_or(DomainError::FrameInvalid)?,
        );
        if rank == 0 || rank > MAX_TENSOR_RANK {
            return Err(DomainError::FrameBoundsExceeded);
        }
        let mut dimensions = Vec::with_capacity(rank);
        for _ in 0..rank {
            let bytes: [u8; 4] = self
                .read_exact(4)?
                .try_into()
                .map_err(|_| DomainError::FrameInvalid)?;
            dimensions.push(u32::from_be_bytes(bytes));
        }
        Ok(Some(TensorDescriptor::new(dtype, dimensions)?))
    }

    fn read_hash(&mut self) -> DomainResult<[u8; 32]> {
        self.read_exact(32)?
            .try_into()
            .map_err(|_| DomainError::FrameInvalid)
    }

    fn read_extensions(&mut self) -> DomainResult<Vec<FrameExtension>> {
        let mut extensions = Vec::new();
        while !self.is_finished() {
            let tag = *self
                .read_exact(1)?
                .first()
                .ok_or(DomainError::FrameInvalid)?;
            if tag == 0 {
                return Err(DomainError::FrameInvalid);
            }
            let bytes: [u8; 2] = self
                .read_exact(2)?
                .try_into()
                .map_err(|_| DomainError::FrameInvalid)?;
            let value = self
                .read_exact(usize::from(u16::from_be_bytes(bytes)))?
                .to_vec();
            extensions.push(FrameExtension::new(tag, value)?);
        }
        Ok(extensions)
    }

    fn read_exact(&mut self, length: usize) -> DomainResult<&'a [u8]> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(DomainError::FrameBoundsExceeded)?;
        let result = self
            .bytes
            .get(self.offset..end)
            .ok_or(DomainError::FrameInvalid)?;
        self.offset = end;
        Ok(result)
    }

    const fn is_finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{DecodedFrame, FrameCodec, FrameCompression, FrameExtension, SafeTraceId};
    use crate::{DomainError, ModelReference};

    use super::super::{
        FrameEnvelope, FrameMessageType, FrameProtocolVersion, FrameSequence, FrameTarget,
        RemainingDeadline, SessionId, ShardId, StreamId, TensorDescriptor, TensorDtype,
    };

    fn envelope() -> FrameEnvelope {
        FrameEnvelope::new(
            FrameProtocolVersion::current(),
            FrameMessageType::Data,
            SessionId::new("session-00000001".to_owned()).expect("fixture session is valid"),
            StreamId::new(1).expect("fixture stream is valid"),
            FrameSequence::initial(),
            FrameTarget {
                model: ModelReference::parse(format!(
                    "registry://fixtures/tinyllama@sha256:{}",
                    "a".repeat(64)
                ))
                .expect("fixture model is valid"),
                shard: ShardId::new("first".to_owned()).expect("fixture shard is valid"),
            },
            Some(
                TensorDescriptor::new(TensorDtype::F32, vec![1, 2])
                    .expect("fixture tensor is valid"),
            ),
            RemainingDeadline::new(Duration::from_millis(500)).expect("fixture deadline is valid"),
        )
        .expect("fixture envelope is valid")
    }

    fn encoded() -> Vec<u8> {
        FrameCodec::encode(
            &envelope(),
            &[0, 0, 128, 63, 0, 0, 0, 64],
            FrameCompression::None,
            Some(&SafeTraceId::new("trace-0000000001".to_owned()).expect("fixture trace is valid")),
        )
        .expect("fixture frame should encode")
    }

    #[test]
    fn round_trips_a_canonical_data_frame() {
        let decoded = FrameCodec::decode(&encoded()).expect("encoded frame should decode");

        assert_eq!(decoded.envelope, envelope());
        assert_eq!(decoded.envelope.sequence.value(), 0);
        assert_eq!(decoded.payload, vec![0, 0, 128, 63, 0, 0, 0, 64]);
        assert_eq!(decoded.compression, FrameCompression::None);
        assert_eq!(
            decoded.trace_id.expect("trace should be present").as_str(),
            "trace-0000000001"
        );
    }

    #[test]
    fn round_trips_a_frame_without_an_optional_trace_id() {
        let encoded = FrameCodec::encode(
            &envelope(),
            &[0, 0, 128, 63, 0, 0, 0, 64],
            FrameCompression::None,
            None,
        )
        .expect("frame without trace should encode");

        let decoded = FrameCodec::decode(&encoded).expect("frame without trace should decode");
        assert!(decoded.trace_id.is_none());
    }

    #[test]
    fn preserves_bounded_additive_header_extensions() {
        let extension =
            FrameExtension::new(1, b"SYN-FRAME-004".to_vec()).expect("fixture extension is valid");
        let encoded = FrameCodec::encode_with_extensions(
            &envelope(),
            &[0, 0, 128, 63, 0, 0, 0, 64],
            FrameCompression::None,
            None,
            std::slice::from_ref(&extension),
        )
        .expect("frame with extension should encode");

        let decoded = FrameCodec::decode(&encoded).expect("frame with extension should decode");
        assert_eq!(decoded.extensions(), [extension]);
    }

    #[test]
    fn canonical_data_frame_matches_the_golden_vector() {
        const GOLDEN: &[u8] = &[
            83, 89, 78, 70, 0, 1, 0, 0, 0, 0, 0, 210, 0, 0, 0, 8, 16, 115, 101, 115, 115, 105, 111,
            110, 45, 48, 48, 48, 48, 48, 48, 48, 49, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
            0, 101, 114, 101, 103, 105, 115, 116, 114, 121, 58, 47, 47, 102, 105, 120, 116, 117,
            114, 101, 115, 47, 116, 105, 110, 121, 108, 108, 97, 109, 97, 64, 115, 104, 97, 50, 53,
            54, 58, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97,
            97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97,
            97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97,
            5, 102, 105, 114, 115, 116, 0, 0, 0, 0, 0, 0, 1, 244, 1, 16, 116, 114, 97, 99, 101, 45,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 49, 1, 1, 2, 0, 0, 0, 1, 0, 0, 0, 2, 185, 200, 11,
            90, 222, 202, 69, 7, 83, 161, 105, 80, 195, 204, 101, 93, 39, 31, 123, 239, 122, 72,
            91, 200, 63, 17, 43, 114, 254, 242, 29, 55, 0, 0, 128, 63, 0, 0, 0, 64,
        ];
        let actual = encoded();
        assert_eq!(actual.len(), GOLDEN.len());
        for (index, (actual, expected)) in actual.iter().zip(GOLDEN).enumerate() {
            assert_eq!(actual, expected, "golden byte differs at index {index}");
        }
    }

    #[test]
    fn rejects_unsupported_versions_before_header_decode() {
        let mut frame = encoded();
        frame[5] = 2;
        assert!(matches!(
            FrameCodec::decode(&frame),
            Err(DomainError::ProtocolUnsupported)
        ));
    }

    #[test]
    fn rejects_corruption_and_truncated_packets() {
        let mut corrupted = encoded();
        let last = corrupted.len() - 1;
        corrupted[last] ^= 1;
        assert!(matches!(
            FrameCodec::decode(&corrupted),
            Err(DomainError::FrameIntegrity)
        ));

        let mut truncated = encoded();
        truncated.pop();
        assert!(matches!(
            FrameCodec::decode(&truncated),
            Err(DomainError::FrameInvalid)
        ));

        for length in 0..encoded().len() {
            let candidate = &encoded()[..length];
            assert!(FrameCodec::decode(candidate).is_err());
        }
    }

    #[test]
    fn rejects_oversized_payloads_and_unsupported_compression_without_decompression() {
        let mut oversized = encoded();
        oversized[12..16].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(matches!(
            FrameCodec::decode(&oversized),
            Err(DomainError::FrameBoundsExceeded)
        ));

        let mut compressed = encoded();
        compressed[7] = 1;
        assert!(matches!(
            FrameCodec::decode(&compressed),
            Err(DomainError::FrameInvalid)
        ));
    }

    #[test]
    fn rejects_malformed_tags_and_never_panics_on_bounded_noise() {
        let mut magic = encoded();
        magic[0] ^= 1;
        assert!(matches!(
            FrameCodec::decode(&magic),
            Err(DomainError::FrameInvalid)
        ));

        let mut message = encoded();
        message[6] = u8::MAX;
        assert!(matches!(
            FrameCodec::decode(&message),
            Err(DomainError::FrameInvalid)
        ));

        let mut state = 0x5A17_1CE5_u32;
        for length in 0..512 {
            let mut candidate = Vec::with_capacity(length);
            for _ in 0..length {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                candidate.push((state >> 24) as u8);
            }
            let _ = FrameCodec::decode(&candidate);
        }
    }

    #[test]
    fn rejects_unknown_dtype_and_incorrect_sequence_shape() {
        let mut dtype = encoded();
        let tensor_tag_offset = dtype.len() - 8 - 32 - 11;
        dtype[tensor_tag_offset + 1] = 3;
        assert!(matches!(
            FrameCodec::decode(&dtype),
            Err(DomainError::FrameDtypeUnsupported)
        ));

        let control = FrameEnvelope::new(
            FrameProtocolVersion::current(),
            FrameMessageType::Ack,
            envelope().session_id,
            StreamId::new(1).expect("fixture stream is valid"),
            FrameSequence::initial(),
            envelope().target,
            None,
            RemainingDeadline::new(Duration::from_millis(1)).expect("fixture deadline is valid"),
        )
        .expect("control envelope is valid");
        assert!(matches!(
            FrameCodec::encode(&control, &[1], FrameCompression::None, None),
            Err(DomainError::FrameBoundsExceeded)
        ));
    }

    #[test]
    fn safely_ignores_well_formed_additive_header_extensions() {
        let mut frame = encoded();
        let header_len = u32::from_be_bytes(
            frame[8..12]
                .try_into()
                .expect("fixed prefix has a header length"),
        );
        frame[8..12].copy_from_slice(&(header_len + 4).to_be_bytes());
        let payload_offset = 16 + usize::try_from(header_len).expect("fixture header fits usize");
        frame.splice(payload_offset..payload_offset, [7, 0, 1, 42]);

        let decoded = FrameCodec::decode(&frame).expect("additive extension should decode");
        assert_eq!(decoded.payload, vec![0, 0, 128, 63, 0, 0, 0, 64]);
        assert_eq!(decoded.extensions()[0].tag(), 7);
        assert_eq!(decoded.extensions()[0].value(), [42]);
    }

    #[test]
    fn decoded_frame_exposes_only_the_verified_hash() {
        let DecodedFrame { payload_sha256, .. } =
            FrameCodec::decode(&encoded()).expect("frame should decode");
        assert_ne!(payload_sha256, [0; 32]);
    }
}
