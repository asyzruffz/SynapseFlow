use candle_core::{DType, Device, Result, Tensor};

pub(crate) const MAX_CONTEXT_TOKENS: usize = 4_096;

pub(crate) fn rope_frequencies(
    rope_dimension: usize,
    context_limit: usize,
    frequency_base: f32,
) -> Result<(Tensor, Tensor)> {
    let theta = (0..rope_dimension)
        .step_by(2)
        .map(|index| 1.0_f32 / frequency_base.powf(index as f32 / rope_dimension as f32))
        .collect::<Vec<_>>();
    let theta = Tensor::new(theta.as_slice(), &Device::Cpu)?;
    let positions = Tensor::arange(0, context_limit as u32, &Device::Cpu)?
        .to_dtype(DType::F32)?
        .reshape((context_limit, 1))?;
    let frequencies = positions.matmul(&theta.reshape((1, theta.elem_count()))?)?;
    Ok((frequencies.cos()?, frequencies.sin()?))
}

pub(crate) fn causal_mask(token_count: usize, position_start: usize) -> Result<Tensor> {
    let cache_length = position_start + token_count;
    let values = (0..token_count)
        .flat_map(|row| {
            (0..cache_length).map(move |column| u8::from(column > position_start + row))
        })
        .collect::<Vec<_>>();
    Tensor::from_slice(&values, (token_count, cache_length), &Device::Cpu)
}

pub(crate) fn repeat_key_values(values: Tensor, repeats: usize) -> Result<Tensor> {
    if repeats == 1 {
        return Ok(values);
    }
    let (batch, key_value_heads, token_count, head_dimension) = values.dims4()?;
    Tensor::cat(&vec![&values; repeats], 2)?.reshape((
        batch,
        key_value_heads * repeats,
        token_count,
        head_dimension,
    ))
}

#[cfg(test)]
mod tests {
    use super::{causal_mask, rope_frequencies};

    #[test]
    fn builds_a_rectangular_causal_mask_for_cached_tokens() {
        let values = causal_mask(3, 2)
            .and_then(|mask| mask.to_vec2::<u8>())
            .expect("mask should materialize on CPU");

        assert_eq!(
            values,
            vec![vec![0, 0, 0, 1, 1], vec![0, 0, 0, 0, 1], vec![0; 5]]
        );
    }

    #[test]
    fn bounds_rotary_frequencies_to_the_requested_context() {
        let (cos, sin) = rope_frequencies(4, 8, 10_000.0).expect("frequencies should build");

        assert_eq!(cos.dims(), [8, 2]);
        assert_eq!(sin.dims(), [8, 2]);
    }
}
