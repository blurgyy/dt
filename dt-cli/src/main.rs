use std::path::PathBuf;

use color_eyre::Report;
use structopt::StructOpt;

use dt_core::{config::DTConfig, syncing};

#[derive(StructOpt, Debug)]
#[structopt(
    global_settings(&[structopt::clap::AppSettings::ColoredHelp])
)]
struct Args {
    #[structopt(help = "Path to config file", parse(from_os_str))]
    config_path: PathBuf,

    #[structopt(
        help = "Show changes to be made without actually syncing files",
        short,
        long
    )]
    dry_run: bool,
}

fn main() -> Result<(), Report> {
    setup()?;

    let opt = Args::from_args();
    let config: DTConfig = DTConfig::from_pathbuf(opt.config_path)?;
    if opt.dry_run {
        syncing::dry_sync(&config)?;
    } else {
        syncing::sync(&config)?;
    }

    Ok(())
}

fn setup() -> Result<(), Report> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    pretty_env_logger::init();
    color_eyre::install()?;

    Ok(())
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 20 2021, 23:23 [CST]
