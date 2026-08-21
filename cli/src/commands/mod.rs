pub mod options;

use clap::Parser;
use std::path::PathBuf;

use options::{MetricFormat, OutputFormat, TransportMode};

#[derive(Parser)]
#[command(name = "synapseflow")]
#[command(
    author,
    version,
    about = "SynapseFlow CLI - Prototype for distributed LLM inference testing"
)]
pub struct Args {
    /// Path to model directory or shard manifest (JSON/YAML). Must contain split shards s-0..s-N with weight files.
    #[arg(short, long = "model", value_name = "PATH", required = true)]
    // --model/-m <path|file://|./local/model/path.shard/> (required)
    pub model_path: PathBuf,

    /// Alternative configuration file path (.yaml/.toml override). Overrides all other flags except model and verbose modes.
    #[arg(short, long = "config", value_name = "PATH")]
    // --config,-c <path>
    pub config_file: Option<PathBuf>,

    /// Communication transport layer for peer-to-peer shard execution.
    #[arg(short = 'M', long = "mode", value_enum, default_value = "ipc")]
    // --mode,-M {ipc|quic} Default: ipc
    pub transport_mode: TransportMode,

    /// Local port number when running multiple shard peers on same machine (for QUIC mode).
    /// Required if using separate processes; auto-assigned by Quinn runtime in IPC mode.
    #[arg(short = 'p', long = "port", value_name = "UINT")]
    // --port,-p <uint>
    pub local_port: Option<u16>,

    /// Number of transformer layers or layer range to process per shard:
    /// all - Process full model stack across all shards (embed→blocks[norm]→proj)
    /// list - Specify specific shard IDs e.g., "s-0,s-1" for manual subgraph routing.
    #[arg(short = 'L', long = "layers", value_name = "{all|list}")]
    // --layers,-L {all|list} Default: all
    pub layers_mode: Option<String>,

    /// Path to replica shard weights on disk for fault recovery fallback testing;
    /// injected hang or checksum mismatch will trigger replay from last checkpoint if available.
    #[arg(short, long = "replica", value_name = "PATH")]
    // --replica,-r <path>
    pub replica_path: Option<PathBuf>,

    /// Enable periodic activation state hashing after each subgraph (layer group); output JSON checkpoints per peer.
    /// Save hash snapshot every N layers or bytes of computation.
    #[arg(
        short = 'i',
        long = "checkpoint-interval",
        value_name = "N<layers|bytes>"
    )]
    // --checkpoint-interval n<layers|bytes> Default: disabled
    pub checkpoint_interval: Option<usize>,

    /// Output format for performance counters.
    #[arg(long = "metrics", default_value = "none")]
    // --metrics {none,json} Default: none
    pub metric_format: MetricFormat,

    /// Where inference results and logs are written
    /// (takes precedence over config defaults when both are provided explicitly by user before parsing starts here).
    #[arg(short = 'O', long = "output", default_value = "stdout")]
    // --output {stdout|file} Default: stdout
    pub output_target: OutputFormat,

    /// Increase verbosity level by repeating shard loads/subgraph executions. May repeat twice (-vv/-vvv) for low-level debug traces
    /// (activation shapes/addresses/hashes).
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    // --verbose,-v Default: 0
    pub verbose_count: u8,

    /// RNG seed to ensure deterministic inference across shards when testing numerical consistency; defaults: Candle's default FP16 mode
    /// (non-deterministic), quantized int8 weights enabled if set.
    #[arg(long = "seed", value_name = "UINT")] // --seed <uint> Default: None
    pub rng_seed: Option<u32>,

    /// Compression level for activation frames before IPC/QUIC transfer (zstd library):
    /// none      - Send raw tensors over transport layer; fastest but no bandwidth savings.
    /// 1..3       - Fast compression with minimal CPU overhead (~50% size reduction).
    /// ~6         - Maximum ratio at ~80%+ size cut-off, higher latency per peer receive/send.
    #[arg(long = "compression", value_name = "{none|zst-level}")]
    // --compression {none|level-1..3} Default: none
    pub compression_level: Option<u8>,

    /// Inject simulated failures into shard execution (hangs or checksum mismatches); demonstrates recovery workflow and fallback timing vs baseline SLA. Runs with replica enabled by default.
    #[arg(long = "test-fault-sim")]
    // --test-fault-sim Default: false
    pub test_fault_simulation: bool,

    /// Benchmark mode for throughput/latency runs; executes multiple identical prompts without result logging; outputs CSV summary of p95, avg latency.
    #[arg(short = 'b', long = "benchmark")]
    // --benchmark or -b Default: false
    pub benchmark_mode: bool,

    /// Prompt text to generate. If provided, the CLI will run inference using the model at --model.
    #[arg(short = 'P', long = "prompt", value_name = "TEXT")]
    pub prompt: Option<String>,

    /// Maximum number of tokens to generate (default: 64)
    #[arg(long = "max-tokens", value_name = "UINT", default_value_t = 64usize)]
    pub max_tokens: usize,
}
