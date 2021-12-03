use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::error::{Error as AppError, Result};
use crate::{config::*, item::DTItem};

/// Parameters for controlling how an item is synced.
#[derive(Debug)]
struct SyncingParameters {
    /// Path to source item, assumed to exist, and is already host-specific
    /// if possible.
    spath: PathBuf,
    /// Path to the parent dir of the disired sync destination.
    tparent: PathBuf,
    /// Whether to issue a dry run.
    dry: bool,
    /// Whether overwrite existing files or not.
    allow_overwrite: bool,
    /// A [`SyncMethod`] instance.
    ///
    /// [`SyncMethod`]: crate::config::SyncMethod
    method: SyncMethod,
    /// Path to staging directory.
    staging: PathBuf,
    /// Name of current group.
    group_name: String,
    /// Configured [`basedir`] of current group
    ///
    /// [`basedir`]: crate::config::LocalGroup::basedir
    basedir: PathBuf,
    /// Separator for per-host settings.
    hostname_sep: String,
}

/// Expands tilde and globs in [`sources`], returns new config object.
///
/// It does the following operations on given config:
///
/// 1. Convert [`global.staging`] to an absolute path.
/// 2. Convert all [`basedir`]s, [`target`]s in `[[local]]` to absolute paths.
/// 3. Replace [`basedir`]s and paths in [`sources`] with their host-specific
///    counterpart, if there exists any.
/// 4. Recursively expand globs and directories in [`sources`].
///
/// [`sources`]: crate::config::LocalGroup::sources
/// [`global.staging`]: crate::config::GlobalConfig::staging
/// [`basedir`]: crate::config::LocalGroup::basedir
/// [`target`]: crate::config::LocalGroup::target
/// [`[[local]]`]: crate::config::LocalGroup
fn expand(config: &DTConfig) -> Result<DTConfig> {
    let mut ret = DTConfig {
        global: match &config.global {
            Some(global) => Some(GlobalConfig {
                staging: match &global.staging {
                    Some(staging) => Some(staging.absolute()?),
                    None => GlobalConfig::default().staging,
                },
                ..global.to_owned()
            }),
            None => Some(GlobalConfig::default()),
        },
        local: Vec::new(),
    };
    for original in &config.local {
        let mut next = LocalGroup {
            basedir: original.basedir.absolute()?,
            sources: Vec::new(),
            target: original.target.absolute()?,
            ..original.to_owned()
        };

        let group_hostname_sep =
            original.get_hostname_sep(ret.global.as_ref().unwrap_or_else(|| {
                unreachable!("The [global] object is already a `Some`, this message should never be seen");
            }));

        // Check for host-specific `basedir`
        let host_specific_basedir =
            next.basedir.host_specific(&group_hostname_sep);
        if host_specific_basedir.exists() {
            next.basedir = host_specific_basedir;
        }

        // Above process does not guarantee the `basedir` to exist, since a
        // warning will be emitted later in the expanding process (see
        // function `expand_recursive()`), just don't attempt to read
        // non-existent `basedir` here by first checking its
        // existence.
        if next.basedir.exists() {
            // Check read permission of `basedir`
            if let Err(e) = std::fs::read_dir(&next.basedir) {
                log::error!(
                    "Could not read basedir '{}'",
                    next.basedir.display(),
                );
                return Err(e.into());
            }
        }

        // Check for host-specific `sources`
        let sources: Vec<PathBuf> = original
            .sources
            .iter()
            .map(|s| {
                let try_s = next
                    .basedir
                    .join(s)
                    .absolute()
                    .unwrap_or_else(|e| panic!("{}", e));
                let try_s = try_s.host_specific(&group_hostname_sep);
                if try_s.exists() {
                    try_s
                } else {
                    s.to_owned()
                }
            })
            .collect();

        // Recursively expand source paths
        for s in &sources {
            let s = next.basedir.join(s);
            let mut s = expand_recursive(
                &s,
                &next.get_hostname_sep(
                    &config.global.to_owned().unwrap_or_default(),
                ),
                true,
            )?;
            next.sources.append(&mut s);
        }
        next.sources.sort();
        next.sources.dedup();
        ret.local.push(next);
    }

    let ret = resolve(ret)?;

    check(&ret)?;

    Ok(ret)
}

