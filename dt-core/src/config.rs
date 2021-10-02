use std::{panic, path::PathBuf, str::FromStr};

use color_eyre::{eyre::eyre, Report};
use serde::Deserialize;

/// The configuration object constructed from configuration file.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct DTConfig {
    /// (Optional) Sets fallback behaviours.
    pub global: Option<GlobalConfig>,

    /// Local items groups.
    pub local: Vec<LocalSyncConfig>,
}

impl DTConfig {
    /// Loads configuration from a file.
    pub fn from_pathbuf(path: PathBuf) -> Result<Self, Report> {
        let confstr = std::fs::read_to_string(path)?;
        Self::from_str(&confstr)
    }

    /// Loads configuration from string.
    fn from_str(s: &str) -> Result<Self, Report> {
        let ret: Self = toml::from_str(s)?;
        ret.validate_pre_expansion()?;
        let ret = ret.expand()?;
        match ret.validate_post_expansion() {
            Ok(()) => Ok(ret),
            Err(e) => Err(e),
        }
    }

    fn validate_pre_expansion(self: &Self) -> Result<(), Report> {
        let mut group_name_rec: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for group in &self.local {
            if let Some(_) = group_name_rec.get(&group.name) {
                return Err(eyre!("Duplicated group name: {}", group.name));
            }
            group_name_rec.insert(group.name.to_owned());
            for s in &group.sources {
                if let Some(strpath) = s.to_str() {
                    if strpath == ".*" || strpath.contains("/.*") {
                        return Err(eyre!(
                            "Do not use globbing patterns like '.*', because it also matches curent directory (.) and parent directory (..)"
                        ));
                    }
                } else {
                    return Err(eyre!(
                        "Invalide unicode encountered in sources"
                    ));
                }
            }
            if group.target.exists() && !group.target.is_dir() {
                return Err(eyre!(
                    "Target path exists and is not a directory"
                ));
            }
            for i in &group.ignored {
                if i.contains(&"/".to_owned()) {
                    return Err(eyre!(
                        "Ignored pattern contains slash, this is not allowed"
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_post_expansion(self: &Self) -> Result<(), Report> {
        for group in &self.local {
            if !group.basedir.is_dir() {
                return Err(eyre!(
                    "Configured basedir {} is invalid",
                    group.basedir.display(),
                ));
            }
        }
        Ok(())
    }

    /// Expand tilde and globs in "sources" and manifest new config object.
    fn expand(&self) -> Result<Self, Report> {
        let globbing_options = glob::MatchOptions {
            case_sensitive: true,
            require_literal_separator: true,
            require_literal_leading_dot: true,
        };
        let mut ret = Self {
            global: self.global.to_owned(),
            local: vec![],
        };
        for original in &self.local {
            let mut next = LocalSyncConfig {
                basedir: PathBuf::from_str(&shellexpand::tilde(
                    original.basedir.to_str().unwrap(),
                ))?
                .canonicalize()?,
                sources: vec![],
                target: PathBuf::from_str(&shellexpand::tilde(
                    original.target.to_str().unwrap(),
                ))?
                .canonicalize()?,
                ..original.to_owned()
            };
            for s in &original.sources {
                let s = next.basedir.join(s);
                let mut s =
                    glob::glob_with(s.to_str().unwrap(), globbing_options)?
                        .map(|x| {
                            x.expect(&format!(
                                "Failed globbing source path {}",
                                s.display(),
                            ))
                        })
                        .filter(|x| {
                            if let Some(ignored) = &next.ignored {
                                if ignored.len() == 0 {
                                    true
                                } else {
                                    ignored.iter().any(|y| {
                                        x.iter()
                                            .all(|z| z.to_str().unwrap() != y)
                                    })
                                }
                            } else {
                                true
                            }
                        })
                        .map(|x| {
                            x.canonicalize().expect(&format!(
                                "Failed canonicalizing path {}",
                                x.display(),
                            ))
                        })
                        .collect();
                next.sources.append(&mut s);
            }
            ret.local.push(next);
        }

        Ok(ret)
    }
}

/// Configures how local items (files/directories) are synced.
#[derive(Default, Clone, Deserialize, Debug)]
pub struct LocalSyncConfig {
    /// Name of this group, used as namespaces when staging.
    pub name: String,

    /// The base directory of all source items.  This simplifies configuration files with common
    /// prefixes in `local.sources` array.
    ///
    /// ## Example
    ///
    /// For a directory structure like:
    ///
    /// ```plain
    /// dt/
    /// ├── dt-core/
    /// │  └── src/
    /// │     └── config.rs
    /// ├── dt-cli/
    /// │  └── src/
    /// │     └── main.rs
    /// └── README.md
    /// ```
    ///
    /// Consider the following config file:
    ///
    /// ```toml
    /// [[local]]
    /// basedir = "dt/dt-cli"
    /// sources = ["*"]
    /// target = "."
    /// ```
    ///
    /// It will only sync `src/main.rs` to the configured target directory (in this case, the
    /// directory where `dt` is being executed).
    pub basedir: PathBuf,

    /// Paths (relative to `basedir`) to the items to be synced.
    pub sources: Vec<PathBuf>,

    /// The absolute path of the parent dir of the final synced items.
    ///
    /// ## Example
    ///
    /// ```toml
    /// source = ["/source/file"]
    /// target = "/tar/get"
    /// ```
    ///
    /// will sync "/source/file" to "/tar/get/file" (creating non-existing directories along the way), while
    ///
    /// ```toml
    /// source = ["/source/dir"]
    /// target = "/tar/get/dir"
    /// ```
    ///
    /// will sync "source/dir" to "/tar/get/dir/dir" (creating non-existing directories along the way).
    pub target: PathBuf,

    /// (Optional) Ignored names.
    ///
    /// ## Example
    ///
    /// Consider the following ignored setting:
    ///
    /// ```toml
    /// ignored = [".git"]
    /// ```
    ///
    /// With this setting, all files or directories with their basename as ".git" will be skipped.
    ///
    /// Cannot contain slash in any of the patterns.
    pub ignored: Option<Vec<String>>,

    /// (Optional) Whether to allow overwriting existing files.  Dead symlinks are treated as
    /// non-existing, and are always overwrited (regardless of this option).
    allow_overwrite: Option<bool>,

    /// (Optional) Syncing method, overrides `global.method` key.
    method: Option<SyncMethod>,
    // // The pattern specified in `match_begin` is matched against all
    // match_begin: String,
    // replace_begin: String,
    // match_end: String,
    // replace_end: String,
}

impl LocalSyncConfig {
    /// Gets the `allow_overwrite` key from a `LocalSyncConfig` object, falls back to the `allow_overwrite` from provided global config.
    pub fn get_allow_overwrite(&self, global_config: &GlobalConfig) -> bool {
        match self.allow_overwrite {
            Some(allow_overwrite) => allow_overwrite,
            _ => global_config.allow_overwrite,
        }
    }

    /// Gets the `method` key from a `LocalSyncConfig` object, falls back to the `method` from provided global config.
    pub fn get_method(&self, global_config: &GlobalConfig) -> SyncMethod {
        match self.method {
            Some(method) => method,
            _ => global_config.method,
        }
    }
}

/// Configures default behaviours.
#[derive(Clone, Debug, Deserialize)]
pub struct GlobalConfig {
    /// The staging root directory.
    ///
    /// Only works when `method` (see below) is set to `Symlink`.  When syncing with `Symlink`
    /// method, items will be copied to their staging directory (composed by joining staging root
    /// directory with their group name), then symlinked (as of `ln -sf`) from their staging
    /// directory to the target directory.
    ///
    /// Default to `$XDG_CACHE_HOME/dt/staging` if `XDG_CACHE_HOME` is set, or
    /// `$HOME/.cache/dt/staging` if `HOME` is set.  Panics when neither `XDG_CACHE_HOME` nor
    /// `HOME` is set and config file does not specify this.
    pub staging: Option<PathBuf>,

    /// The syncing method.
    ///
    /// Available values are:
    ///
    /// - `Copy`
    /// - `Symlink`
    ///
    /// When `method` is `Copy`, the above `staging` setting will be disabled.
    pub method: SyncMethod,

    /// Whether to allow overwriting existing files.
    ///
    /// This alters syncing behaviours when the target file exists.  If set to `true`, no
    /// errors/warnings will be omitted when the target file exists; otherwise reports error and
    /// skips the existing item.  Using dry run to spot the existing files before syncing is
    /// recommended.
    pub allow_overwrite: bool,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        let default_staging: PathBuf;
        if let Ok(xdg_cache_home) = std::env::var("XDG_CACHE_HOME") {
            log::debug!(
                "Using environment variable XDG_CACHE_HOME to determine staging directory"
            );
            default_staging = PathBuf::from_str(&xdg_cache_home)
                .expect("Failed constructing default staging directory from xdg_cache_home")
                .join("dt")
                .join("staging");
        } else if let Ok(home) = std::env::var("HOME") {
            log::debug!(
                "Using environment variable HOME to determine staging directory"
            );
            default_staging = PathBuf::from_str(&home)
                .expect(
                    "Failed constructing default staging directory from home",
                )
                .join(".cache")
                .join("dt")
                .join("staging");
        } else {
            panic!("Cannot infer staging directory, set either XDG_CACHE_HOME or HOME to solve this.");
        }
        GlobalConfig {
            staging: Some(default_staging),
            method: SyncMethod::Symlink,
            allow_overwrite: false,
        }
    }
}

/// Syncing methods.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub enum SyncMethod {
    /// Instructs syncing module to directly copy each item from source to target.
    Copy,

    /// Instructs syncing module to first copy iach item from source to its staging directory, then
    /// symlink staged items from their staging directory to target.
    Symlink,
}

impl Default for SyncMethod {
    fn default() -> Self {
        SyncMethod::Symlink
    }
}

#[cfg(test)]
mod validating {
    use std::{path::PathBuf, str::FromStr};

    use color_eyre::{eyre::eyre, Report};

    use super::DTConfig;

    #[test]
    fn s_file_t_file() -> Result<(), Report> {
        if let Err(msg) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/s_file_t_file.toml",
        )?) {
            assert_eq!(
                msg.to_string(),
                "Target path exists and is not a directory",
            );
            Ok(())
        } else {
            Err(eyre!(
                "This config should not be loaded because target is not a directory",
            ))
        }
    }

    #[test]
    fn s_file_t_dir() -> Result<(), Report> {
        if let Ok(_config) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/s_file_t_dir.toml",
        )?) {
            Ok(())
        } else {
            Err(eyre!(
                "This config should be loaded because target is a directory"
            ))
        }
    }

