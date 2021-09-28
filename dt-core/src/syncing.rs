use std::path::PathBuf;

use color_eyre::{eyre::eyre, Report};

use crate::config::DTConfig;

/// Syncs items specified in configuration.
pub fn sync(config: &DTConfig) -> Result<(), Report> {
    for local in &config.local {
        for spath in &local.sources {
            sync_recursive(spath, &local.target, false)?;
        }
    }
    Ok(())
}

/// Show changes to be made according to configuration, without actually syncing items.
pub fn dry_sync(config: &DTConfig) -> Result<(), Report> {
    for local in &config.local {
        for spath in &local.sources {
            sync_recursive(spath, &local.target, true)?;
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
fn sync_recursive(
    spath: &PathBuf,
    tparent: &PathBuf,
    dry: bool,
) -> Result<(), Report> {
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
                log::warn!("DRYRUN: Target path ({:?}) exists", &tpath);
            }
            log::info!("DRYRUN: {:?} -> {:?}", &spath, &tpath);
        } else {
            log::debug!("{:?} => {:?}", &spath, &tpath);
            std::fs::copy(spath, tpath)?;
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
                    "DRYRUN: Stopping recursion at non-existing directory '{:?}'",
                    &tpath
                );
                return Ok(());
            } else {
                log::debug!("Creating directory hierarchy: '{:?}'", &tpath);
                std::fs::create_dir_all(&tpath)?;
            }
        }

        for item in std::fs::read_dir(spath)? {
            let item = item?;
            sync_recursive(&item.path(), &tpath, dry)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod dry_run_behaviours {
    // #[test]
    // fn
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 23 2021, 00:05 [CST]
