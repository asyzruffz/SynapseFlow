//! User-facing REST/gRPC inference endpoints for text generation requests.
//!
//! Endpoint responsibilities:
//! - `/v1/predict`: accept prompt, stream token results back to client
//! - Accept optional parameters (temperature, max_tokens) via query params
