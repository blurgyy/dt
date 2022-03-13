use std::path::PathBuf;

use structopt::StructOpt;

use dt_core::{
    config::DTConfig,
    error::{Error as AppError, Result},
    syncing,
    utils::default_config_path,
};

#[derive(StructOpt, Debug)]
#[structopt(
    global_settings(&[structopt::clap::AppSettings::ColoredHelp])
)]
struct Opt {
    #[structopt(help = "Specifies path to config file", short, long)]
    config_path: Option<PathBuf>,

    #[structopt(
        name = "group_name",
        help = "Specifies name(s) of the group(s) to be processed"
    )]
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

fn run() -> Result<()> {
    let opt = Opt::from_args();
    setup(opt.verbose - opt.quiet + { opt.dry_run as i8 });

    log::trace!("Parsed command line: {:#?}", &opt);

    let config_path = match opt.config_path {
        Some(p) => {
            log::debug!(
                "Using config file '{}' (from command line)",
                p.display(),
            );
            p
        }
        None => default_config_path(
            "DT_CLI_CONFIG_PATH",
            "DT_CONFIG_DIR",
            &["cli.toml"],
        )?,
    };

    let config = DTConfig::from_path(config_path)?;
    // Filter groups when appropriate
    let config = if opt.group_names.is_empty() {
        config
    } else {
        config.filter_names(opt.group_names)
    };
    syncing::sync(config, opt.dry_run)?;
    Ok(())
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

fn main() {
    if let Err(e) = run() {
        log::error!("{}", e);
        match e {
            AppError::ConfigError(_) => std::process::exit(1),
            AppError::IoError(_) => std::process::exit(2),
            AppError::ParseError(_) => std::process::exit(3),
            AppError::PathError(_) => std::process::exit(4),
            AppError::RenderingError(_) => std::process::exit(5),
            AppError::SyncingError(_) => std::process::exit(6),
            AppError::TemplatingError(_) => std::process::exit(7),

            #[allow(unreachable_patterns)]
            _ => std::process::exit(255),
        }
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 20 2021, 23:23 [CST]
