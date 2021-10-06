use std::{ops::Not, path::PathBuf, str::FromStr};

use color_eyre::{eyre::eyre, Report};

use crate::{config::*, utils};

/// Expand tilde and globs in "sources" and manifest new config object.
fn expand(config: &DTConfig) -> Result<DTConfig, Report> {
    validate_pre_expansion(&config)?;

    let mut ret = DTConfig {
        global: match &config.global {
            Some(global) => Some(GlobalConfig {
                staging: match &global.staging {
                    Some(staging) => Some(utils::to_absolute(staging)?),
                    None => GlobalConfig::default().staging,
                },
                ..global.to_owned()
            }),
            None => Some(GlobalConfig::default()),
        },
        local: vec![],
    };
    for original in &config.local {
        let mut next = LocalSyncConfig {
            basedir: utils::to_absolute(&original.basedir)?,
            sources: vec![],
            target: utils::to_absolute(&original.target)?,
            ..original.to_owned()
        };

        // Check for host-specific basedir
        let host_specific_basedir = utils::to_host_specific(
            &next.basedir,
            &next
                .hostname_sep
                .to_owned()
                .unwrap_or(DEFAULT_HOSTNAME_SEPARATOR.to_owned()),
        )?;
        if host_specific_basedir.exists() {
            next.basedir = host_specific_basedir;
        }

        for s in &original.sources {
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

    validate_post_expansion(&ret)?;

    Ok(ret)
}

fn expand_recursive(
    path: &PathBuf,
    hostname_sep: &str,
    do_glob: bool,
) -> Result<Vec<PathBuf>, Report> {
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
                    x.expect(&format!(
                        "Failed globbing source path {}",
                        path.display(),
                    ))
                })
                // Filter out paths that are meant for other hosts
                .filter(|x| utils::is_for_other_host(x, hostname_sep).not())
                // Make items host-specific wherever possible
                .map(|x| {
                    let hspath = utils::to_host_specific(&x, hostname_sep)
                        .expect("Error getting host-specific item name");
                    if hspath.exists() {
                        hspath
                    } else {
                        x
                    }
                })
                // Convert to absolute paths
                .map(|x| {
                    utils::to_absolute(&x).expect(&format!(
                        "Failed converting to absolute path: {}",
                        x.display(),
                    ))
                })
                .collect();

        let mut ret: Vec<PathBuf> = Vec::new();
        for p in initial {
            assert!(p.exists());
            if p.is_file() {
                ret.push(p);
            } else if p.is_dir() {
                ret.append(&mut expand_recursive(&p, hostname_sep, false)?);
            } else {
                unimplemented!(
                    "Unimplemented file type.  Metadata: {:#?}",
                    p.metadata()?
                );
            }
        }

        Ok(ret)
    } else {
        let initial: Vec<PathBuf> = std::fs::read_dir(path)?
            .map(|x| {
                x.expect(&format!(
                    "Cannot read dir properly: {}",
                    path.display()
                ))
                .path()
            })
            .collect();

        let mut ret: Vec<PathBuf> = Vec::new();
        for p in initial {
            if p.is_file() {
                ret.push(p);
            } else if p.is_dir() {
                ret.append(&mut expand_recursive(&p, hostname_sep, false)?);
            } else {
                unimplemented!();
            }
        }

        Ok(ret)
    }
}

fn validate_pre_expansion(config: &DTConfig) -> Result<(), Report> {
    let mut group_name_rec: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    for group in &config.local {
        if let Some(_) = group_name_rec.get(&group.name) {
            return Err(eyre!("Duplicated local group name: {}", group.name));
        }
        group_name_rec.insert(group.name.to_owned());
        if group.ignored.is_some() {
            todo!("`ignored` array works poorly and I decided to implement it in the future");
        }
        // for i in &group.ignored { // This is WRONG because group.ignored is an Option<>
        // if i.contains(&"/".to_owned()) {
        // return Err(eyre!(
        // "Ignored pattern contains slash, this is not allowed"
        // ));
        // }
        // }
    }
    Ok(())
}