    #[test]
    fn s_dir_t_dir() -> Result<(), Report> {
        if let Ok(_config) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/s_dir_t_dir.toml",
        )?) {
            Ok(())
        } else {
            Err(eyre!(
                "This config should be loaded because target is a directory"
            ))
        }
    }

    #[test]
    fn s_dir_t_file() -> Result<(), Report> {
        if let Err(msg) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/s_dir_t_file.toml",
        )?) {
            assert_eq!(
                msg.to_string(),
                "Target path exists and is not a directory",
            );
            Ok(())
        } else {
            Err(eyre!(
                "This config should not be loaded because target is not a directory",
            ))
        }
    }
}

#[cfg(test)]
mod paths_expansion {
    use std::path::PathBuf;
    use std::str::FromStr;

    use color_eyre::{eyre::eyre, Report};

    use super::DTConfig;

    #[test]
    fn except_dot_asterisk_glob() -> Result<(), Report> {
        if let Err(msg) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/except_dot_asterisk_glob.toml",
        )?) {
            assert_eq!(
                msg.to_string(),
                "Do not use globbing patterns like '.*', because it also matches curent directory (.) and parent directory (..)",
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because it contains bad globs (.* and /.*)"))
        }
    }

    #[test]
    fn tilde() -> Result<(), Report> {
        if let Ok(home) = std::env::var("HOME") {
            if let Ok(config) = DTConfig::from_pathbuf(PathBuf::from_str(
                "../testroot/configs/expand_tilde.toml",
            )?) {
                for local in &config.local {
                    assert_eq!(local.basedir.to_str(), Some(home.as_str()));
                    assert_eq!(local.target.to_str(), Some(home.as_str()));
                }
                Ok(())
            } else {
                Err(eyre!(
                    "This config should be loaded because target is a directory"
                ))
            }
        } else {
            Err(eyre!(
                "Set the `HOME` environment variable to complete this test"
            ))
        }
    }

    #[test]
    fn glob() -> Result<(), Report> {
        if let Ok(config) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/expand_glob.toml",
        )?) {
            for local in &config.local {
                assert_eq!(
                    vec![
                        PathBuf::from_str("../Cargo.lock")?.canonicalize()?,
                        PathBuf::from_str("../Cargo.toml")?.canonicalize()?,
                    ],
                    local.sources
                );
            }
        } else {
            return Err(eyre!("Failed loading testing config"));
        }
        Ok(())
    }

    #[test]
    fn tilde_with_glob() -> Result<(), Report> {
        if let Ok(home) = std::env::var("HOME") {
            if let Ok(config) = DTConfig::from_pathbuf(PathBuf::from_str(
                "../testroot/configs/expand_tilde_with_glob.toml",
            )?) {
                let entries = std::fs::read_dir(&home)?
                    .map(|x| x.expect("Failed reading dir entry"))
                    .map(|x| {
                        x.path().canonicalize().expect(&format!(
                            "Failed canonicalizing path {}",
                            x.path().display(),
                        ))
                    })
                    .collect::<Vec<_>>();
                for local in &config.local {
                    assert_eq!(entries.len(), local.sources.len());
                    for s in &local.sources {
                        assert!(entries.contains(s));
                    }
                }
                Ok(())
            } else {
                Err(eyre!(
                    "This config should be loaded because target is a directory"
                ))
            }
        } else {
            Err(eyre!(
                "Set the `HOME` environment variable to complete this test"
            ))
        }
    }

    #[test]
    fn basedir() -> Result<(), Report> {
        if let Ok(config) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/basedir.toml",
        )?) {
            for group in config.local {
                assert_eq!(
                    group.sources,
                    vec![
                        PathBuf::from_str("../Cargo.lock")?.canonicalize()?,
                        PathBuf::from_str("../Cargo.toml")?.canonicalize()?,
                    ]
                )
            }
        } else {
            return Err(eyre!("Failed loading testing config"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod ignored_patterns {
    use color_eyre::{eyre::eyre, Report};
    use std::path::PathBuf;
    use std::str::FromStr;

    use super::DTConfig;

    #[test]
    fn empty_ignored_array() -> Result<(), Report> {
        if let Ok(config) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/empty_ignored_array.toml",
        )?) {
            for group in &config.local {
                let expected_sources =
                    vec![PathBuf::from_str("../testroot/README.md")?
                        .canonicalize()?];
                assert_eq!(group.sources, expected_sources);
                assert_eq!(
                    group.target,
                    PathBuf::from_str(".")?.canonicalize()?,
                );
                assert_eq!(group.ignored, Some(Vec::<String>::new()));
            }
        } else {
            return Err(eyre!("Failed loading testing config"));
        }

        Ok(())
    }

    #[test]
    fn empty_source_array() -> Result<(), Report> {
        if let Ok(config) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/empty_source_array.toml",
        )?) {
            for group in &config.local {
                let expected_sources: Vec<PathBuf> = vec![];
                assert_eq!(group.sources, expected_sources);
                assert_eq!(
                    group.target,
                    PathBuf::from_str(".")?.canonicalize()?,
                );
                assert_eq!(group.ignored, Some(vec!["README.md".to_owned()]));
            }
        } else {
            return Err(eyre!("Failed loading testing config"));
        }

        Ok(())
    }

    #[test]
    fn partial_filename() -> Result<(), Report> {
        if let Ok(config) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/partial_filename.toml",
        )?) {
            for group in &config.local {
                let expected_sources = vec![
                    PathBuf::from_str("../Cargo.lock")?.canonicalize()?,
                    PathBuf::from_str("../Cargo.toml")?.canonicalize()?,
                ];
                assert_eq!(group.sources, expected_sources);
                assert_eq!(
                    group.target,
                    PathBuf::from_str(".")?.canonicalize()?
                );
                assert_eq!(group.ignored, Some(vec![".lock".to_owned()]));
            }
        } else {
            return Err(eyre!("Failed loading testing config"));
        }

        Ok(())
    }

    #[test]
    fn regular_ignore() -> Result<(), Report> {
        if let Ok(config) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/regular_ignore.toml",
        )?) {
            for group in &config.local {
                let expected_sources =
                    vec![PathBuf::from_str("../Cargo.lock")?.canonicalize()?];
                assert_eq!(group.sources, expected_sources);
                assert_eq!(
                    group.target,
                    PathBuf::from_str(".")?.canonicalize()?,
                );
                assert_eq!(
                    group.ignored,
                    Some(vec!["Cargo.toml".to_owned()])
                );
            }
        } else {
            return Err(eyre!("Failed loading testing config"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod overriding_global_config {
    use std::{path::PathBuf, str::FromStr};

    use color_eyre::{eyre::eyre, Report};

    use super::{DTConfig, SyncMethod};

    #[test]
    fn overriding_allow_overwrite_no_global() -> Result<(), Report> {
        if let Ok(config) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/overriding_allow_overwrite_no_global.toml",
        )?) {
            for local in config.local {
                assert_eq!(
                    local.get_allow_overwrite(
                        &config.global.to_owned().unwrap_or_default()
                    ),
                    true,
                );
            }
        } else {
            return Err(eyre!("Failed loading testing config"));
        }

        Ok(())
    }

    #[test]
    fn overriding_allow_overwrite_with_global() -> Result<(), Report> {
        if let Ok(config) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/overriding_allow_overwrite_with_global.toml",
        )?) {
            for local in config.local {
                assert_eq!(
                    local.get_allow_overwrite(
                        &config.global.to_owned().unwrap_or_default()
                    ),
                    false,
                );
            }
        } else {
            return Err(eyre!("Failed loading testing config"));
        }

        Ok(())
    }

    #[test]
    fn overriding_method_no_global() -> Result<(), Report> {
        if let Ok(config) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/overriding_method_no_global.toml",
        )?) {
            for local in config.local {
                assert_eq!(
                    local.get_method(
                        &config.global.to_owned().unwrap_or_default(),
                    ),
                    SyncMethod::Copy,
                )
            }
        } else {
            return Err(eyre!("Failed loading testing config"));
        }

        Ok(())
    }

    #[test]
    fn overriding_method_with_global() -> Result<(), Report> {
        if let Ok(config) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/overriding_method_with_global.toml",
        )?) {
            for local in config.local {
                assert_eq!(
                    local.get_method(
                        &config.global.to_owned().unwrap_or_default(),
                    ),
                    SyncMethod::Symlink,
                )
            }
        } else {
            return Err(eyre!("Failed loading testing config"));
        }

        Ok(())
    }

    #[test]
    fn overriding_both_allow_overwrite_and_method_no_global(
    ) -> Result<(), Report> {
        if let Ok(config) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/overriding_both_allow_overwrite_and_method_no_global.toml",
        )?) {
            for local in config.local {
                assert_eq!(
                    local.get_method(
                        &config.global.to_owned().unwrap_or_default()
                    ),
                    SyncMethod::Copy,
                );
                assert_eq!(
                    local.get_allow_overwrite(
                        &config.global.to_owned().unwrap_or_default()
                    ),
                    true,
                );
            }
        } else {
            return Err(eyre!("Failed loading testing config"));
        }
        Ok(())
    }

    #[test]
    fn overriding_both_allow_overwrite_and_method_with_global(
    ) -> Result<(), Report> {
        if let Ok(config) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/overriding_both_allow_overwrite_and_method_with_global.toml",
        )?) {
            for local in config.local {
                assert_eq!(
                    local.get_method(
                        &config.global.to_owned().unwrap_or_default()
                    ),
                    SyncMethod::Symlink,
                );
                assert_eq!(
                    local.get_allow_overwrite(
                        &config.global.to_owned().unwrap_or_default()
                    ),
                    false,
                );
            }
        } else {
            return Err(eyre!("Failed loading testing config"));
        }

        Ok(())
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 21 2021, 01:14 [CST]
