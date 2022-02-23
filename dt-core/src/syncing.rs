use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::{
    config::*,
    error::{Error as AppError, Result},
    item::Operate,
    registry::{Register, Registry},
};

/// Expands tildes and globs in [`sources`], returns the updated config
/// object.
///
/// It does the following operations on given config:
///
/// 1. Convert all [`base`]s, [`target`]s to absolute paths.
/// 2. Replace [`base`]s and paths in [`sources`] with their host-specific
///    counterpart, if there exists any.
/// 3. Recursively expand globs and directories found in [`sources`].
///
/// [`sources`]: crate::config::Group::sources
/// [`global.staging`]: crate::config::GlobalConfig::staging
/// [`base`]: crate::config::Group::base
/// [`target`]: crate::config::Group::target
fn expand(config: DTConfig) -> Result<DTConfig> {
    let mut ret = DTConfig {
        // Remove `global` and `context` in expanded configuration object.
        // Further references of these two values are referenced via Rc from
        // within groups.
        global: config.global,
        context: config.context,
        local: Vec::new(),
        remote: Vec::new(),
    };

    for original in config.local {
        let mut next = LocalGroup {
            global: Rc::clone(&original.global),
            base: original.base.to_owned().absolute()?,
            sources: Vec::new(),
            target: original.target.to_owned().absolute()?,
            ..original.to_owned()
        };

        let group_hostname_sep = original.get_hostname_sep();

        // Check for host-specific `base`
        let host_specific_base =
            next.base.to_owned().host_specific(&group_hostname_sep);
        if host_specific_base.exists() {
            next.base = host_specific_base;
        }

        // Check for host-specific `sources`
        let sources: Vec<PathBuf> = original
            .sources
            .iter()
            .map(|s| {
                let try_s = next
                    .base
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
            let s = next.base.join(s);
            let mut s = expand_recursive(&s, &next.get_hostname_sep(), true)?;
            next.sources.append(&mut s);
        }
        next.sources.sort();
        next.sources.dedup();
        ret.local.push(next);
    }

    let ret = resolve(ret)?;

    check_readable(&ret)?;

    Ok(ret)
}

/// Recursively expands glob from a given path.
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
                // **After** filtering out paths that are meant for other
                // hosts, replace current path to its host-specific
                // counterpart if it exists.
                .map(|x| {
                    let host_specific_x =
                        x.to_owned().host_specific(hostname_sep);
                    if host_specific_x.exists() {
                        host_specific_x
                    } else {
                        x
                    }
                })
                // Convert to absolute paths
                .map(|x| {
                    x.to_owned().absolute().unwrap_or_else(|_| {
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
            // Filter out paths that are meant for other hosts
            .filter(|x| !x.is_for_other_host(hostname_sep))
            // **After** filtering out paths that are meant for other
            // hosts, replace current path to its host-specific
            // counterpart if it exists.
            .map(|x| {
                let host_specific_x =
                    x.to_owned().host_specific(hostname_sep);
                if host_specific_x.exists() {
                    host_specific_x
                } else {
                    x
                }
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
                    "Skipping unimplemented file type at '{}'",
                    p.display(),
                );
                log::trace!("{:#?}", p.symlink_metadata()?);
            }
        }

        Ok(ret)
    }
}

/// Resolve priorities within expanded [`DTConfig`], this function is called
/// after [`expand`] so that it can correctly resolve priorities of all
/// expanded sources, and before [`check_readable`], since it does not have to
/// query the filesystem.
fn resolve(config: DTConfig) -> Result<DTConfig> {
    // Maps an item to the index of the group which holds the highest priority
    // of it.
    let mut mapping: HashMap<PathBuf, usize> = HashMap::new();

    // Get each item's highest priority group.
    for i in 0..config.local.len() {
        let current_priority = &config.local[i].scope;
        for s in &config.local[i].sources {
            let t = s.to_owned().make_target(
                &config.local[i].get_hostname_sep(),
                &config.local[i].base,
                &config.local[i].target,
                config.local[i].get_renaming_rules(),
            )?;
            match mapping.get(&t) {
                Some(prev_group_idx) => {
                    let prev_priority = &config.local[*prev_group_idx].scope;
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
                    .filter(|&s| {
                        let t = s
                            .to_owned()
                            .make_target(
                                &group.get_hostname_sep(),
                                &group.base,
                                &group.target,
                                group.get_renaming_rules(),
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

/// Checks validity of the given [DTConfig].
fn check_readable(config: &DTConfig) -> Result<()> {
    for group in &config.local {
        for s in &group.sources {
            if std::fs::File::open(s).is_err() {
                return Err(AppError::IoError(format!(
                    "'{}' is not readable in group '{}'",
                    s.display(),
                    group.name,
                )));
            }
            if !s.is_file() {
                unreachable!();
            }
        }
    }

    Ok(())
}

/// Syncs items specified with given [DTConfig].
pub fn sync(config: DTConfig, dry_run: bool) -> Result<()> {
    if config.local.is_empty() {
        log::warn!("Nothing to be synced");
        return Ok(());
    }
    log::trace!("Local groups to process: {:#?}", config.local);

    let config = expand(config)?;
    let registry =
        Rc::new(Registry::default().register_helpers()?.load(&config)?);

    for group in &config.local {
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

        let group_ref = Rc::new(group.to_owned());
        for spath in &group.sources {
            if dry_run {
                spath.populate_dry(Rc::clone(&group_ref))?;
            } else {
                spath
                    .populate(Rc::clone(&group_ref), Rc::clone(&registry))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    mod validation {
        use std::str::FromStr;

        use color_eyre::{eyre::eyre, Report};
        use pretty_assertions::assert_eq;

        use crate::config::DTConfig;
        use crate::error::Error as AppError;

        use super::super::expand;
        use crate::utils::testing::{
            get_testroot, prepare_directory, prepare_file,
        };

        #[test]
        fn unreadable_source() -> Result<(), Report> {
            // setup
            let source_basename = "src-file-but-unreadable";
            let base = prepare_directory(
                get_testroot().join("unreadable_source").join("base"),
                0o755,
            )?;
            let _source_path =
                prepare_file(base.join(source_basename), 0o200)?;
            let target_path = prepare_directory(
                get_testroot().join("unreadable_source").join("target"),
                0o755,
            )?;

            if let Err(err) = expand(
                DTConfig::from_str(&format!(
                    r#"
[[local]]
name = "source is unreadable"
base = "{}"
sources = ["{}"]
target = "{}""#,
                    base.display(),
                    source_basename,
                    target_path.display(),
                ))
                .unwrap(),
            ) {
                assert_eq!(
                err,
                AppError::IoError(
                    "'/tmp/dt-testing/syncing/unreadable_source/base/src-file-but-unreadable' is not readable in group 'source is unreadable'"
                        .to_owned(),
                ),
                "{}",
                err,
            );
                Ok(())
            } else {
                Err(eyre!("This config should not be loaded because source item is not readable"))
            }
        }
    }

    mod expansion {
        use std::{path::PathBuf, str::FromStr};

        use color_eyre::Report;
        use pretty_assertions::assert_eq;

        use crate::{config::*, item::Operate};

        use super::super::expand;
        use crate::utils::testing::{
            get_testroot, prepare_directory, prepare_file,
        };

        #[test]
        fn glob() -> Result<(), Report> {
            let target_path = prepare_directory(
                get_testroot().join("glob").join("target"),
                0o755,
            )?;

            let config = expand(
                DTConfig::from_str(&format!(
                    r#"
[[local]]
name = "globbing test"
base = ".."
sources = ["dt-c*"]
target = "{}""#,
                    target_path.display(),
                ))
                .unwrap(),
            )?;
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
                        PathBuf::from_str("../dt-core/src/registry.rs")
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
            println!("Creating base ..");
            let base_path = prepare_directory(
                get_testroot().join("sorting_and_deduping").join("base"),
                0o755,
            )?;
            println!("Creating target ..");
            let target_path = prepare_directory(
                get_testroot().join("sorting_and_deduping").join("target"),
                0o755,
            )?;
            for f in ["A-a", "A-b", "A-c", "B-a", "B-b", "B-c"] {
                println!("Creating source {} ..", f);
                prepare_file(base_path.join(f), 0o644)?;
            }

            let config = expand(
                DTConfig::from_str(&format!(
                    r#"
[[local]]
name = "sorting and deduping"
base = "{}"
sources = ["B-*", "*-c", "A-b", "A-a"]
target = "{}""#,
                    base_path.display(),
                    target_path.display(),
                ))
                .unwrap(),
            )?;
            for group in config.local {
                assert_eq!(
                    group.sources,
                    vec![
                        base_path.join("A-a"),
                        base_path.join("A-b"),
                        base_path.join("A-c"),
                        base_path.join("B-a"),
                        base_path.join("B-b"),
                        base_path.join("B-c"),
                    ],
                );
            }
            Ok(())
        }
    }

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
        fn former_group_has_higher_priority_within_same_scope() -> Result<()>
        {
            let config = expand(DTConfig::from_str(
                r#"
                [[local]]
                name = "highest"
                # Scope is omitted to use default scope (i.e. General)
                base = "../dt-cli"
                sources = ["Cargo.toml"]
                target = "."
                [[local]]
                name = "low"
                # Scope is omitted to use default scope (i.e. General)
                base = "../dt-server"
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
            let config = expand(DTConfig::from_str(
                r#"
                [[local]]
                name = "lowest"
                scope = "General"
                base = "../dt-cli"
                sources = ["Cargo.toml"]
                target = "."
                [[local]]
                name = "medium"
                scope = "App"
                base = "../dt-server"
                sources = ["Cargo.toml"]
                target = "."
                [[local]]
                name = "highest"
                scope = "Dropin"
                base = ".."
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
            let config = expand(DTConfig::from_str(
                r#"
                [[local]]
                name = "lowest"
                scope = "General"
                base = "../dt-cli"
                sources = ["Cargo.toml"]
                target = "."
                [[local]]
                name = "medium"
                scope = "App"
                base = "../dt-server"
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
            let config = expand(DTConfig::from_str(
                r#"
                [[local]]
                name = "omitted scope but defined first, has higher priority"
                # Scope is omitted to use default scope (i.e. General)
                base = "../dt-cli"
                sources = ["Cargo.toml"]
                target = "."
                [[local]]
                name = "specified scope but defined last, has lower priority"
                scope = "General"
                base = "../dt-server"
                sources = ["Cargo.toml"]
                target = "."
            "#,
            )?)?;

            assert!(!config.local[0].sources.is_empty());
            assert!(config.local[1].sources.is_empty());

            let config = expand(DTConfig::from_str(
                r#"
                [[local]]
                name = "omitted scope, uses general"
                # Scope is omitted to use default scope (i.e. General)
                base = ".."
                sources = ["Cargo.toml"]
                target = "."
                [[local]]
                name = "specified scope with higher priority"
                scope = "App"
                base = ".."
                sources = ["Cargo.toml"]
                target = "."
            "#,
            )?)?;

            assert!(config.local[0].sources.is_empty());
            assert!(!config.local[1].sources.is_empty());

            Ok(())
        }

        #[test]
        fn duplicated_item_same_name_same_scope() -> Result<()> {
            let config = expand(DTConfig::from_str(
                r#"
                [[local]]
                name = "dup"
                scope = "General"
                base = "../dt-cli"
                sources = ["Cargo.toml"]
                target = "."
                [[local]]
                name = "dup"
                scope = "General"
                base = "../dt-server"
                sources = ["Cargo.toml"]
                target = "."
            "#,
            )?)?;

            assert!(!config.local[0].sources.is_empty());
            assert!(config.local[1].sources.is_empty());

            Ok(())
        }

        #[test]
        fn duplicated_item_same_name_different_scope() -> Result<()> {
            let config = expand(DTConfig::from_str(
                r#"
                [[local]]
                name = "dup"
                scope = "General"
                base = "../dt-cli"
                sources = ["Cargo.toml"]
                target = "."
                [[local]]
                name = "dup"
                scope = "App"
                base = "../dt-server"
                sources = ["Cargo.toml"]
                target = "."
            "#,
            )?)?;

            assert!(config.local[0].sources.is_empty());
            assert!(!config.local[1].sources.is_empty());

            Ok(())
        }
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 23 2021, 00:05 [CST]