fn validate_post_expansion(config: &DTConfig) -> Result<(), Report> {
    for group in &config.local {
        if group.basedir.exists().not() {
            return Err(eyre!(
                "Group [{}]: base directory ({}) does not exist",
                group.name,
                group.basedir.display(),
            ));
        }

        if group.basedir == group.target {
            return Err(eyre!(
                "Group [{}]: base directory and its target are the same",
                group.name,
            ));
        }

        if group.target.exists() && !group.target.is_dir() {
            return Err(eyre!(
                "Group [{}]: target path exists and is not a directory",
                group.name,
            ));
        }

        if group.basedir.is_dir().not() {
            return Err(eyre!(
                "Configured basedir {} is invalid",
                group.basedir.display(),
            ));
        }
    }
    Ok(())
}

/// Syncs items specified in configuration.
pub fn sync(config: &DTConfig) -> Result<(), Report> {
    let config = expand(&config)?;

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

    for group in &config.local {
        let group_staging = staging.join(PathBuf::from_str(&group.name)?);
        if group_staging.exists().not() {
            log::debug!(
                "Creating non-existing staging directory {}",
                group_staging.display(),
            );
            std::fs::create_dir_all(&group_staging)?;
        }
        for spath in &group.sources {
            sync_core(
                spath,
                &group.target,
                false,
                group.get_allow_overwrite(
                    &config.global.to_owned().unwrap_or_default(),
                ),
                group.get_method(
                    &config.global.to_owned().unwrap_or_default(),
                ),
                &group_staging,
                &group.name,
                &group.basedir,
                &group.get_hostname_sep(
                    &config.global.to_owned().unwrap_or_default(),
                ),
            )?;
        }
    }
    Ok(())
}

/// Show changes to be made according to configuration, without actually syncing items.
pub fn dry_sync(config: &DTConfig) -> Result<(), Report> {
    let config = expand(config)?;

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

    for group in &config.local {
        let group_staging = staging.join(PathBuf::from_str(&group.name)?);
        if group_staging.exists().not() {
            log::info!("Staging directory does not exist, will be automatically created when syncing");
        } else if staging.is_dir().not() {
            log::info!(
                "Staging directory seems to exist and is not a directory"
            )
        }
        for spath in &group.sources {
            sync_core(
                spath,
                &group.target,
                true,
                group.get_allow_overwrite(
                    &config.global.to_owned().unwrap_or_default(),
                ),
                group.get_method(
                    &config.global.to_owned().unwrap_or_default(),
                ),
                &group_staging,
                &group.name,
                &group.basedir,
                &group.get_hostname_sep(
                    &config.global.to_owned().unwrap_or_default(),
                ),
            )?;
        }
    }
    Ok(())
}