/// Recursively expands a given path.
///
/// - If `do_glob` is `true`, trys to expand glob;
/// - If `do_glob` is `false`, `path` must be a directory, then children of
///   `path` are recursively expanded.
///
/// Returns a [`Vec`] of the expanded paths.
///
/// [`Vec`]: Vec
fn expand_recursive(
    path: &Path,
    hostname_sep: &str,
    do_glob: bool,
) -> Result<Vec<PathBuf>> {
    if do_glob {
        let globbing_options = glob::MatchOptions {
            case_sensitive: true,
            require_literal_separator: true,
            require_literal_leading_dot: true,
        };

        let initial: Vec<PathBuf> =
            glob::glob_with(path.to_str().unwrap(), globbing_options)?
                // Extract value from Result<PathBuf>
                .map(|x| {
                    x.unwrap_or_else(|_| {
                        panic!(
                            "Failed globbing source path '{}'",
                            path.display(),
                        )
                    })
                })
                // Filter out paths that are meant for other hosts
                .filter(|x| !x.is_for_other_host(hostname_sep))
                // Convert to absolute paths
                .map(|x| {
                    x.absolute().unwrap_or_else(|_| {
                        panic!(
                            "Failed converting to absolute path '{}'",
                            x.display(),
                        )
                    })
                })
                .collect();
        if initial.is_empty() {
            log::warn!("'{}' did not match anything", path.display());
        }

        let mut ret: Vec<PathBuf> = Vec::new();
        for p in initial {
            if p.is_file() {
                ret.push(p);
            } else if p.is_dir() {
                ret.append(&mut expand_recursive(&p, hostname_sep, false)?);
            } else {
                log::warn!(
                    "Skipping unimplemented file type at '{}'",
                    p.display(),
                );
                log::trace!("{:#?}", p.symlink_metadata()?);
            }
        }

        Ok(ret)
    } else {
        let initial: Vec<PathBuf> = std::fs::read_dir(path)?
            .map(|x| {
                x.unwrap_or_else(|_| {
                    panic!("Cannot read dir properly '{}'", path.display())
                })
                .path()
            })
            .filter(|x| !x.is_for_other_host(hostname_sep))
            .collect();

        let mut ret: Vec<PathBuf> = Vec::new();
        for p in initial {
            if p.is_file() {
                ret.push(p);
            } else if p.is_dir() {
                ret.append(&mut expand_recursive(&p, hostname_sep, false)?);
            } else {
                log::warn!(
                    "Skipping unimplemented file type at '{}'",
                    p.display(),
                );
                log::trace!("{:#?}", p.symlink_metadata()?);
            }
        }

        Ok(ret)
    }
}

/// Resolve priorities within expanded ret, this function is called before
/// [`check`] because it does not have to query the filesystem.
///
/// [`check`]: check
fn resolve(config: DTConfig) -> Result<DTConfig> {
    // Maps an item to the index of the group which holds the highest priority
    // of it.
    let mut mapping: HashMap<PathBuf, usize> = HashMap::new();

    // Get each item's highest priority group.
    for i in 0..config.local.len() {
        let current_priority =
            config.local[i].scope.as_ref().unwrap_or_default();
        for ref mut s in &config.local[i].sources {
            let t = s.make_target(
                &config.local[i].get_hostname_sep(
                    &config.global.to_owned().unwrap_or_default(),
                ),
                &config.local[i].basedir,
                &config.local[i].target,
            )?;
            match mapping.get(&t) {
                Some(prev_group_idx) => {
                    let prev_priority = config.local[*prev_group_idx]
                        .scope
                        .as_ref()
                        .unwrap_or_default();
                    // Only replace group index when current group has
                    // strictly higher priority than previous group, thus
                    // achieving "former defined groups of the same scope have
                    // higher priority" effect.
                    if current_priority > prev_priority {
                        mapping.insert(t, i);
                    }
                }
                None => {
                    mapping.insert(t, i);
                }
            }
        }
    }

    // Remove redundant groups.
    Ok(DTConfig {
        local: config
            .local
            .iter()
            .enumerate()
            .map(|(cur_id, group)| LocalGroup {
                sources: group
                    .sources
                    .iter()
                    .filter(|s| {
                        let t = s
                            .make_target(
                                &group.get_hostname_sep(
                                    &config
                                        .global
                                        .to_owned()
                                        .unwrap_or_default(),
                                ),
                                &group.basedir,
                                &group.target,
                            )
                            .unwrap();
                        let best_id = *mapping.get(&t).unwrap();
                        best_id == cur_id
                    })
                    .map(|s| s.to_owned())
                    .collect(),
                ..group.to_owned()
            })
            .collect(),
        ..config
    })
}

