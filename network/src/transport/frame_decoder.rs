//! Frame decoder for receiving chunked activations from peers.
//!
//! Responsibilities:
//! - Parse frame headers and extract metadata (shape, dtype)
//! - Decompress payload using zstd/snap decompression
//! - Verify checksums against expected values
//! - Forward NACK/RETRY control messages up the stack
