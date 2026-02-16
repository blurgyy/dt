use std::path::PathBuf;

use structopt::StructOpt;

use dt_core::{
    collecting::{self, default_state_path},
    config::DTConfig,
    error::{Error as AppError, Result},
    syncing,
    utils::default_config_path,
};

#[derive(StructOpt, Debug)]
#[structopt(
    global_settings(&[structopt::clap::AppSettings::ColoredHelp]),
    name = "dt",
    about = "Dotfiles manager with reverse collection support"
)]
enum Opt {
    /// Sync files from source to target (default)
    #[structopt(name = "sync", alias = "s")]
    Sync(SyncArgs),
    
    /// Collect changes from target back to source
    #[structopt(name = "collect", alias = "c")]
    Collect(CollectArgs),
}

#[derive(StructOpt, Debug)]
struct SyncArgs {
    /// Specifies path to config file
    #[structopt(short, long)]
    config_path: Option<PathBuf>,

    /// Specifies name(s) of the group(s) to be processed
    #[structopt(name = "group_name")]
    group_names: Vec<String>,

    /// Shows changes to be made without actually syncing files
    #[structopt(short, long)]
    dry_run: bool,

    /// Increases logging verbosity
    #[structopt(short, long, parse(from_occurrences), conflicts_with = "quiet")]
    verbose: i8,

    /// Decreases logging verbosity
    #[structopt(short, long, parse(from_occurrences), conflicts_with = "verbose")]
    quiet: i8,
}

#[derive(StructOpt, Debug)]
struct CollectArgs {
    /// Specifies path to config file
    #[structopt(short, long)]
    config_path: Option<PathBuf>,

    /// Specifies name(s) of the group(s) to be processed
    #[structopt(name = "group_name")]
    group_names: Vec<String>,

    /// Shows changes to be made without actually collecting files
    #[structopt(short, long)]
    dry_run: bool,

    /// Specifies path to state file
    #[structopt(short, long)]
    state_path: Option<PathBuf>,

    /// Skip files with uncommitted changes instead of aborting
    #[structopt(long)]
    skip_dirty: bool,

    /// Increases logging verbosity
    #[structopt(short, long, parse(from_occurrences), conflicts_with = "quiet")]
    verbose: i8,

    /// Decreases logging verbosity
    #[structopt(short, long, parse(from_occurrences), conflicts_with = "verbose")]
    quiet: i8,
}

fn run() -> Result<()> {
    let opt = Opt::from_args();
    
    match opt {
        Opt::Sync(args) => run_sync(args),
        Opt::Collect(args) => run_collect(args),
    }
}

fn run_sync(args: SyncArgs) -> Result<()> {
    setup(args.verbose - args.quiet + { args.dry_run as i8 });
    log::trace!("Parsed command line: {:?}", &args);

    let config_path = match args.config_path {
        Some(p) => {
            log::debug!("Using config file '{}' (from command line)", p.display());
            p
        }
        None => default_config_path("DT_CLI_CONFIG_PATH", "DT_CONFIG_DIR", &["cli.toml"])?,
    };

    let config = DTConfig::from_path(config_path)?;
    // Filter groups when appropriate
    let config = if args.group_names.is_empty() {
        config
    } else {
        config.filter_names(args.group_names)
    };
    syncing::sync(config, args.dry_run)?;
    Ok(())
}

fn run_collect(args: CollectArgs) -> Result<()> {
    setup(args.verbose - args.quiet + { args.dry_run as i8 });
    log::trace!("Parsed command line: {:?}", &args);

    let config_path = match args.config_path {
        Some(p) => {
            log::debug!("Using config file '{}' (from command line)", p.display());
            p
        }
        None => default_config_path("DT_CLI_CONFIG_PATH", "DT_CONFIG_DIR", &["cli.toml"])?,
    };

    let state_path = match args.state_path {
        Some(p) => p,
        None => default_state_path(),
    };

    let config = DTConfig::from_path(config_path)?;
    // Filter groups when appropriate
    let config = if args.group_names.is_empty() {
        config
    } else {
        config.filter_names(args.group_names)
    };
    
    // Expand glob patterns in sources before collecting
    // Use expand_for_collect to avoid resolve() filtering out overlapping groups
    let config = syncing::expand_for_collect(config)?;
    
    let result = collecting::collect(&config, &state_path, args.dry_run, args.skip_dirty
    )?;
    
    // Handle conflicts
    if !result.conflicts.is_empty() {
        eprintln!("\nERROR: Cannot collect {} file(s) - source has uncommitted changes:\n", 
            result.conflicts.len());
        for conflict in &result.conflicts {
            eprintln!("{}", conflict);
            if let Some(status) = &conflict.git_status {
                eprintln!("    Git status: {}", status.trim());
            }
        }
        eprintln!("\nTo fix:");
        eprintln!("  1. Commit source changes first: cd <repo> && git add -A && git commit");
        eprintln!("  2. Or run with --skip-dirty to skip dirty files");
        eprintln!();
        
        return Err(AppError::SyncingError(
            format!("Collection aborted due to {} conflict(s)", result.conflicts.len())
        ));
    }
    
    // Output results
    if result.changes.is_empty() && result.skipped == 0 {
        log::info!("No changes detected.");
    } else {
        if !result.changes.is_empty() {
            log::info!("Detected {} change(s):", result.changes.len());
            for change in &result.changes {
                let change_type_str = match change.change_type {
                    collecting::ChangeType::New => "NEW",
                    collecting::ChangeType::Modified => "MODIFIED",
                    collecting::ChangeType::Deleted => "DELETED",
                };
                log::info!("  [{}] {}", change_type_str, change.relative_path.display());
            }
        }
        
        if result.skipped > 0 {
            log::warn!("Skipped {} file(s) with uncommitted changes", result.skipped);
        }
        
        if args.dry_run {
            log::info!("(Dry run - no changes were applied)");
        } else if !result.changes.is_empty() {
            log::info!("Changes have been collected to source.");
        }
    }
    
    Ok(())
}

fn setup(verbosity: i8) {
    match verbosity {
        i8::MIN..=-2 => unsafe { std::env::set_var("RUST_LOG", "error") },
        -1 => unsafe { std::env::set_var("RUST_LOG", "warn") },
        0 => unsafe {
            std::env::set_var(
                "RUST_LOG",
                std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned()),
            )
        },
        1 => unsafe { std::env::set_var("RUST_LOG", "debug") },
        2..=i8::MAX => unsafe { std::env::set_var("RUST_LOG", "trace") },
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
            AppError::ProcessError(_) => std::process::exit(8),

            #[allow(unreachable_patterns)]
            _ => std::process::exit(255),
        }
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 20 2021, 23:23 [CST]
