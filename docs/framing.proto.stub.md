# Frame Protocol Specification (Protobuf Stub)

This directory contains protocol buffer definitions for frame-level encoding. The spec defines:

```proto
// TODO: Implement after QUIC transport prototype is built
message ActivationFrame {
  uint64 session_id = 1;           // Unique request identifier from coordinator  
  int32 frame_index = 2;            // Sequence number (0..N) within this batch  
  bytes checksum_hash = 3;          // SHA-256 hash of payload for integrity check  
}
```

The protocol supports compression via zstd in the message body. Control frames use minimal headers with special `frame_type` enum values: FRAME, ACK/NACK/RETRY/CANCEL/HEARTBEAT.

