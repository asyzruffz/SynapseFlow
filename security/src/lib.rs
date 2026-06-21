//! SynapseFlow Security Library
//!
//! Cryptographic verification, optional TEE attestation, and audit logging for checksums.

mod attestations; // Optional Intel SGX / AMD SEV integration (when available)
mod audits;
pub mod crypto; // Ed25519 signatures, TLS/DTLS support // Checksum verification + structured log outputs
