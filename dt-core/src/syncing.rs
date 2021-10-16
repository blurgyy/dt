use std::{
    ops::Not,
    path::{Path, PathBuf},
    str::FromStr,
};

use color_eyre::{eyre::eyre, Report};

use crate::{config::*, utils};

/// Parameters for controlling how an item is synced.
struct SyncingParameters {
    /// Path to source item, assumed to exist, and is already host-specific if possible.
    spath: PathBuf,
    /// Path to the parent dir of the disired sync destination.
    tparent: PathBuf,
    /// Whether to issue a dry run.
    dry: bool,
    /// Whether overwrite existing files or not.
    allow_overwrite: bool,
    /// A `SyncMethod` instance.
    method: SyncMethod,
    /// Path to staging directory.
    staging: PathBuf,
    /// Name of current group.
    group_name: String,
    /// Configured `basedir` of current group
    basedir: PathBuf,
    /// Separator for per-host settings.
    hostname_sep: String,
}

/// Expand tilde and globs in "sources" and manifest new config object.
fn expand(config: &DTConfig) -> Result<DTConfig, Report> {
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
        let mut next = LocalGroup {
            basedir: utils::to_absolute(&original.basedir)?,
            sources: vec![],
            target: utils::to_absolute(&original.target)?,
            ..original.to_owned()
        };

        let group_hostname_sep =
            original.get_hostname_sep(ret.global.as_ref().unwrap_or_else(|| {
                unreachable!("The [global] object is already a `Some`, this message should never be seen");
            }));

        // Check for host-specific basedir
        let host_specific_basedir =
            utils::to_host_specific(&next.basedir, &group_hostname_sep)?;
        if host_specific_basedir.exists() {
            next.basedir = host_specific_basedir;
        }

        // Check for host-specific sources
        let sources: Vec<PathBuf> = original
            .sources
            .iter()
            .map(|s| {
                let try_s = utils::to_absolute(next.basedir.join(s))
                    .unwrap_or_else(|e| panic!("{}", e));
                let try_s =
                    utils::to_host_specific(try_s, &group_hostname_sep)
                        .unwrap_or_else(|e| panic!("{}", e));
                if try_s.exists() {
                    try_s
                } else {
                    s.to_owned()
                }
            })
            .collect();

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

    check(&ret)?;

    Ok(ret)
}

fn expand_recursive(
    path: &Path,
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
                    x.unwrap_or_else(|_| {
                        panic!(
                            "Failed globbing source path {}",
                            path.display(),
                        )
                    })
                })
                // Filter out paths that are meant for other hosts
                .filter(|x| utils::is_for_other_host(x, hostname_sep).not())
                // Convert to absolute paths
                .map(|x| {
                    utils::to_absolute(&x).unwrap_or_else(|_| {
                        panic!(
                            "Failed converting to absolute path: {}",
                            x.display(),
                        )
                    })
                })
                .collect();

        let mut ret: Vec<PathBuf> = Vec::new();
        for p in initial {
            if p.is_file() {
                ret.push(p);
            } else if p.is_dir() {
                ret.append(&mut expand_recursive(&p, hostname_sep, false)?);
            } else {
                log::warn!(
                    "Skipping unimplemented file type at {}",
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
                    panic!("Cannot read dir properly: {}", path.display())
                })
                .path()
            })
            .filter(|x| utils::is_for_other_host(x, hostname_sep).not())
            .collect();

        let mut ret: Vec<PathBuf> = Vec::new();
        for p in initial {
            if p.is_file() {
                ret.push(p);
            } else if p.is_dir() {
                ret.append(&mut expand_recursive(&p, hostname_sep, false)?);
            } else {
                log::warn!(
                    "Skipping unimplemented file type at {}",
                    p.display(),
                );
                log::trace!("{:#?}", p.symlink_metadata()?);
            }
        }

        Ok(ret)
    }
}

