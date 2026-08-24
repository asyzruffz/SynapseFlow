use synapseflow_domain::execution::{DecodedFrame, FrameExtension, TensorDtype};
use synapseflow_domain::{DomainError, DomainResult};

/// Protocol-v1 extension tag for the first token position represented by a data frame.
pub const POSITION_START_EXTENSION_TAG: u8 = 2;

/// Validated stage input for the Llama-specific native adapter.
#[derive(Clone, Debug, PartialEq)]
pub enum StageInput {
    TokenIds {
        token_ids: Vec<u32>,
        position_start: u64,
    },
    Boundary {
        activations: Vec<f32>,
        position_start: u64,
    },
}

pub fn parse_stage_input(frame: &DecodedFrame, first_stage: bool) -> DomainResult<StageInput> {
    let tensor = frame
        .envelope
        .tensor
        .as_ref()
        .ok_or(DomainError::FrameInvalid)?;
    let position_start = position_start(frame.extensions())?;
    match (first_stage, tensor.dtype) {
        (true, TensorDtype::U32) => Ok(StageInput::TokenIds {
            token_ids: parse_u32(&frame.payload)?,
            position_start,
        }),
        (false, TensorDtype::F32) => Ok(StageInput::Boundary {
            activations: parse_f32(&frame.payload)?,
            position_start,
        }),
        _ => Err(DomainError::FrameDtypeUnsupported),
    }
}

fn parse_u32(payload: &[u8]) -> DomainResult<Vec<u32>> {
    if payload.len() % 4 != 0 {
        return Err(DomainError::FrameInvalid);
    }
    payload
        .chunks_exact(4)
        .map(|bytes| {
            let bytes: [u8; 4] = bytes.try_into().map_err(|_| DomainError::FrameInvalid)?;
            Ok(u32::from_le_bytes(bytes))
        })
        .collect()
}

fn parse_f32(payload: &[u8]) -> DomainResult<Vec<f32>> {
    if payload.len() % 4 != 0 {
        return Err(DomainError::FrameInvalid);
    }
    payload
        .chunks_exact(4)
        .map(|bytes| {
            let bytes: [u8; 4] = bytes.try_into().map_err(|_| DomainError::FrameInvalid)?;
            Ok(f32::from_le_bytes(bytes))
        })
        .collect()
}

pub fn position_extension(position_start: u64) -> DomainResult<FrameExtension> {
    FrameExtension::new(
        POSITION_START_EXTENSION_TAG,
        position_start.to_be_bytes().to_vec(),
    )
}

fn position_start(extensions: &[FrameExtension]) -> DomainResult<u64> {
    let mut values = extensions
        .iter()
        .filter(|extension| extension.tag() == POSITION_START_EXTENSION_TAG);
    let value = values.next().ok_or(DomainError::FrameInvalid)?;
    if values.next().is_some() || value.value().len() != 8 {
        return Err(DomainError::FrameInvalid);
    }
    let bytes: [u8; 8] = value
        .value()
        .try_into()
        .map_err(|_| DomainError::FrameInvalid)?;
    Ok(u64::from_be_bytes(bytes))
}
