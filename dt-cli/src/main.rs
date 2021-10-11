use std::{ops::Not, path::PathBuf};

use color_eyre::Report;
use structopt::StructOpt;

use dt_core::{config::DTConfig, syncing};

#[derive(StructOpt, Debug)]
#[structopt(
    global_settings(&[structopt::clap::AppSettings::ColoredHelp])
)]
struct Args {
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
    verbose: u8,
}

fn main() -> Result<(), Report> {
    let opt = Args::from_args();
    setup(opt.verbose)?;

    let config: DTConfig = DTConfig::from_pathbuf(
        opt.config_path.unwrap_or(default_config_path()?),
    )?;
    if opt.dry_run {
        syncing::dry_sync(&config)?;
    } else {
        syncing::sync(&config)?;
    }

    Ok(())
}

fn setup(verbosity: u8) -> Result<(), Report> {
    match verbosity {
        0 => std::env::set_var(
            "RUST_LOG",
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned()),
        ),
        1 => std::env::set_var("RUST_LOG", "debug"),
        _ => std::env::set_var("RUST_LOG", "trace"),
    }

    pretty_env_logger::init();
    color_eyre::install()?;

    Ok(())
}

/// Gets the default config file path,
fn default_config_path() -> Result<PathBuf, Report> {
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| panic!("Cannot determine default config path"))
        .join("dt")
        .join("cli.toml");

    if config_path.exists().not() {
        Err(color_eyre::eyre::eyre!(
            "No default config path could be inferred"
        ))
    } else {
        Ok(config_path)
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 20 2021, 23:23 [CST]