fn check(config: &DTConfig) -> Result<(), Report> {
    let mut has_symlink: bool = false;

    for group in &config.local {
        if has_symlink.not()
            && group.get_method(&config.global.to_owned().unwrap_or_default())
                == SyncMethod::Symlink
        {
            has_symlink = true;
        }

        // Non-existing basedir
        if group.basedir.exists().not() {
            return Err(eyre!(
                "Group [{}]: basedir path does not exist",
                group.name,
            ));
        }

        // Wrong type of existing basedir path
        if group.basedir.is_dir().not() {
            return Err(eyre!(
                "Group [{}]: basedir path exists but is not a valid directory",
                group.name,
            ));
        }

        // Wrong type of existing target path
        if group.target.exists() && !group.target.is_dir() {
            return Err(eyre!(
                "Group [{}]: target path exists but is not a valid directory",
                group.name,
            ));
        }

        // Path to target contains readonly parent directory
        if parent_readonly(&group.target) {
            return Err(eyre!(
                "Group [{}]: target path cannot be created due to insufficient permissions",
                group.name,
            ));
        }

        for s in &group.sources {
            if s.is_file() && std::fs::File::open(s).is_err() {
                return Err(eyre!(
                    "Group [{}]: there exists one or more source item(s) that is not readable",
                    group.name,
                ));
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
        if staging.exists() && staging.is_dir().not() {
            return Err(eyre!(
                "Staging root path exists but is not a valid directory",
            ));
        }

        // Path to staging root contains readonly parent directory
        if parent_readonly(staging) {
            return Err(eyre!(
                "Staging root path cannot be created due to insufficient permissions"
            ));
        }
    }

    Ok(())
}

fn parent_readonly(p: impl AsRef<Path>) -> bool {
    let mut p = p.as_ref();
    let first_existing_parent = loop {
        if p.exists() {
            break p;
        }
        p = p.parent().unwrap();
    };
    first_existing_parent
        .metadata()
        .unwrap()
        .permissions()
        .readonly()
}

/// Syncs items specified in configuration.
pub fn sync(config: &DTConfig) -> Result<(), Report> {
    let config = expand(config)?;

    let staging = &config
        .global
        .to_owned()
        .unwrap_or_default()
        .staging
        .unwrap_or_else(|| GlobalConfig::default().staging.unwrap());
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

/// Show changes to be made according to configuration, without actually syncing items.
pub fn dry_sync(config: &DTConfig) -> Result<(), Report> {
    let config = expand(config)?;

    let staging = &config
        .global
        .to_owned()
        .unwrap_or_default()
        .staging
        .unwrap_or_else(|| GlobalConfig::default().staging.unwrap());
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
///
/// Args:
fn sync_core(params: SyncingParameters) -> Result<(), Report> {
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
            std::fs::create_dir_all(&tparent)?;
        }
    }

    // First, get target path (without the per-host suffix).
    let tpath = tparent.join(
        utils::to_non_host_specific(&spath, &hostname_sep)?.strip_prefix(
            utils::to_non_host_specific(&basedir, &hostname_sep)?,
        )?,
    );
    std::fs::create_dir_all(tpath.parent().unwrap_or_else(|| {
        panic!(
            "Structrue of target directory could not be created at {}",
            tpath.display(),
        )
    }))?;

    // Finally, get the staging path with source path (staging path does not have host-specific
    // suffix).
    let staging_path = staging.join(
        utils::to_non_host_specific(&spath, &hostname_sep)?.strip_prefix(
            utils::to_non_host_specific(&basedir, &hostname_sep)?,
        )?,
    );
    std::fs::create_dir_all(staging_path.parent().unwrap_or_else(|| {
        panic!(
            "Structrue of staging directory could not be created at {}",
            staging_path.display(),
        )
    }))?;

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
                if allow_overwrite {
                    log::info!(
                        "DRYRUN::OVERWRITE [{}]> {} -> {}",
                        group_name,
                        spath.display(),
                        tpath.display()
                    );
                } else {
                    log::error!(
                        "DRYRUN [{}]> Target path ({}) exists",
                        group_name,
                        tpath.display(),
                    );
                }
            } else {
                log::info!(
                    "DRYRUN [{}]> {} -> {}",
                    group_name,
                    spath.display(),
                    tpath.display()
                );
            }
        } else if tpath.exists() && allow_overwrite.not() {
            log::error!(
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
                if std::fs::remove_file(&tpath).is_ok() {
                    log::trace!(
                        "SYNC::OVERWRITE [{}]> {}",
                        group_name,
                        tpath.display()
                    )
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

                if std::fs::remove_file(&staging_path).is_ok() {
                    log::trace!(
                        "SYNC::OVERWRITE [{}]> {}",
                        group_name,
                        staging_path.display(),
                    )
                }
                std::fs::copy(spath, &staging_path)?;

                // Symlinking
                if std::fs::remove_file(&tpath).is_ok() {
                    {
                        log::trace!(
                            "SYNC::OVERWRITE [{}]> {}",
                            group_name,
                            tpath.display(),
                        );
                    }
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
    use std::{
        ops::Not, os::unix::prelude::PermissionsExt, path::PathBuf,
        str::FromStr,
    };

    use color_eyre::{eyre::eyre, Report};

    use crate::config::DTConfig;

    use super::expand;

    #[test]
    fn non_existing_basedir() -> Result<(), Report> {
        if let Err(msg) = expand(&DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-non_existing_basedir.toml"
        )?)?) {
            assert_eq!(
                msg.to_string(),
                "Group [non-existing basedir]: basedir path does not exist",
                "{}",
                msg,
            );
            Ok(())
        } else {
            Err(eyre!(""))
        }
    }

    #[test]
    fn basedir_is_file() -> Result<(), Report> {
        if let Err(msg) = expand(&DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-basedir_is_file.toml",
        )?)?) {
            assert_eq!(
                msg.to_string(),
                "Group [basedir path is file]: basedir path exists but is not a valid directory",
                "{}",
                msg,
            );
            Ok(())
        } else {
            Err(eyre!(
                "This config should not be loaded because basedir is not a directory",
            ))
        }
    }

    #[test]
    fn target_is_file_relative() -> Result<(), Report> {
        if let Err(msg) = expand(&DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-target_is_file_relative.toml",
        )?)?) {
            assert_eq!(
                msg.to_string(),
                "Group [target path is relative]: target path exists but is not a valid directory",
                "{}",
                msg,
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
        if let Err(msg) = expand(&DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-target_is_file_absolute.toml",
        )?)?) {
            assert_eq!(
                msg.to_string(),
                "Group [target path is absolute]: target path exists but is not a valid directory",
                "{}",
                msg,
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
            filepath.exists().not(),
            "A previous test seem to abort abnormally, remove the file d6a8e0bc1647c38548432ccfa1d79355 in your $HOME to continue testing",
        );
        std::fs::write(&filepath, "Created by `dt` when testing")?;

        // Read config (expected to fail)
        if let Err(msg) = expand(&DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-target_is_file_has_tilde.toml",
        )?)?) {
            assert_eq!(
                msg.to_string(),
                "Group [target contains tilde to be expanded]: target path exists but is not a valid directory",
                "{}",
                msg,
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
    fn target_readonly() -> Result<(), Report> {
        std::fs::set_permissions(
            PathBuf::from_str(
                "../testroot/items/syncing/invalid_configs/target_readonly/target")?,
                std::fs::Permissions::from_mode(0o555),
        )?;
        if let Err(msg) = expand(&DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-target_readonly.toml",
        )?)?) {
            assert_eq!(
                msg.to_string(),
                "Group [target is readonly]: target path cannot be created due to insufficient permissions",
                "{}",
                msg,
            );
            std::fs::set_permissions(
                PathBuf::from_str(
                    "../testroot/items/syncing/invalid_configs/target_readonly/target")?,
                    std::fs::Permissions::from_mode(0o755),
            )?;
            Ok(())
        } else {
            std::fs::set_permissions(
                PathBuf::from_str(
                    "../testroot/items/syncing/invalid_configs/target_readonly/target")?,
                    std::fs::Permissions::from_mode(0o755),
            )?;
            Err(eyre!(
                "This config should not be loaded because target path is readonly",
            ))
        }
    }

    #[test]
    fn staging_is_file() -> Result<(), Report> {
        if let Err(msg) = expand(&DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-staging_is_file.toml",
        )?)?) {
            assert_eq!(
                msg.to_string(),
                "Staging root path exists but is not a valid directory",
                "{}",
                msg,
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
                "../testroot/items/syncing/invalid_configs/staging_readonly/staging")?,
                std::fs::Permissions::from_mode(0o555),
        )?;
        if let Err(msg) = expand(&DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-staging_readonly.toml",
        )?)?) {
            assert_eq!(
                msg.to_string(),
                "Staging root path cannot be created due to insufficient permissions",
                "{}",
                msg,
            );
            std::fs::set_permissions(
                PathBuf::from_str(
                    "../testroot/items/syncing/invalid_configs/staging_readonly/staging")?,
                    std::fs::Permissions::from_mode(0o755),
            )?;
            Ok(())
        } else {
            std::fs::set_permissions(
                PathBuf::from_str(
                    "../testroot/items/syncing/invalid_configs/staging_readonly/staging")?,
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
        if let Err(msg) = expand(&DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-unreadable_source.toml",
        )?)?) {
            assert_eq!(
                msg.to_string(),
                "Group [source is unreadable]: there exists one or more source item(s) that is not readable",
                "{}",
                msg,
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

    use crate::{config::*, utils};

    use super::expand;

    #[test]
    fn glob() -> Result<(), Report> {
        let config = expand(&DTConfig::from_path(PathBuf::from_str(
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
                        "../dt-cli/README.md",
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
        let config = expand(&DTConfig::from_path(PathBuf::from_str(
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