/// Checks validity of the given `config`.
fn check(config: &DTConfig) -> Result<()> {
    let mut has_symlink: bool = false;

    for group in &config.local {
        if !has_symlink
            && group.get_method(&config.global.to_owned().unwrap_or_default())
                == SyncMethod::Symlink
        {
            has_symlink = true;
        }

        // Wrong type of existing target path
        if group.target.exists() && !group.target.is_dir() {
            return Err(AppError::ConfigError(format!(
                "target path exists but is not a valid directory in group '{}'",
                group.name,
            )));
        }

        // Path to target contains readonly parent directory
        if group.target.parent_readonly() {
            return Err(AppError::ConfigError(format!(
                "target path cannot be created due to insufficient permissions in group '{}'",
                group.name,
            )));
        }

        for s in &group.sources {
            // Ignore cargo-clippy warnings here, since using
            // ```rust
            // std::fs::File::open(s)?;
            // ```
            // here will result in an IoError, while in this context, a
            // ConfigError should be thrown.
            #[allow(clippy::question_mark)]
            if std::fs::File::open(s).is_err() {
                return Err(AppError::ConfigError(format!(
                    "there exists one or more source item(s) that is not readable in group '{}'",
                    group.name,
                )));
            }
            if !s.is_file() {
                unreachable!();
            }
        }
    }

    if has_symlink {
        let staging = if config.global.is_none()
            || config.global.as_ref().unwrap().staging.is_none()
        {
            GlobalConfig::default().staging.unwrap()
        } else {
            config
                .global
                .as_ref()
                .unwrap()
                .staging
                .as_ref()
                .unwrap()
                .to_owned()
        };
        // Wrong type of existing staging path
        if staging.exists() && !staging.is_dir() {
            return Err(AppError::ConfigError(
                "staging root path exists but is not a valid directory"
                    .to_owned(),
            ));
        }

        // Path to staging root contains readonly parent directory
        if staging.parent_readonly() {
            return Err(AppError::ConfigError(
                "staging root path cannot be created due to insufficient permissions"
                    .to_owned(),
            ));
        }
    }

    Ok(())
}

/// Syncs items specified with given configuration object.
pub fn sync(config: DTConfig) -> Result<()> {
    if config.local.is_empty() {
        log::warn!("Nothing to be synced");
        return Ok(());
    }
    log::trace!("Local groups to process: {:#?}", config.local);

    let config = expand(&config)?;

    let staging = &config
        .global
        .to_owned()
        .unwrap_or_default()
        .staging
        .unwrap_or_else(|| GlobalConfig::default().staging.unwrap());
    if !staging.exists() {
        log::trace!(
            "Creating non-existing staging root '{}'",
            staging.display(),
        );
        std::fs::create_dir_all(staging)?;
    }

    for group in config.local {
        log::info!("Local group: [{}]", group.name);
        if group.sources.is_empty() {
            log::debug!(
                "Group [{}]: skipping due to empty group",
                group.name,
            );
            continue;
        } else {
            log::debug!(
                "Group [{}]: {} {} detected",
                group.name,
                group.sources.len(),
                if group.sources.len() <= 1 {
                    "item"
                } else {
                    "items"
                },
            );
        }

        let group_staging =
            staging.join(PathBuf::from_str(&group.name).unwrap());
        if !group_staging.exists() {
            log::trace!(
                "Creating non-existing staging directory '{}'",
                group_staging.display(),
            );
            std::fs::create_dir_all(&group_staging)?;
        }
        for spath in &group.sources {
            let params = SyncingParameters {
                spath: spath.to_owned(),
                tparent: group.target.to_owned(),
                dry: false,
                allow_overwrite: group.get_allow_overwrite(
                    &config.global.to_owned().unwrap_or_default(),
                ),
                method: group.get_method(
                    &config.global.to_owned().unwrap_or_default(),
                ),
                staging: group_staging.to_owned(),
                group_name: group.name.to_owned(),
                basedir: group.basedir.to_owned(),
                hostname_sep: group.get_hostname_sep(
                    &config.global.to_owned().unwrap_or_default(),
                ),
            };
            sync_core(params)?;
        }
    }
    Ok(())
}

/// Show changes to be made according to configuration, without actually
/// syncing items.
pub fn dry_sync(config: DTConfig) -> Result<()> {
    if config.local.is_empty() {
        log::warn!("Nothing to be synced");
        return Ok(());
    }
    log::trace!("Local groups to process: {:#?}", config.local);

    let config = expand(&config)?;

    let staging = &config
        .global
        .to_owned()
        .unwrap_or_default()
        .staging
        .unwrap_or_else(|| GlobalConfig::default().staging.unwrap());
    if !staging.exists() {
        log::info!("Staging root does not exist, will be automatically created when syncing");
    } else if !staging.is_dir() {
        log::error!("Staging root seems to exist and is not a directory");
    }

    for group in config.local {
        log::info!("Dry-running with local group: [{}]", group.name);
        if group.sources.is_empty() {
            log::debug!(
                "Group [{}]: skipping due to empty group",
                group.name,
            );
            continue;
        } else {
            log::debug!(
                "Group [{}]: {} {} detected",
                group.name,
                group.sources.len(),
                if group.sources.len() <= 1 {
                    "item"
                } else {
                    "items"
                },
            );
        }

        let group_staging =
            staging.join(PathBuf::from_str(&group.name).unwrap());
        if !group_staging.exists() {
            log::info!("Staging directory does not exist, will be automatically created when syncing");
        } else if !staging.is_dir() {
            log::error!(
                "Staging directory seems to exist and is not a directory"
            )
        }

        for spath in &group.sources {
            let params = SyncingParameters {
                spath: spath.to_owned(),
                tparent: group.target.to_owned(),
                dry: true,
                allow_overwrite: group.get_allow_overwrite(
                    &config.global.to_owned().unwrap_or_default(),
                ),
                method: group.get_method(
                    &config.global.to_owned().unwrap_or_default(),
                ),
                staging: group_staging.to_owned(),
                group_name: group.name.to_owned(),
                basedir: group.basedir.to_owned(),
                hostname_sep: group.get_hostname_sep(
                    &config.global.to_owned().unwrap_or_default(),
                ),
            };
            sync_core(params)?;
        }
    }
    Ok(())
}

