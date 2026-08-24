use crate::{DomainError, DomainResult};

/// Maximum tensor rank admitted by the first distributed frame contract.
pub const MAX_TENSOR_RANK: usize = 8;
/// Maximum uncompressed tensor payload admitted by the first distributed frame contract.
pub const MAX_TENSOR_BYTES: u64 = 64 * 1024 * 1024;

/// Tensor element encoding supported by the first layer-range boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TensorDtype {
    F32,
    U32,
}

impl TensorDtype {
    const fn bytes_per_element(self) -> u64 {
        match self {
            Self::F32 | Self::U32 => 4,
        }
    }
}

/// Validated tensor metadata before payload decode or allocation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TensorDescriptor {
    pub dtype: TensorDtype,
    pub dimensions: Vec<u32>,
    byte_len: u64,
}

impl TensorDescriptor {
    pub fn new(dtype: TensorDtype, dimensions: Vec<u32>) -> DomainResult<Self> {
        if dimensions.is_empty() || dimensions.len() > MAX_TENSOR_RANK {
            return Err(DomainError::FrameInvalid);
        }

        let elements = dimensions.iter().try_fold(1_u64, |count, dimension| {
            if *dimension == 0 {
                return None;
            }
            count.checked_mul(u64::from(*dimension))
        });
        let Some(elements) = elements else {
            return Err(DomainError::FrameInvalid);
        };
        let Some(byte_len) = elements.checked_mul(dtype.bytes_per_element()) else {
            return Err(DomainError::FrameInvalid);
        };
        if byte_len > MAX_TENSOR_BYTES {
            return Err(DomainError::FrameInvalid);
        }

        Ok(Self {
            dtype,
            dimensions,
            byte_len,
        })
    }

    pub const fn byte_len(&self) -> u64 {
        self.byte_len
    }
}

#[cfg(test)]
mod tests {
    use super::{TensorDescriptor, TensorDtype, MAX_TENSOR_BYTES};
    use crate::DomainError;

    #[test]
    fn calculates_valid_tensor_bytes_before_payload_handling() {
        let tensor = TensorDescriptor::new(TensorDtype::F32, vec![16, 2_048])
            .expect("bounded f32 tensor should be valid");

        assert_eq!(tensor.byte_len(), 131_072);

        let token_ids = TensorDescriptor::new(TensorDtype::U32, vec![16])
            .expect("bounded u32 token tensor should be valid");
        assert_eq!(token_ids.byte_len(), 64);
    }

    #[test]
    fn rejects_zero_or_oversized_dimensions() {
        assert!(matches!(
            TensorDescriptor::new(TensorDtype::F32, vec![0]),
            Err(DomainError::FrameInvalid)
        ));
        assert!(matches!(
            TensorDescriptor::new(TensorDtype::F32, vec![1, (MAX_TENSOR_BYTES / 4 + 1) as u32]),
            Err(DomainError::FrameInvalid)
        ));
    }
}
