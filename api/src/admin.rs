//! Admin endpoints: model/shard management, peer registry operations, system diagnostics.
//!
//! Endpoint responsibilities:
//! - `/admin/models`: upload signed manifests for new models
//! - `/admin/health`: report uptime, shard counts, peer metrics