/// Syncs `spath` to a directory `tparent`, being aware of its base directory.
fn sync_core(params: SyncingParameters) -> Result<()> {
    log::trace!("Parameters for `sync_core`: {:#?}", params);
    let SyncingParameters {
        spath,
        tparent,
        dry,
        allow_overwrite,
        method,
        staging,
        group_name,
        basedir,
        hostname_sep,
    } = params;
    if !tparent.exists() {
        if dry {
            log::warn!(
                "DRYRUN [{}]> Non-existing target directory '{}'",
                group_name,
                tparent.display(),
            );
        } else {
            log::trace!(
                "SYNC::CREATE [{}]> '{}'",
                group_name,
                tparent.display()
            );
            std::fs::create_dir_all(&tparent)?;
        }
    }

    // First, get target path (without the per-host suffix).
    let tpath = spath.make_target(&hostname_sep, &basedir, &tparent)?;
    if !dry {
        std::fs::create_dir_all(tpath.parent().unwrap_or_else(|| {
            panic!(
                "Structrue of target directory could not be created at '{}'",
                tpath.display(),
            )
        }))?;
    }

    // Finally, get the staging path with source path (staging path does not
    // have host-specific suffix).
    let staging_path =
        spath.make_target(&hostname_sep, &basedir, &staging)?;
    if !dry && method == SyncMethod::Symlink {
        std::fs::create_dir_all(staging_path.parent().unwrap_or_else(
            || {
                panic!(
            "Structrue of staging directory could not be created at '{}'",
            staging_path.display(),
        )
            },
        ))?;
    }

    if spath.is_file() {
        if tpath.is_dir() {
            if dry {
                log::error!(
                    "DRYRUN [{}]> A directory ('{}') exists at the target path of a source file ('{}')",
                    group_name,
                    tpath.display(),
                    spath.display(),
                );
            } else {
                return Err(
                    AppError::SyncingError(format!(
                        "a directory '{}' exists at the target path of a source file '{}'",
                        tpath.display(),
                        spath.display(),
                    ))
                );
            };
        }

        if dry {
            if tpath.exists() {
                if allow_overwrite {
                    log::debug!(
                        "DRYRUN::OVERWRITE [{}]> '{}' -> '{}'",
                        group_name,
                        spath.display(),
                        tpath.display(),
                    );
                } else {
                    log::error!(
                        "DRYRUN [{}]> Target path ('{}') exists",
                        group_name,
                        tpath.display(),
                    );
                }
            } else {
                log::debug!(
                    "DRYRUN [{}]> '{}' -> '{}'",
                    group_name,
                    spath.display(),
                    tpath.display(),
                );
            }
        } else if tpath.exists() && !allow_overwrite {
            log::warn!(
                "SYNC::SKIP [{}]> Target path ('{}') exists",
                group_name,
                tpath.display(),
            );
        } else {
            // In this block, either:
            //
            //  - `tpath` does not exist
            //  - `allow_overwrite` is true
            //
            // or both are true.
            if method == SyncMethod::Copy {
                // Treat `tpath` as a symlink if `std::fs::read_link` returns
                // Ok on `tpath`.
                // Note: `Path::is_symlink()` has been stablized
                // (rust-lang/rust#89677), might be a good idea to switch to
                // it as soon as it enters stable Rust..
                if std::fs::read_link(&tpath).is_ok() {
                    // If `tpath` is a symlink, we want to delete the symlink
                    // because SyncMethod is Copy, if we don't remove it
                    // first, we will write to the symlink's destination
                    // instead of our desired target path.
                    log::trace!(
                        "SYNC::COPY [{}]> '{}' is a symlink, removing it",
                        group_name,
                        tpath.display(),
                    );
                    std::fs::remove_file(&tpath)?;
                }
                if let Ok(dest_content) = std::fs::read(&tpath) {
                    let src_content = std::fs::read(&spath)?;
                    if src_content == dest_content {
                        log::trace!(
                            "SYNC::COPY::SKIP [{}]> '{}' has identical content as '{}'",
                            group_name,
                            staging_path.display(),
                            spath.display(),
                        );
                    } else {
                        if let Err(_) = std::fs::copy(&spath, &tpath) {
                            log::warn!(
                                "SYNC::COPY::OVERWRITE [{}]> '{}' seems to be readonly, trying to remove it first ..",
                                group_name,
                                staging_path.display(),
                            );
                            std::fs::remove_file(&staging_path)?;
                            log::trace!(
                                "SYNC::COPY::OVERWRITE [{}]> '{}' => '{}'",
                                group_name,
                                spath.display(),
                                tpath.display(),
                            );
                            std::fs::copy(spath, &staging_path)?;
                        }
                    }
                } else {
                    if tpath.exists() {
                        log::warn!(
                            "SYNC::COPY::OVERWRITE [{}]> Could not read content of target file ('{}'), trying to remove it first ..",
                            group_name,
                            tpath.display(),
                        );
                        std::fs::remove_file(&tpath)?;
                        log::trace!(
                            "SYNC::COPY::OVERWRITE [{}]> '{}' => '{}'",
                            group_name,
                            spath.display(),
                            tpath.display(),
                        );
                        std::fs::copy(spath, &tpath)?;
                    } else {
                        log::trace!(
                            "SYNC::COPY [{}]> '{}' => '{}'",
                            group_name,
                            spath.display(),
                            tpath.display(),
                        );
                        std::fs::copy(spath, &tpath)?;
                    }
                }
            } else if method == SyncMethod::Symlink {
                // Staging
                // Check if the content of destination is already the same as
                // source first. This operation is significantly faster than
                // copying to an existing target file when the file is large.
                if let Ok(dest_content) = std::fs::read(&staging_path) {
                    let src_content = std::fs::read(&spath)?;
                    if src_content == dest_content {
                        log::trace!(
                            "SYNC::STAGE::SKIP [{}]> '{}' has identical content as '{}'",
                            group_name,
                            staging_path.display(),
                            spath.display(),
                        );
                    } else {
                        if let Err(_) = std::fs::copy(&spath, &staging_path) {
                            log::warn!(
                                "SYNC::STAGE::OVERWRITE [{}]> '{}' seems to be readonly, trying to remove it first ..",
                                group_name,
                                staging_path.display(),
                            );
                            std::fs::remove_file(&staging_path)?;
                            log::trace!(
                                "SYNC::STAGE [{}]> '{}' => '{}'",
                                group_name,
                                spath.display(),
                                staging_path.display(),
                            );
                            std::fs::copy(spath, &staging_path)?;
                        }
                    }
                } else {
                    log::warn!(
                        "SYNC::COPY::OVERWRITE [{}]> Could not read content of staging file ('{}'), trying to remove it first ..",
                        group_name,
                        staging_path.display(),
                    );
                    if staging_path.exists() {
                        std::fs::remove_file(&staging_path)?;
                    }
                    log::trace!(
                        "SYNC::STAGE [{}]> '{}' => '{}'",
                        group_name,
                        spath.display(),
                        staging_path.display(),
                    );
                    std::fs::copy(spath, &staging_path)?;
                }

                // Symlinking
                // Do not remove target file if it is already a symlink that
                // points to the correct location.
                if let Ok(dest) = std::fs::read_link(&tpath) {
                    if dest == staging_path {
                        log::trace!(
                            "SYNC::SYMLINK::SKIP [{}]> '{}' is already a symlink pointing to '{}'",
                            group_name,
                            tpath.display(),
                            staging_path.display(),
                        );
                    } else {
                        log::trace!(
                            "SYNC::SYMLINK::OVERWRITE [{}]> '{}' => '{}'",
                            group_name,
                            staging_path.display(),
                            tpath.display(),
                        );
                        std::fs::remove_file(&tpath)?;
                        std::os::unix::fs::symlink(&staging_path, &tpath)?;
                    }
                }
                // If target file exists but is not a symlink, try to remove
                // it first, then also make a symlink from `staging_path` to
                // `tpath`.
                else if tpath.exists() {
                    log::trace!(
                        "SYNC::SYMLINK::OVERWRITE [{}]> '{}' => '{}'",
                        group_name,
                        staging_path.display(),
                        tpath.display(),
                    );
                    std::fs::remove_file(&tpath)?;
                    std::os::unix::fs::symlink(&staging_path, &tpath)?;
                }
                // The final case is that when `tpath` does not exist yet, we
                // can then directly create a symlink.
                else {
                    log::trace!(
                        "SYNC::SYMLINK [{}]> '{}' => '{}'",
                        group_name,
                        staging_path.display(),
                        tpath.display(),
                    );
                    std::os::unix::fs::symlink(&staging_path, &tpath)?;
                }
            }
        }
    } else if spath.is_dir() {
        if tpath.is_file() {
            if dry {
                log::error!(
                    "DRYRUN [{}]> A file ('{}') exists at the target path of a source directory ('{}')",
                    group_name,
                    tpath.display(),
                    spath.display(),
                );
            } else {
                return Err(
                    AppError::SyncingError(format!(
                        "a file '{}' exists at the target path of a source directory '{}'",
                        tpath.display(),
                        spath.display(),
                    ))
                );
            };
        }

        // `staging_path` and `tpath` should be directories at this point, if
        // they exist.
        if method == SyncMethod::Symlink && !staging_path.exists()
            || !tpath.exists()
        {
            if dry {
                log::warn!(
                    "DRYRUN [{}]> Non-existing directory: '{}'",
                    group_name,
                    if !tpath.exists() {
                        tpath.display()
                    } else {
                        staging_path.display()
                    },
                );
            } else {
                if method == SyncMethod::Symlink && !staging_path.exists() {
                    log::trace!(
                        "SYNC::STAGE::CREATE [{}]> '{}'",
                        group_name,
                        staging_path.display(),
                    );
                    std::fs::create_dir_all(staging_path)?;
                }
                if !tpath.exists() {
                    log::trace!(
                        "SYNC::CREATE [{}]> '{}'",
                        group_name,
                        tpath.display(),
                    );
                    std::fs::create_dir_all(&tpath)?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod invalid_configs {
    use std::{
        os::unix::prelude::PermissionsExt, path::PathBuf, str::FromStr,
    };

    use color_eyre::{eyre::eyre, Report};
    use pretty_assertions::assert_eq;

    use crate::config::DTConfig;
    use crate::error::Error as AppError;

    use super::expand;

    #[test]
    fn basedir_unreadable() -> Result<(), Report> {
        if let Err(err) = expand(&DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-basedir_unreadable-not_a_directory.toml",
        ).unwrap())?) {
            assert_eq!(
                err,
                AppError::IoError(
                    "Not a directory (os error 20)".to_owned(),
                ),
                "{}",
                err,
            );
        } else {
            return Err(eyre!(
                "This config should not be loaded because basedir is not a directory",
            ));
        }

        std::fs::set_permissions(
            PathBuf::from_str(
                "../testroot/items/syncing/invalid_configs/basedir_unreadable/basedir"
            ).unwrap(),
            std::fs::Permissions::from_mode(0o333),
        )?;
        if let Err(err) = expand(&DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-basedir_unreadable-permission_denied.toml",
        ).unwrap())?) {
            assert_eq!(
                err,
                AppError::IoError(
                    "Permission denied (os error 13)"
                        .to_owned(),
                ),
                "{}",
                err,
            );
            std::fs::set_permissions(
                PathBuf::from_str(
                    "../testroot/items/syncing/invalid_configs/basedir_unreadable/basedir"
                ).unwrap(),
                std::fs::Permissions::from_mode(0o755),
            )?;
        } else {
            std::fs::set_permissions(
                PathBuf::from_str(
                    "../testroot/items/syncing/invalid_configs/basedir_unreadable/basedir"
                ).unwrap(),
                std::fs::Permissions::from_mode(0o755),
            )?;
            return Err(eyre!(
                "This config should not be loaded because insufficient permissions to basedir",
            ));
        }

        Ok(())
    }

    #[test]
    fn target_is_file_relative() -> Result<(), Report> {
        if let Err(err) = expand(&DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-target_is_file_relative.toml",
        ).unwrap())?) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "target path exists but is not a valid directory in group 'target path is relative'"
                        .to_owned(),
                ),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!(
                "This config should not be loaded because target is not a directory",
            ))
        }
    }

    #[test]
    fn target_is_file_absolute() -> Result<(), Report> {
        if let Err(err) = expand(&DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-target_is_file_absolute.toml",
        ).unwrap())?) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "target path exists but is not a valid directory in group 'target path is absolute'"
                        .to_owned(),
                ),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!(
                "This config should not be loaded because target is not a directory",
            ))
        }
    }

    #[test]
    fn target_is_file_has_tilde() -> Result<(), Report> {
        // setup
        let filepath = dirs::home_dir()
            .unwrap()
            .join("d6a8e0bc1647c38548432ccfa1d79355");
        assert!(
            !filepath.exists(),
            "A previous test seems to have aborted abnormally, remove the file '$HOME/d6a8e0bc1647c38548432ccfa1d79355' to continue testing",
        );
        std::fs::write(&filepath, "Created by `dt` when testing")?;

        // Read config (expected to fail)
        if let Err(err) = expand(&DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-target_is_file_has_tilde.toml",
        ).unwrap())?) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "target path exists but is not a valid directory in group 'target contains tilde to be expanded'"
                        .to_owned(),
                ),
                "{}",
                err,
            );
            // clean up
            std::fs::remove_file(filepath)?;
        } else {
            // clean up
            std::fs::remove_file(filepath)?;
            return Err(eyre!(
                "This config should not be loaded because target is not a directory",
            ));
        }

        Ok(())
    }

    #[test]
    fn target_readonly() -> Result<(), Report> {
        std::fs::set_permissions(
            PathBuf::from_str(
                "../testroot/items/syncing/invalid_configs/target_readonly/target"
            ).unwrap(),
            std::fs::Permissions::from_mode(0o555),
        )?;
        if let Err(err) = expand(&DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-target_readonly.toml",
        ).unwrap())?) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "target path cannot be created due to insufficient permissions in group 'target is readonly'"
                        .to_owned(),
                ),
                "{}",
                err,
            );
            std::fs::set_permissions(
                PathBuf::from_str(
                    "../testroot/items/syncing/invalid_configs/target_readonly/target"
                ).unwrap(),
                std::fs::Permissions::from_mode(0o755),
            )?;
            Ok(())
        } else {
            std::fs::set_permissions(
                PathBuf::from_str(
                    "../testroot/items/syncing/invalid_configs/target_readonly/target"
                ).unwrap(),
                std::fs::Permissions::from_mode(0o755),
            )?;
            Err(eyre!(
                "This config should not be loaded because target path is readonly",
            ))
        }
    }

    #[test]
    fn staging_is_file() -> Result<(), Report> {
        if let Err(err) = expand(&DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-staging_is_file.toml",
        ).unwrap())?) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "staging root path exists but is not a valid directory"
                        .to_owned(),
                ),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!(
                "This config should not be loaded because target path is readonly",
            ))
        }
    }

    #[test]
    fn staging_readonly() -> Result<(), Report> {
        std::fs::set_permissions(
            PathBuf::from_str(
                "../testroot/items/syncing/invalid_configs/staging_readonly/staging"
            ).unwrap(),
            std::fs::Permissions::from_mode(0o555),
        )?;
        if let Err(err) = expand(&DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-staging_readonly.toml",
        ).unwrap())?) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "staging root path cannot be created due to insufficient permissions"
                        .to_owned(),
                ),
                "{}",
                err,
            );
            std::fs::set_permissions(
                PathBuf::from_str(
                    "../testroot/items/syncing/invalid_configs/staging_readonly/staging"
                ).unwrap(),
                std::fs::Permissions::from_mode(0o755),
            )?;
            Ok(())
        } else {
            std::fs::set_permissions(
                PathBuf::from_str(
                    "../testroot/items/syncing/invalid_configs/staging_readonly/staging"
                ).unwrap(),
                std::fs::Permissions::from_mode(0o755),
            )?;
            Err(eyre!(
                "This config should not be loaded because staging path is readonly",
            ))
        }
    }

    #[test]
    fn unreadable_source() -> Result<(), Report> {
        std::fs::set_permissions(
            "../testroot/items/syncing/invalid_configs/unreadable_source/no_read_access",
            std::fs::Permissions::from_mode(0o222)
        )?;
        if let Err(err) = expand(&DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-unreadable_source.toml",
        ).unwrap())?) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "there exists one or more source item(s) that is not readable in group 'source is unreadable'"
                        .to_owned(),
                ),
                "{}",
                err,
            );
            std::fs::set_permissions(
                "../testroot/items/syncing/invalid_configs/unreadable_source/no_read_access",
                std::fs::Permissions::from_mode(0o644)
            )?;
            Ok(())
        } else {
            std::fs::set_permissions(
                "../testroot/items/syncing/invalid_configs/unreadable_source/no_read_access",
                std::fs::Permissions::from_mode(0o644)
            )?;
            Err(eyre!("This config should not be loaded because source item is not readable"))
        }
    }
}

