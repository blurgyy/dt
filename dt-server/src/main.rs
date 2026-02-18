use std::path::PathBuf;

use dt_core::{
    config::DTConfig,
    error::{Error as AppError, Result},
    utils::default_config_path,
};
use structopt::StructOpt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(StructOpt, Debug)]
#[structopt(
    global_settings(&[structopt::clap::AppSettings::ColoredHelp])
)]
struct Opt {
    /// Specifies path to config file
    #[structopt(short, long)]
    config_path: Option<PathBuf>,

    /// Specifies a directory to serve static files from
    #[structopt(short, long)]
    static_dir: Option<PathBuf>,

    ///Specifies the url prefix for served items
    #[structopt(short, long)]
    root: Option<String>,

    /// Increases logging verbosity
    #[structopt(short, long, parse(from_occurrences), conflicts_with = "quiet")]
    verbose: i8,

    /// Decreases logging verbosity
    #[structopt(short, long, parse(from_occurrences), conflicts_with = "verbose")]
    quiet: i8,
}

async fn run() -> Result<()> {
    let opt = Opt::from_args();
    setup(opt.verbose - opt.quiet);

    let config_path = match opt.config_path {
        Some(p) => {
            tracing::debug!("Using config file '{}' (from command line)", p.display());
            p
        }
        None => default_config_path("DT_SERVER_CONFIG_PATH", "DT_CONFIG_DIR", &["server.toml"])?,
    };

    let _config = DTConfig::from_path(config_path)?;
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        tracing::error!("{}", e);
        match e {
            AppError::ConfigError(_) => std::process::exit(1),
            AppError::IoError(_) => std::process::exit(2),
            AppError::ParseError(_) => std::process::exit(3),
            AppError::PathError(_) => std::process::exit(4),
            AppError::RenderingError(_) => std::process::exit(5),
            AppError::SyncingError(_) => std::process::exit(6),
            AppError::TemplatingError(_) => std::process::exit(7),
            AppError::ProcessError(_) => std::process::exit(8),
            #[allow(unreachable_patterns)]
            _ => std::process::exit(255),
        }
    }
}

fn setup(verbosity: i8) {
    // Map verbosity level to log level
    let log_level = match verbosity {
        i8::MIN..=-2 => "error",
        -1 => "warn",
        0 => "info",
        1 => "debug",
        2..=i8::MAX => "trace",
    };

    // Initialize tracing subscriber with env filter
    // Use RUST_LOG if set, otherwise use the verbosity-based level
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .without_time()
                .with_target(false),
        )
        .with(filter)
        .init();
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Oct 19 2021, 21:57 [CST]
