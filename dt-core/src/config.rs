use std::{path::PathBuf, str::FromStr};

use color_eyre::{eyre::eyre, Report};
use serde::Deserialize;

/// The configuration object constructed from configuration file.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct DTConfig {
    /// Optional, sets fallback behaviours.
    pub global: Option<GlobalConfig>,
    pub local: Vec<LocalSyncConfig>,
}

/// Configures how local items (files/directories) are synced.
///
/// Each item should satisfy one of the following:
///     - is relative to the path from which the executable is being run
///     - is an absolute path
#[derive(Default, Clone, Deserialize, Debug)]
pub struct LocalSyncConfig {
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
    pub basedir: Option<PathBuf>,

    /// Paths to the items to be synced.
    pub sources: Vec<PathBuf>,

    /// The parent dir of the final synced items.
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

    /// (Optional) Ignored patterns.
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
    // // The pattern specified in `match_begin` is matched against all
    // match_begin: String,
    // replace_begin: String,
    // match_end: String,
    // replace_end: String,
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
        ret.validate()?;
        ret.expand()
    }

    fn validate(self: &Self) -> Result<(), Report> {
        for group in &self.local {
            if group.target.exists() && !group.target.is_dir() {
                return Err(eyre!("Target path exists and not a directory"));
            }
            for i in &group.ignored {
                if i.contains(&"/".to_owned()) {
                    return Err(eyre!(
                        "Ignored pattern contains slash, this is not allowed"
                    ));
                }
            }
        }
        if let Some(global) = &self.global {
            if global.method == SyncMethod::Symlink {
                todo!("Syncing with symlinks is not implemented");
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
                basedir: None, // basedir is expanded and removed.
                sources: vec![],
                target: PathBuf::from_str(&shellexpand::tilde(
                    original.target.to_str().unwrap(),
                ))?,
                ignored: original.ignored.to_owned(),
            };
            for s in &original.sources {
                let s = if let Some(basedir) = &original.basedir {
                    basedir.join(s)
                } else {
                    s.to_owned()
                };
                let s = shellexpand::tilde(s.to_str().unwrap());
                let mut s = glob::glob_with(&s, globbing_options)?
                    .map(|x| {
                        x.expect(&format!(
                            "Failed globbing source path {}",
                            &s
                        ))
                    })
                    .filter(|x| {
                        if let Some(ignored) = &next.ignored {
                            if ignored.len() == 0 {
                                true
                            } else {
                                ignored.iter().any(|y| {
                                    x.iter().all(|z| z.to_str().unwrap() != y)
                                })
                            }
                        } else {
                            true
                        }
                    })
                    .collect();
                next.sources.append(&mut s);
            }
            ret.local.push(next);
        }

        Ok(ret)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GlobalConfig {
    /// The staging directory.
    ///
    /// Only works when `method` (see below) is set to `Symlink`.  When syncing with `Symlink`
    /// method, items will be copied to the staging directory, then symlinked (as of `ln -sf`) from
    /// the staging directory to the target directory.
    ///
    /// Default to `$XDG_CACHE_HOME/dt/staging` if XDG_CACHE_HOME is set, or
    /// `$HOME/.cache/dt/staging` if HOME is set.  Panics if neigther XDG_CACHE_HOME or HOME is set, and
    /// config file does not specify this.
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
    /// This alters syncing behaviours.
    pub allow_overwrite: bool,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        let default_staging: PathBuf;
        if let Ok(xdg_cache_home) = std::env::var("XDG_CACHE_HOME") {
            log::debug!(
                "Using environment variable XDG_CACHE_HOME to determine stating directory"
            );
            default_staging = PathBuf::from_str(&xdg_cache_home)
                .expect("Failed constructing default staging directory from xdg_cache_home")
                .join("dt")
                .join("staging");
        } else if let Ok(home) = std::env::var("HOME") {
            log::debug!(
                "Using environment variable HOME to determine stating directory"
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
            method: SyncMethod::Copy,
            allow_overwrite: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub enum SyncMethod {
    Copy,
    Symlink,
}

#[cfg(test)]
mod validating {
    use std::{path::PathBuf, str::FromStr};

    use color_eyre::{eyre::eyre, Report};

    use super::DTConfig;

    #[test]
    fn s_file_t_file() -> Result<(), Report> {
        if let Ok(config) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/s_file_t_file.toml",
        )?) {
            Err(eyre!(
                "This config should not be loaded because target is not a directory: {:#?}",
                config
            ))
        } else {
            Ok(())
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
        if let Ok(config) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/s_dir_t_file.toml",
        )?) {
            Err(eyre!(
                "This config should not be loaded because target is not a directory: {:#?}",
                config
            ))
        } else {
            Ok(())
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
    fn tilde() -> Result<(), Report> {
        if let Ok(home) = std::env::var("HOME") {
            if let Ok(config) = DTConfig::from_pathbuf(PathBuf::from_str(
                "../testroot/configs/expand_tilde.toml",
            )?) {
                for local in &config.local {
                    for s in &local.sources {
                        assert_eq!(s.to_str(), Some(home.as_str()));
                    }
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
                        PathBuf::from_str("../Cargo.lock")?,
                        PathBuf::from_str("../Cargo.toml")?,
                    ],
                    local.sources
                );
            }
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
                    .map(|x| x.path())
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
                        PathBuf::from_str("../Cargo.lock")?,
                        PathBuf::from_str("../Cargo.toml")?,
                    ]
                )
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod ignored_patterns {
    use color_eyre::Report;
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
                    vec![PathBuf::from_str("../testroot/README.md")?];
                assert_eq!(group.sources, expected_sources);
                assert_eq!(group.target, PathBuf::from_str(".")?);
                assert_eq!(group.ignored, Some(Vec::<String>::new()));
            }
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
                assert_eq!(group.target, PathBuf::from_str(".")?);
                assert_eq!(group.ignored, Some(vec!["README.md".to_owned()]));
            }
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
                    PathBuf::from_str("../Cargo.lock")?,
                    PathBuf::from_str("../Cargo.toml")?,
                ];
                assert_eq!(group.sources, expected_sources);
                assert_eq!(group.target, PathBuf::from_str(".")?);
                assert_eq!(group.ignored, Some(vec![".lock".to_owned()]));
            }
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
                    vec![PathBuf::from_str("../Cargo.lock")?];
                assert_eq!(group.sources, expected_sources);
                assert_eq!(group.target, PathBuf::from_str(".")?);
                assert_eq!(
                    group.ignored,
                    Some(vec!["Cargo.toml".to_owned()])
                );
            }
        }
        Ok(())
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 21 2021, 01:14 [CST]
