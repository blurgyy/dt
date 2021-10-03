use std::{ops::Not, path::PathBuf, str::FromStr};

use color_eyre::{eyre::eyre, Report};

use crate::{config::*, utils};

/// Syncs items specified in configuration.
pub fn sync(config: &DTConfig) -> Result<(), Report> {
    let staging = &config
        .global
        .to_owned()
        .unwrap_or_default()
        .staging
        .unwrap_or(GlobalConfig::default().staging.unwrap());
    if staging.exists().not() {
        log::debug!(
            "Creating non-existing staging root {}",
            staging.display(),
        );
        std::fs::create_dir_all(staging)?;
    }

    for local in &config.local {
        let group_staging = staging.join(PathBuf::from_str(&local.name)?);
        if group_staging.exists().not() {
            log::debug!(
                "Creating non-existing staging directory {}",
                group_staging.display(),
            );
            std::fs::create_dir_all(&group_staging)?;
        }
        for spath in &local.sources {
            sync_recursive(
                spath,
                &local.target,
                false,
                local.get_allow_overwrite(
                    &config.global.to_owned().unwrap_or_default(),
                ),
                local.get_method(
                    &config.global.to_owned().unwrap_or_default(),
                ),
                &group_staging,
                &local.basedir,
                local.per_host.unwrap_or(false),
                &local.hostname_sep.as_ref().unwrap_or(&"@@".to_owned()),
            )?;
        }
    }
    Ok(())
}

/// Show changes to be made according to configuration, without actually syncing items.
pub fn dry_sync(config: &DTConfig) -> Result<(), Report> {
    let staging = &config
        .global
        .to_owned()
        .unwrap_or_default()
        .staging
        .unwrap_or(GlobalConfig::default().staging.unwrap());
    if staging.exists().not() {
        log::info!("Staging root does not exist, will be automatically created when syncing");
    } else if staging.is_dir().not() {
        log::error!("Staging root seems to exist and is not a directory");
    }

    for local in &config.local {
        let group_staging = staging.join(PathBuf::from_str(&local.name)?);
        if group_staging.exists().not() {
            log::info!("Staging directory does not exist, will be automatically created when syncing");
        } else if staging.is_dir().not() {
            log::info!(
                "Staging directory seems to exist and is not a directory"
            )
        }
        for spath in &local.sources {
            sync_recursive(
                spath,
                &local.target,
                true,
                local.get_allow_overwrite(
                    &config.global.to_owned().unwrap_or_default(),
                ),
                local.get_method(
                    &config.global.to_owned().unwrap_or_default(),
                ),
                &staging,
                &local.basedir,
                local.per_host.unwrap_or(false),
                &local.hostname_sep.as_ref().unwrap_or(&"@@".to_owned()),
            )?;
        }
    }
    Ok(())
}

