# Third-party notices

## Candle

Loom uses `candle-core` and `candle-nn` version 0.11.0. Its focused Llama
layer, quantized-GGUF loading, rotary-position, causal-mask, and grouped-query
attention structure was adapted from Candle's quantized Llama implementation:

- <https://github.com/huggingface/candle/tree/0.11.0/candle-transformers/src/models>
- <https://docs.rs/candle-transformers/0.11.0/src/candle_transformers/models/quantized_llama.rs.html>

Candle is dual-licensed under Apache-2.0 and MIT. The complete licence texts
and upstream source are available at <https://github.com/huggingface/candle>.
Loom does not expose Candle types outside the adapter crate.
