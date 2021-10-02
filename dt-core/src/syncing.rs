use std::path::PathBuf;

use color_eyre::{eyre::eyre, Report};

use crate::config::*;

/// Syncs items specified in configuration.
pub fn sync(config: &DTConfig) -> Result<(), Report> {
    for local in &config.local {
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
            )?;
        }
    }
    Ok(())
}

/// Show changes to be made according to configuration, without actually syncing items.
pub fn dry_sync(config: &DTConfig) -> Result<(), Report> {
    for local in &config.local {
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
fn sync_recursive(
    spath: &PathBuf,
    tparent: &PathBuf,
    dry: bool,
    allow_overwrite: bool,
    method: SyncMethod,
) -> Result<(), Report> {
    if method == SyncMethod::Symlink {
        todo!("Syncing with symlinks is not implemented");
    }

    if !tparent.exists() {
        if dry {
            log::info!(
                "DRYRUN: Stopping at non-existing target directory {:?}",
                tparent
            );
        } else {
            log::debug!("Creating target directory {:?}", tparent);
            std::fs::create_dir_all(tparent)?;
        }
    }
    let overwrite_log_level = if allow_overwrite {
        log::Level::Warn
    } else {
        log::Level::Error
    };

    let sname = spath.file_name().unwrap();
    let tpath = tparent.join(sname);
    if spath.is_file() {
        if tpath.is_dir() {
            return if dry {
                log::error!(
                    "DRYRUN: A directory ({:?}) exists at the target path of a source file ({:?})",
                    tpath,
                    spath,
                );
                Ok(())
            } else {
                Err(
                    eyre!(
                        "A directory ({:?}) exists at the target path of a source file ({:?})",
                        tpath,
                        spath,
                    )
                )
            };
        }

        if dry {
            if tpath.exists() {
                log::log!(
                    overwrite_log_level,
                    "DRYRUN: Target path ({:?}) exists",
                    &tpath
                );
            }
            log::info!("DRYRUN: {:?} -> {:?}", &spath, &tpath);
        } else {
            if tpath.exists() && !allow_overwrite {
                log::log!(
                    overwrite_log_level,
                    "SKIPPING: Target path ({:?}) exists",
                    &tpath
                );
            } else {
                log::debug!("SYNCING: {:?} => {:?}", spath, tpath);
                std::fs::copy(spath, tpath)?;
            }
        }
    } else if spath.is_dir() {
        if tpath.is_file() {
            return if dry {
                log::error!(
                    "DRYRUN: A file ({:?}) exists at the target path of a source directory ({:?})",
                    tpath,
                    spath,
                );
                Ok(())
            } else {
                Err(
                    eyre!(
                        "A file ({:?}) exists at the target path of a source directory ({:?})",
                        tpath,
                        spath,
                    )
                )
            };
        }

        if !tpath.exists() {
            if dry {
                log::info!(
                    "DRYRUN: Stopping recursion at non-existing directory {:?}",
                    &tpath
                );
                return Ok(());
            } else {
                log::debug!("CREATING: {:?}", &tpath);
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
            )?;
        }
    }
    Ok(())
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 23 2021, 00:05 [CST]