#[cfg(test)]
mod expansion {
    use std::{path::PathBuf, str::FromStr};

    use color_eyre::Report;
    use pretty_assertions::assert_eq;

    use crate::{config::*, item::DTItem};

    use super::expand;

    #[test]
    fn glob() -> Result<(), Report> {
        let config = expand(&DTConfig::from_path(
            PathBuf::from_str(
                "../testroot/configs/syncing/expansion-glob.toml",
            )
            .unwrap(),
        )?)?;
        for group in &config.local {
            assert_eq!(
                group.sources,
                vec![
                    PathBuf::from_str("../dt-cli/Cargo.toml")
                        .unwrap()
                        .absolute()?,
                    PathBuf::from_str("../dt-cli/README.md")
                        .unwrap()
                        .absolute()?,
                    PathBuf::from_str("../dt-cli/src/main.rs")
                        .unwrap()
                        .absolute()?,
                    PathBuf::from_str("../dt-core/Cargo.toml")
                        .unwrap()
                        .absolute()?,
                    PathBuf::from_str("../dt-core/README.md")
                        .unwrap()
                        .absolute()?,
                    PathBuf::from_str("../dt-core/src/config.rs")
                        .unwrap()
                        .absolute()?,
                    PathBuf::from_str("../dt-core/src/error.rs")
                        .unwrap()
                        .absolute()?,
                    PathBuf::from_str("../dt-core/src/item.rs")
                        .unwrap()
                        .absolute()?,
                    PathBuf::from_str("../dt-core/src/lib.rs")
                        .unwrap()
                        .absolute()?,
                    PathBuf::from_str("../dt-core/src/syncing.rs")
                        .unwrap()
                        .absolute()?,
                    PathBuf::from_str("../dt-core/src/utils.rs")
                        .unwrap()
                        .absolute()?,
                ],
            );
        }
        Ok(())
    }

