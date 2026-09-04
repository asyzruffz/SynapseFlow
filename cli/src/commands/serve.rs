use std::path::PathBuf;

use clap::Args;

/// Startup overrides for the reusable node server library.
#[derive(Args, Debug)]
pub struct ServeCommand {
    /// TOML configuration file. Environment and explicit options take precedence.
    #[arg(long)]
    pub config: Option<PathBuf>,
    /// `development` is loopback-only; `operational` permits a protected public bind.
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long)]
    pub public_bind: Option<String>,
    #[arg(long)]
    pub management_bind: Option<String>,
    #[arg(long)]
    pub public_tls_cert_file: Option<PathBuf>,
    #[arg(long)]
    pub public_tls_key_file: Option<PathBuf>,
    #[arg(long, value_delimiter = ',')]
    pub trusted_proxy_address: Vec<String>,
    #[arg(long)]
    pub keycloak_issuer: Option<String>,
    #[arg(long)]
    pub keycloak_audience: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub keycloak_allowed_algorithm: Vec<String>,
    #[arg(long)]
    pub keycloak_jwks_max_staleness_seconds: Option<u64>,
    #[arg(long)]
    pub keycloak_clock_skew_seconds: Option<u64>,
    #[arg(long)]
    pub audit_directory: Option<PathBuf>,
    /// Immutable manifest reference allowed for generation. May be repeated.
    #[arg(long)]
    pub allowed_model: Vec<String>,
}
