use std::path::PathBuf;

use color_eyre::Report;
use structopt::StructOpt;

use dt_core::{config::DTConfig, syncing, utils::default_config_path};

#[derive(StructOpt, Debug)]
#[structopt(
    global_settings(&[structopt::clap::AppSettings::ColoredHelp])
)]
struct Opt {
    #[structopt(help = "Specifies path to config file", short, long)]
    config_path: Option<PathBuf>,

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

fn main() -> Result<(), Report> {
    let opt = Opt::from_args();
    setup(opt.verbose - opt.quiet)?;

    let config: DTConfig = DTConfig::from_path(
        opt.config_path
            .unwrap_or_else(|| default_config_path("config.toml")),
    )?;
    if opt.dry_run {
        syncing::dry_sync(&config)?;
    } else {
        syncing::sync(&config)?;
    }

    Ok(())
}

fn setup(verbosity: i8) -> Result<(), Report> {
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
    color_eyre::install()?;

    Ok(())
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 20 2021, 23:23 [CST]