/// Recursively syncs `spath` to a directory `tparent`.
///
/// Args:
///   - `spath`: Path to source item.
///   - `tparent`: Path to the parent dir of the disired sync destination.
///   - `dry`: Whether to issue a dry run.
///   - `allow_overwrite`: Whether overwrite existing files or not.
///   - `method`: A `SyncMethod` instance.
///   - `staging`: Path to staging directory.
///   - `per_host`: Whether to check if host-specific item is present.
fn sync_recursive(
    spath: &PathBuf,
    tparent: &PathBuf,
    dry: bool,
    allow_overwrite: bool,
    method: SyncMethod,
    staging: &PathBuf,
    basedir: &PathBuf,
    per_host: bool,
    hostname_sep: &str,
) -> Result<(), Report> {
    if tparent.exists().not() {
        if dry {
            log::info!(
                "DRYRUN> Stopping at non-existing target directory {}",
                tparent.display(),
            );
        } else {
            log::debug!("Creating target directory {}", tparent.display());
            std::fs::create_dir_all(tparent)?;
        }
    }
    let overwrite_log_level = if allow_overwrite {
        log::Level::Warn
    } else {
        log::Level::Error
    };

    // First, get target path (without the per-host suffix).
    let sname = spath.file_name().unwrap();
    let tpath = tparent.join(sname);

    // Next, update source path `spath` if `per_host` is set and a per-host item exists.
    let spath = if per_host {
        let per_host_spath = utils::to_host_specific(spath, hostname_sep)?;
        if per_host_spath.exists() {
            per_host_spath
        } else {
            spath.to_owned()
        }
    } else {
        spath.to_owned()
    };

    // Finally, get the staging path with updated source path
    let staging_path = staging.join(spath.strip_prefix(basedir)?);

    if spath.is_file() {
        if tpath.is_dir() {
            if dry {
                log::error!(
                    "DRYRUN> A directory ({}) exists at the target path of a source file ({})",
                    tpath.display(),
                    spath.display(),
                );
            } else {
                return Err(
                    eyre!(
                        "A directory ({}) exists at the target path of a source file ({})",
                        tpath.display(),
                        spath.display(),
                    )
                );
            };
        }

        if dry {
            if tpath.exists() {
                log::log!(
                    overwrite_log_level,
                    "DRYRUN> Target path ({}) exists",
                    tpath.display(),
                );
            }
            log::info!("DRYRUN> {} -> {}", spath.display(), tpath.display());
        } else {
            if tpath.exists() && !allow_overwrite {
                log::log!(
                    overwrite_log_level,
                    "SYNC::SKIP> Target path ({}) exists",
                    tpath.display(),
                );
            } else {
                // Allows overwrite in this block.
                if method == SyncMethod::Copy {
                    log::debug!(
                        "SYNC::COPY> {} => {}",
                        spath.display(),
                        tpath.display()
                    );
                    match std::fs::remove_file(&tpath) {
                        Ok(_) => log::trace!(
                            "SYNC::OVERWRITE> {}",
                            tpath.display()
                        ),
                        _ => {}
                    }
                    std::fs::copy(spath, tpath)?;
                } else if method == SyncMethod::Symlink {
                    // Staging
                    log::debug!(
                        "SYNC::STAGE> {} => {}",
                        spath.display(),
                        staging_path.display(),
                    );

                    match std::fs::remove_file(&staging_path) {
                        Ok(_) => log::trace!(
                            "SYNC::OVERWRITE> {}",
                            staging_path.display(),
                        ),
                        _ => {}
                    }
                    std::fs::copy(spath, &staging_path)?;

                    // Symlinking
                    log::debug!(
                        "SYNC::SYMLINK> {} => {}",
                        staging_path.display(),
                        tpath.display(),
                    );
                    match std::fs::remove_file(&tpath) {
                        Ok(_) => {
                            log::trace!(
                                "SYNC::OVERWRITE> {}",
                                tpath.display(),
                            );
                        }
                        _ => {}
                    }
                    std::os::unix::fs::symlink(staging_path, tpath)?;
                }
            }
        }
    } else if spath.is_dir() {
        if tpath.is_file() {
            if dry {
                log::error!(
                    "DRYRUN> A file ({}) exists at the target path of a source directory ({})",
                    tpath.display(),
                    spath.display(),
                );
            } else {
                return Err(
                    eyre!(
                        "A file ({}) exists at the target path of a source directory ({})",
                        tpath.display(),
                        spath.display(),
                    )
                );
            };
        }

        if tpath.exists().not()
            || method == SyncMethod::Symlink && !staging_path.exists()
        {
            if dry {
                log::info!(
                    "DRYRUN> Stopping recursion at non-existing directory {}",
                    tpath.display(),
                );
                return Ok(());
            } else {
                if method == SyncMethod::Symlink {
                    log::debug!(
                        "SYNC::STAGE::CREATE> {}",
                        staging_path.display(),
                    );
                    std::fs::create_dir_all(staging_path)?;
                }
                log::debug!("SYNC::CREATE> {}", tpath.display());
                std::fs::create_dir_all(&tpath)?;
            }
        }

        for item in std::fs::read_dir(spath)? {
            let item = item?;
            sync_recursive(
                &item.path(),
                &tpath,
                dry,
                allow_overwrite,
                method,
                staging,
                basedir,
                per_host,
                hostname_sep,
            )?;
        }
    }
    Ok(())
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 23 2021, 00:05 [CST]
