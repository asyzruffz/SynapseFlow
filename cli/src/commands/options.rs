//! Command options for CLI

/// Transport mode enum for peer-to-peer communication
#[derive(clap::ValueEnum, Clone)]
pub enum TransportMode {
    /// Use local IPC (Unix sockets/named pipes) for fastest loopback testing. Preferred for CPU-only prototype development without QUIC overhead.
    Ipc,
    /// Use Quinn-based QUIC over TCP loopback (tcp://127.0.0.1:...). Recommended for realistic network simulation, latency measurement and later migration to multi-node setups.
    Quic,
}

/// Output format enumeration for metrics results
#[derive(clap::ValueEnum, Clone)]
pub enum MetricFormat {
    /// Suppress metric logging; only final result printed (best for interactive use)
    None,
    /// Stream Prometheus-compatible metrics as JSON per peer activation transfer. Best for automated dashboards or later integration with Grafana/Prometheus clients.
    Json,
    /// Minimal single-line log with latency/compression ratios post-inference; best for quick iteration during development cycles without external tools setup required yet.
    Compact,
}

/// Output format enumeration for result writing
#[derive(clap::ValueEnum, Clone)]
pub enum OutputFormat {
    /// Print to terminal stdout; best for debugging/monitoring and interactive testing sessions with humans in the loop involved at some point downline later.
    Stdout,
    /// Write output + metrics to specified path.
    File,
}
