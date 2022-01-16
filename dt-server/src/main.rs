use std::path::PathBuf;

use dt_core::{config::DTConfig, utils::default_config_path};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    global_settings(&[structopt::clap::AppSettings::ColoredHelp])
)]
struct Opt {
    #[structopt(help = "Specifies path to config file", short, long)]
    config_path: Option<PathBuf>,

    #[structopt(
        help = "Specifies a directory to serve static files from",
        short,
        long
    )]
    static_dir: Option<PathBuf>,

    #[structopt(
        help = "Specifies the url prefix for served items",
        short,
        long
    )]
    root: Option<String>,

    #[structopt(
        help = "Increases logging verbosity",
        short,
        long,
        parse(from_occurrences)
    )]
    verbose: i8,

    #[structopt(
        help = "Decreases logging verbosity",
        short,
        long,
        parse(from_occurrences),
        conflicts_with = "verbose"
    )]
    quiet: i8,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    setup(opt.verbose - opt.quiet);

    let config_path = match opt.config_path {
        Some(p) => {
            log::debug!(
                "Using config file from command line '{}'",
                p.display(),
            );
            p
        }
        None => {
            let p = default_config_path("server.toml");
            let p = if p.exists() {
                p
            } else {
                default_config_path("config.toml")
            };
            log::debug!("Using inferred config file '{}'", p.display());
            p
        }
    };

    let config = match DTConfig::from_path(config_path) {
        Ok(config) => config,
        Err(e) => {
            log::error!("{}", e);
            std::process::exit(1);
        }
    };
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
