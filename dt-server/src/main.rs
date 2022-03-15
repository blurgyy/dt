use std::path::PathBuf;

use dt_core::{
    config::DTConfig,
    error::{Error as AppError, Result},
    utils::default_config_path,
};
use structopt::StructOpt;

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
    #[structopt(
        short,
        long,
        parse(from_occurrences),
        conflicts_with = "quiet"
    )]
    verbose: i8,

    /// Decreases logging verbosity
    #[structopt(
        short,
        long,
        parse(from_occurrences),
        conflicts_with = "verbose"
    )]
    quiet: i8,
}

async fn run() -> Result<()> {
    let opt = Opt::from_args();
    setup(opt.verbose - opt.quiet);

    let config_path = match opt.config_path {
        Some(p) => {
            log::debug!(
                "Using config file '{}' (from command line)",
                p.display(),
            );
            p
        }
        None => default_config_path(
            "DT_SERVER_CONFIG_PATH",
            "DT_CONFIG_DIR",
            &["server.toml"],
        )?,
    };

    let config = DTConfig::from_path(config_path)?;
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        log::error!("{}", e);
        match e {
            #[allow(unreachable_patterns)]
            _ => std::process::exit(255),
        }
    }
}

fn setup(verbosity: i8) {
    match verbosity {
        i8::MIN..=-2 => std::env::set_var("RUST_LOG", "error"),
        -1 => std::env::set_var("RUST_LOG", "warn"),
        0 => std::env::set_var(
            "RUST_LOG",
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned()),
        ),
        1 => std::env::set_var("RUST_LOG", "debug"),
        2..=i8::MAX => std::env::set_var("RUST_LOG", "trace"),
    }

    pretty_env_logger::init();
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Oct 19 2021, 21:57 [CST]
