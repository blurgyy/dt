use std::path::PathBuf;

use color_eyre::Report;
use structopt::StructOpt;

use dt_core::config::DTConfig;

#[derive(StructOpt, Debug)]
#[structopt(
    global_settings(&[structopt::clap::AppSettings::ColoredHelp])
)]
struct Args {
    #[structopt(help = "Path to config file", parse(from_os_str))]
    config_path: PathBuf,
}

fn main() -> Result<(), Report> {
    setup()?;
    println!("Hello, world!");

    let opt = Args::from_args();
    let config: DTConfig = DTConfig::from_pathbuf(opt.config_path)?;

    dbg!(config);

    Ok(())
}

fn setup() -> Result<(), Report> {
    pretty_env_logger::init();
    color_eyre::install()?;

    Ok(())
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 20 2021, 23:23 [CST]
