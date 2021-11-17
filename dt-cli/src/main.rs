use std::path::PathBuf;

use structopt::StructOpt;

use dt_core::{config::DTConfig, syncing, utils::default_config_path};

#[derive(StructOpt, Debug)]
#[structopt(
    global_settings(&[structopt::clap::AppSettings::ColoredHelp])
)]
struct Opt {
    #[structopt(help = "Specifies path to config file", short, long)]
    config_path: Option<PathBuf>,

    #[structopt(help = "Specifies name(s) of the group(s) to be processed")]
    group_names: Vec<String>,

    #[structopt(
        help = "Shows changes to be made without actually syncing files",
        short,
        long
    )]
    dry_run: bool,

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

fn main() {
    let opt = Opt::from_args();
    setup(opt.verbose - opt.quiet + { opt.dry_run as i8 });

    log::trace!("Parsed command line: {:#?}", &opt);

    let config_path = match opt.config_path {
        Some(p) => {
            log::debug!(
                "Using config file from command line '{}'",
                p.display(),
            );
            p
        }
        None => {
            let p = default_config_path("cli.toml");
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
    // Filter groups when appropriate
    let config = if opt.group_names.is_empty() {
        config
    } else {
        config.filter_names(opt.group_names)
    };
    if opt.dry_run {
        if let Err(e) = syncing::dry_sync(config) {
            log::error!("{}", e);
            std::process::exit(2);
        }
    } else if let Err(e) = syncing::sync(config) {
        log::error!("{}", e);
        std::process::exit(3);
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
// Date:   Sep 20 2021, 23:23 [CST]