    #[test]
    fn sorting_and_deduping() -> Result<(), Report> {
        let config = expand(&DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/syncing/expansion-sorting_and_deduping.toml",
        ).unwrap())?)?;
        for group in config.local {
            assert_eq!(
                group.sources,
                vec![
                    PathBuf::from_str(
                        "../testroot/items/sorting_and_deduping/A-a"
                    )
                    .unwrap()
                    .absolute()?,
                    PathBuf::from_str(
                        "../testroot/items/sorting_and_deduping/A-b"
                    )
                    .unwrap()
                    .absolute()?,
                    PathBuf::from_str(
                        "../testroot/items/sorting_and_deduping/A-c"
                    )
                    .unwrap()
                    .absolute()?,
                    PathBuf::from_str(
                        "../testroot/items/sorting_and_deduping/B-a"
                    )
                    .unwrap()
                    .absolute()?,
                    PathBuf::from_str(
                        "../testroot/items/sorting_and_deduping/B-b"
                    )
                    .unwrap()
                    .absolute()?,
                    PathBuf::from_str(
                        "../testroot/items/sorting_and_deduping/B-c"
                    )
                    .unwrap()
                    .absolute()?,
                ],
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod priority_resolving {
    use std::str::FromStr;

    use crate::{config::*, error::*, syncing::expand};

    #[test]
    fn proper_priority_orders() -> Result<()> {
        assert!(DTScope::Dropin > DTScope::App);
        assert!(DTScope::App > DTScope::General);
        assert!(DTScope::Dropin > DTScope::General);

        assert!(DTScope::App < DTScope::Dropin);
        assert!(DTScope::General < DTScope::App);
        assert!(DTScope::General < DTScope::Dropin);

        Ok(())
    }

    #[test]
    fn former_group_has_higher_priority_within_same_scope() -> Result<()> {
        let config = expand(&DTConfig::from_str(
            r#"
                [[local]]
                name = "highest"
                # Scope is omitted to use default scope (i.e. General)
                basedir = "../dt-cli"
                sources = ["Cargo.toml"]
                target = "."
                [[local]]
                name = "low"
                # Scope is omitted to use default scope (i.e. General)
                basedir = "../dt-server"
                sources = ["Cargo.toml"]
                target = "."
        "#,
        )?)?;

        assert!(!config.local[0].sources.is_empty());
        assert!(config.local[1].sources.is_empty());

        Ok(())
    }

    #[test]
    fn dropin_has_highest_priority() -> Result<()> {
        let config = expand(&DTConfig::from_str(
            r#"
                [[local]]
                name = "lowest"
                scope = "General"
                basedir = "../dt-cli"
                sources = ["Cargo.toml"]
                target = "."
                [[local]]
                name = "medium"
                scope = "App"
                basedir = "../dt-server"
                sources = ["Cargo.toml"]
                target = "."
                [[local]]
                name = "highest"
                scope = "Dropin"
                basedir = ".."
                sources = ["Cargo.toml"]
                target = "."
            "#,
        )?)?;

        assert!(config.local[0].sources.is_empty());
        assert!(config.local[1].sources.is_empty());
        assert!(!config.local[2].sources.is_empty());

        Ok(())
    }

    #[test]
    fn app_has_medium_priority() -> Result<()> {
        let config = expand(&DTConfig::from_str(
            r#"
                [[local]]
                name = "lowest"
                scope = "General"
                basedir = "../dt-cli"
                sources = ["Cargo.toml"]
                target = "."
                [[local]]
                name = "medium"
                scope = "App"
                basedir = "../dt-server"
                sources = ["Cargo.toml"]
                target = "."
            "#,
        )?)?;

        assert!(config.local[0].sources.is_empty());
        assert!(!config.local[1].sources.is_empty());

        Ok(())
    }

    #[test]
    fn default_scope_is_general() -> Result<()> {
        let config = expand(&DTConfig::from_str(
            r#"
                [[local]]
                name = "omitted scope but defined first, has higher priority"
                # Scope is omitted to use default scope (i.e. General)
                basedir = "../dt-cli"
                sources = ["Cargo.toml"]
                target = "."
                [[local]]
                name = "specified scope but defined last, has lower priority"
                scope = "General"
                basedir = "../dt-server"
                sources = ["Cargo.toml"]
                target = "."
            "#,
        )?)?;

        assert!(!config.local[0].sources.is_empty());
        assert!(config.local[1].sources.is_empty());

        let config = expand(&DTConfig::from_str(
            r#"
                [[local]]
                name = "omitted scope, uses general"
                # Scope is omitted to use default scope (i.e. General)
                basedir = ".."
                sources = ["Cargo.toml"]
                target = "."
                [[local]]
                name = "specified scope with higher priority"
                scope = "App"
                basedir = ".."
                sources = ["Cargo.toml"]
                target = "."
            "#,
        )?)?;

        assert!(config.local[0].sources.is_empty());
        assert!(!config.local[1].sources.is_empty());

        Ok(())
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 23 2021, 00:05 [CST]