/// Syncs `spath` to a directory `tparent`, being aware of its base directory.
///
/// Args:
///   - `spath`: Path to source item, assumed to exist, and is already host-specific if possible.
///   - `tparent`: Path to the parent dir of the disired sync destination.
///   - `dry`: Whether to issue a dry run.
///   - `allow_overwrite`: Whether overwrite existing files or not.
///   - `method`: A `SyncMethod` instance.
///   - `staging`: Path to staging directory.
///   - `hostname_sep`: Separator for per-host settings.
fn sync_core(
    spath: &PathBuf,
    tparent: &PathBuf,
    dry: bool,
    allow_overwrite: bool,
    method: SyncMethod,
    staging: &PathBuf,
    group_name: &str,
    basedir: &PathBuf,
    hostname_sep: &str,
) -> Result<(), Report> {
    if tparent.exists().not() {
        if dry {
            log::warn!(
                "DRYRUN [{}]> Non-existing target directory {}",
                group_name,
                tparent.display(),
            );
        } else {
            log::debug!(
                "SYNC::CREATE [{}]> {}",
                group_name,
                tparent.display()
            );
            std::fs::create_dir_all(tparent)?;
        }
    }
    let overwrite_log_level = if allow_overwrite {
        log::Level::Warn
    } else {
        log::Level::Error
    };

    // First, get target path (without the per-host suffix).
    let tpath = tparent.join(
        utils::to_non_host_specific(spath, hostname_sep)?
            .strip_prefix(basedir)?,
    );
    std::fs::create_dir_all(tpath.parent().expect(&format!(
        "Structrue of target directory could not be created at {}",
        tpath.display(),
    )))?;

    // Finally, get the staging path with source path (staging path does not have host-specific
    // suffix).
    let staging_path = staging.join(
        utils::to_non_host_specific(spath, hostname_sep)?
            .strip_prefix(basedir)?,
    );
    std::fs::create_dir_all(staging_path.parent().expect(&format!(
        "Structrue of staging directory could not be created at {}",
        staging_path.display(),
    )))?;

    if spath.is_file() {
        if tpath.is_dir() {
            if dry {
                log::error!(
                    "DRYRUN [{}]> A directory ({}) exists at the target path of a source file ({})",
                    group_name,
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
                    "DRYRUN [{}]> Target path ({}) exists",
                    group_name,
                    tpath.display(),
                );
            }
            log::info!(
                "DRYRUN [{}]> {} -> {}",
                group_name,
                spath.display(),
                tpath.display()
            );
        } else {
            if tpath.exists() && !allow_overwrite {
                log::log!(
                    overwrite_log_level,
                    "SYNC::SKIP [{}]> Target path ({}) exists",
                    group_name,
                    tpath.display(),
                );
            } else {
                // Allows overwrite in this block.
                if method == SyncMethod::Copy {
                    log::debug!(
                        "SYNC::COPY [{}]> {} => {}",
                        group_name,
                        spath.display(),
                        tpath.display()
                    );
                    match std::fs::remove_file(&tpath) {
                        Ok(_) => log::trace!(
                            "SYNC::OVERWRITE [{}]> {}",
                            group_name,
                            tpath.display()
                        ),
                        _ => {}
                    }
                    std::fs::copy(spath, tpath)?;
                } else if method == SyncMethod::Symlink {
                    // Staging
                    log::debug!(
                        "SYNC::STAGE [{}]> {} => {}",
                        group_name,
                        spath.display(),
                        staging_path.display(),
                    );

                    match std::fs::remove_file(&staging_path) {
                        Ok(_) => log::trace!(
                            "SYNC::OVERWRITE [{}]> {}",
                            group_name,
                            staging_path.display(),
                        ),
                        _ => {}
                    }
                    std::fs::copy(spath, &staging_path)?;

                    // Symlinking
                    match std::fs::remove_file(&tpath) {
                        Ok(_) => {
                            log::trace!(
                                "SYNC::OVERWRITE [{}]> {}",
                                group_name,
                                tpath.display(),
                            );
                        }
                        _ => {}
                    }
                    log::debug!(
                        "SYNC::SYMLINK [{}]> {} => {}",
                        group_name,
                        staging_path.display(),
                        tpath.display(),
                    );
                    std::os::unix::fs::symlink(staging_path, tpath)?;
                }
            }
        }
    } else if spath.is_dir() {
        if tpath.is_file() {
            if dry {
                log::error!(
                    "DRYRUN [{}]> A file ({}) exists at the target path of a source directory ({})",
                    group_name,
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
            || method == SyncMethod::Symlink && staging_path.exists().not()
        {
            if dry {
                log::warn!(
                    "DRYRUN [{}]> Non-existing directory: {}",
                    group_name,
                    if tpath.exists().not() {
                        tpath.display()
                    } else {
                        staging_path.display()
                    },
                );
            } else {
                if method == SyncMethod::Symlink
                    && staging_path.exists().not()
                {
                    log::debug!(
                        "SYNC::STAGE::CREATE [{}]> {}",
                        group_name,
                        staging_path.display(),
                    );
                    std::fs::create_dir_all(staging_path)?;
                }
                if tpath.exists().not() {
                    log::debug!(
                        "SYNC::CREATE [{}]> {}",
                        group_name,
                        tpath.display()
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
    use std::{ops::Not, path::PathBuf, str::FromStr};

    use color_eyre::{eyre::eyre, Report};

    use crate::{config::DTConfig, utils};

    use super::expand;

    #[test]
    fn target_is_file_relative() -> Result<(), Report> {
        if let Err(msg) = expand(&DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-target_is_file_relative.toml",
        )?)?) {
            assert_eq!(
                msg.to_string(),
                "Group [target path is relative]: target path exists and is not a directory",
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
        if let Err(msg) = expand(&DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-target_is_file_absolute.toml",
        )?)?) {
            assert_eq!(
                msg.to_string(),
                "Group [target path is absolute]: target path exists and is not a directory",
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
        assert!(filepath.exists().not());
        std::fs::write(&filepath, "Created by `dt` when testing")?;

        // Read config (expected to fail)
        if let Err(msg) = expand(&DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-target_is_file_has_tilde.toml",
        )?)?) {
            assert_eq!(
                msg.to_string(),
                "Group [target contains tilde to be expanded]: target path exists and is not a directory",
            );
        } else {
            return Err(eyre!(
                "This config should not be loaded because target is not a directory",
            ));
        }

        // clean up
        std::fs::remove_file(filepath)?;

        Ok(())
    }

    #[test]
    fn basedir_is_target() -> Result<(), Report> {
        if let Err(msg) = expand(&DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-basedir_is_target.toml",
        )?)?) {
            assert_eq!(
                msg.to_string(),
                "Group [basedir is target]: base directory and its target are the same",
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because basedir and target are the same"))
        }
    }

    #[test]
    fn same_names_in_multiple_locals() -> Result<(), Report> {
        if let Err(msg) = expand(&DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-same_names_in_multiple_locals.toml",
        )?)?) {
            assert_eq!(
                msg.to_string(),
                "Duplicated local group name: wubba lubba dub dub",
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because there are multiple local groups share the same name"))
        }
    }

    #[test]
    fn non_existing_basedir() -> Result<(), Report> {
        if let Err(msg) = expand(&DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-non_existing_basedir.toml"
        )?)?) {
            assert_eq!(
                msg.to_string(),
                format!(
                    "Group [non-existing basedir]: base directory ({}) does not exist",
                    utils::to_absolute(
                        PathBuf::from_str(
                            "../e52e04c1d71dd984b145ae3bfa5a2fa2"
                        )?
                    )?
                    .display(),
                ),
            );
            Ok(())
        } else {
            Err(eyre!(""))
        }
    }
}

#[cfg(test)]
mod expansion {
    use std::{path::PathBuf, str::FromStr};

    use color_eyre::Report;

    use crate::{config::*, utils};

    use super::expand;

    #[test]
    fn glob() -> Result<(), Report> {
        let config = expand(&DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/syncing/expansion-glob.toml",
        )?)?)?;
        for group in &config.local {
            assert_eq!(
                group.sources,
                vec![
                    utils::to_absolute(PathBuf::from_str(
                        "../dt-cli/Cargo.toml"
                    )?)?,
                    utils::to_absolute(PathBuf::from_str(
                        "../dt-cli/src/main.rs"
                    )?)?,
                    utils::to_absolute(PathBuf::from_str(
                        "../dt-core/Cargo.toml"
                    )?)?,
                    utils::to_absolute(PathBuf::from_str(
                        "../dt-core/src/config.rs"
                    )?)?,
                    utils::to_absolute(PathBuf::from_str(
                        "../dt-core/src/lib.rs"
                    )?)?,
                    utils::to_absolute(PathBuf::from_str(
                        "../dt-core/src/syncing.rs"
                    )?)?,
                    utils::to_absolute(PathBuf::from_str(
                        "../dt-core/src/utils.rs"
                    )?)?,
                ],
            );
        }
        Ok(())
    }

    #[test]
    fn sorting_and_deduping() -> Result<(), Report> {
        let config = expand(&DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/syncing/expansion-sorting_and_deduping.toml",
        )?)?)?;
        for group in config.local {
            assert_eq!(
                group.sources,
                vec![
                    utils::to_absolute(PathBuf::from_str(
                        "../testroot/items/sorting_and_deduping/A-a"
                    )?)?,
                    utils::to_absolute(PathBuf::from_str(
                        "../testroot/items/sorting_and_deduping/A-b"
                    )?)?,
                    utils::to_absolute(PathBuf::from_str(
                        "../testroot/items/sorting_and_deduping/A-c"
                    )?)?,
                    utils::to_absolute(PathBuf::from_str(
                        "../testroot/items/sorting_and_deduping/B-a"
                    )?)?,
                    utils::to_absolute(PathBuf::from_str(
                        "../testroot/items/sorting_and_deduping/B-b"
                    )?)?,
                    utils::to_absolute(PathBuf::from_str(
                        "../testroot/items/sorting_and_deduping/B-c"
                    )?)?,
                ]
            );
        }
        Ok(())
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 23 2021, 00:05 [CST]
