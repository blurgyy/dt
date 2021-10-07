use std::{panic, path::PathBuf, str::FromStr};

use color_eyre::{eyre::eyre, Report};
use serde::Deserialize;

pub const DEFAULT_HOSTNAME_SEPARATOR: &str = "@@";

/// The configuration object constructed from configuration file.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct DTConfig {
    /// (Optional) Sets fallback behaviours.
    pub global: Option<GlobalConfig>,

    /// Local items groups.
    pub local: Vec<LocalGroup>,
}

impl DTConfig {
    /// Loads configuration from a file.
    pub fn from_pathbuf(path: PathBuf) -> Result<Self, Report> {
        let confstr = std::fs::read_to_string(path)?;
        Self::from_str(&confstr)
    }

    /// Loads configuration from string.
    fn from_str(s: &str) -> Result<Self, Report> {
        let mut ret = toml::from_str::<Self>(s)?;
        ret.expand_tilde();

        ret.validate()
    }

    fn validate(self) -> Result<Self, Report> {
        for group in &self.local {
            if group.name.contains("/") {
                return Err(eyre!(
                    "Group name cannot include the '/' character"
                ));
            }
            if group.sources.iter().any(|s| s.starts_with("../")) {
                return Err(eyre!(
                    "Source item cannot reference parent directory",
                ));
            }
            if group.sources.iter().any(|s| s.starts_with("/")) {
                return Err(eyre!("Source item cannot be an absolute path"));
            }
            if group.sources.iter().any(|s| {
                let s = s.to_str().unwrap();
                s == ".*" || s.contains("/.*")
            }) {
                return Err(eyre!(
                    "Do not use globbing patterns like '.*', because it also matches current directory (.) and parent directory (..)"
                ));
            }
        }
        Ok(self)
    }

    fn expand_tilde(&mut self) {
        // Expand tilde in `global.staging`
        if let Some(ref mut global) = self.global {
            if let Some(ref mut staging) = global.staging {
                *staging = PathBuf::from_str(&shellexpand::tilde(
                    staging.to_str().unwrap(),
                ))
                .expect(&format!(
                    "Failed expanding tilde in `global.staging`: {}",
                    staging.display(),
                ));
            }
        }

        // Expand tilde in fields of `local`
        for ref mut group in &mut self.local {
            // `local.basedir`
            group.basedir = PathBuf::from_str(&shellexpand::tilde(
                group.basedir.to_str().unwrap(),
            ))
            .expect(&format!(
                "Failed expanding tilde in `local.basedir`: {}",
                group.basedir.display(),
            ));

            // `local.target`
            group.target = PathBuf::from_str(&shellexpand::tilde(
                group.target.to_str().unwrap(),
            ))
            .expect(&format!(
                "Failed expanding tilde in `local.target`: {}",
                group.target.display(),
            ));
        }
    }
}

/// Configures how local items (files/directories) are synced.
#[derive(Default, Clone, Deserialize, Debug)]
pub struct LocalGroup {
    /// Name of this group, used as namespace in staging root directory.
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

    /// The path of the parent dir of the final synced items.
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
    /// will sync "source/dir" to "/tar/get/dir/dir" (creating non-existing directories along the
    /// way).
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

    /// (Optional) Separator for per-host settings, default to `@@`.
    ///
    /// An additional item with `${hostname_sep}$(hostname)` appended to the original item name
    /// will be checked first, before looking for the original item.  If the appended item is found,
    /// use this item instead of the configured one.
    ///
    /// Also ignores items that are meant for other hosts by checking if the string after
    /// `hostname_sep` matches current machine's hostname.
    ///
    /// ## Example
    ///
    /// When the following directory structure exists:
    ///
    /// ```plain
    /// ~/.ssh/
    /// ├── authorized_keys
    /// ├── authorized_keys@@sherlock
    /// ├── authorized_keys@@watson
    /// ├── config
    /// ├── config@sherlock
    /// └── config@watson
    /// ```
    ///
    /// On a machine with hostname set to `watson`, the below configuration (extraneous keys are
    /// omitted here)
    ///
    /// ```toml [[local]]
    /// ...
    /// hostname_sep = "@@"
    ///
    /// basedir = "~/.ssh"
    /// sources = ["config"]
    /// target = "/tmp/sshconfig"
    /// ...
    /// ```
    ///
    /// will result in the below target (`/tmp/sshconfig`):
    ///
    /// ```plain
    /// /tmp/sshconfig/
    /// └── config
    /// ```
    ///
    /// Where `/tmp/sshconfig/config` mirrors the content of `~/.ssh/config@watson`.
    pub hostname_sep: Option<String>,

    /// (Optional) Whether to allow overwriting existing files.  Dead symlinks are treated as
    /// non-existing, and are always overwrited (regardless of this option).
    pub allow_overwrite: Option<bool>,

    /// (Optional) Syncing method, overrides `global.method` key.
    pub method: Option<SyncMethod>,
    // // The pattern specified in `match_begin` is matched against all
    // match_begin: String,
    // replace_begin: String,
    // match_end: String,
    // replace_end: String,
}

impl LocalGroup {
    /// Gets the `allow_overwrite` key from a `LocalSyncConfig` object, falls back to the
    /// `allow_overwrite` from provided global config.
    pub fn get_allow_overwrite(&self, global_config: &GlobalConfig) -> bool {
        match self.allow_overwrite {
            Some(allow_overwrite) => allow_overwrite,
            _ => global_config.allow_overwrite.unwrap_or_default(),
        }
    }

    /// Gets the `method` key from a `LocalSyncConfig` object, falls back to the `method` from
    /// provided global config.
    pub fn get_method(&self, global_config: &GlobalConfig) -> SyncMethod {
        match self.method {
            Some(method) => method,
            _ => global_config.method.unwrap_or_default(),
        }
    }

    /// Gets the `method` key from a `LocalSyncConfig` object, falls back to the `method` from
    /// provided global config.
    pub fn get_hostname_sep(&self, global_config: &GlobalConfig) -> String {
        match &self.hostname_sep {
            Some(hostname_sep) => hostname_sep.to_owned(),
            _ => global_config
                .hostname_sep
                .to_owned()
                .unwrap_or(DEFAULT_HOSTNAME_SEPARATOR.to_owned()),
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
    pub method: Option<SyncMethod>,

    /// Whether to allow overwriting existing files.
    ///
    /// This alters syncing behaviours when the target file exists.  If set to `true`, no
    /// errors/warnings will be omitted when the target file exists; otherwise reports error and
    /// skips the existing item.  Using dry run to spot the existing files before syncing is
    /// recommended.
    pub allow_overwrite: Option<bool>,

    /// Default value when `LocalSyncConfig::hostname_sep` is not set.
    pub hostname_sep: Option<String>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        let default_staging: PathBuf;
        if let Some(cache_dir) = dirs::cache_dir() {
            default_staging = cache_dir.join("dt").join("staging");
        } else {
            panic!("Cannot infer default staging directory, set either XDG_CACHE_HOME or HOME to solve this.");
        }
        GlobalConfig {
            staging: Some(default_staging),
            method: Some(SyncMethod::default()),
            allow_overwrite: Some(bool::default()),
            hostname_sep: Some(DEFAULT_HOSTNAME_SEPARATOR.to_owned()),
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
mod overriding_global_config {
    use std::{path::PathBuf, str::FromStr};

    use color_eyre::Report;

    use super::{DTConfig, SyncMethod};

    #[test]
    fn allow_overwrite_no_global() -> Result<(), Report> {
        let config = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/config/overriding_global_config-allow_overwrite_no_global.toml",
        )?)?;
        for group in config.local {
            assert_eq!(
                group.get_allow_overwrite(
                    &config.global.to_owned().unwrap_or_default()
                ),
                true,
            );
        }

        Ok(())
    }

    #[test]
    fn allow_overwrite_with_global() -> Result<(), Report> {
        let config = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/config/overriding_global_config-allow_overwrite_with_global.toml",
        )?)?;
        for group in config.local {
            assert_eq!(
                group.get_allow_overwrite(
                    &config.global.to_owned().unwrap_or_default()
                ),
                false,
            );
        }

        Ok(())
    }

    #[test]
    fn method_no_global() -> Result<(), Report> {
        let config = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/config/overriding_global_config-method_no_global.toml",
        )?)?;
        for group in config.local {
            assert_eq!(
                group.get_method(
                    &config.global.to_owned().unwrap_or_default(),
                ),
                SyncMethod::Copy,
            )
        }

        Ok(())
    }

    #[test]
    fn method_with_global() -> Result<(), Report> {
        let config = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/config/overriding_global_config-method_with_global.toml",
        )?)?;
        for group in config.local {
            assert_eq!(
                group.get_method(
                    &config.global.to_owned().unwrap_or_default(),
                ),
                SyncMethod::Symlink,
            )
        }

        Ok(())
    }

    #[test]
    fn both_allow_overwrite_and_method_no_global() -> Result<(), Report> {
        let config= DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/config/overriding_global_config-both_allow_overwrite_and_method_no_global.toml",
        )?)?;
        for group in config.local {
            assert_eq!(
                group.get_method(
                    &config.global.to_owned().unwrap_or_default()
                ),
                SyncMethod::Copy,
            );
            assert_eq!(
                group.get_allow_overwrite(
                    &config.global.to_owned().unwrap_or_default()
                ),
                true,
            );
        }

        Ok(())
    }

    #[test]
    fn both_allow_overwrite_and_method_with_global() -> Result<(), Report> {
        let config= DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/config/overriding_global_config-both_allow_overwrite_and_method_with_global.toml",
        )?)?;
        for group in config.local {
            assert_eq!(
                group.get_method(
                    &config.global.to_owned().unwrap_or_default()
                ),
                SyncMethod::Symlink,
            );
            assert_eq!(
                group.get_allow_overwrite(
                    &config.global.to_owned().unwrap_or_default()
                ),
                false,
            );
        }

        Ok(())
    }

    #[test]
    fn hostname_sep_no_global() -> Result<(), Report> {
        let config = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/config/overriding_global_config-hostname_sep_no_global.toml"
        )?)?;
        for group in config.local {
            assert_eq!(
                group.get_hostname_sep(
                    &config.global.to_owned().unwrap_or_default()
                ),
                "@-@",
            );
        }

        Ok(())
    }

    #[test]
    fn hostname_sep_with_global() -> Result<(), Report> {
        let config = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/config/overriding_global_config-hostname_sep_with_global.toml"
        )?)?;
        for group in config.local {
            assert_eq!(
                group.get_hostname_sep(
                    &config.global.to_owned().unwrap_or_default()
                ),
                "@-@",
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tilde_expansion {
    use std::{path::PathBuf, str::FromStr};

    use color_eyre::Report;

    use super::DTConfig;

    #[test]
    fn all() -> Result<(), Report> {
        let config = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/config/tilde_expansion-all.toml",
        )?)?;
        assert_eq!(config.global.unwrap().staging, dirs::home_dir());
        config.local.iter().all(|group| {
            assert_eq!(Some(group.to_owned().basedir), dirs::home_dir());
            assert_eq!(Some(group.to_owned().target), dirs::home_dir());
            true
        });

        Ok(())
    }
}

#[cfg(test)]
mod validation {
    use std::{path::PathBuf, str::FromStr};

    use color_eyre::{eyre::eyre, Report};

    use super::DTConfig;

    #[test]
    fn slash_in_group_name() -> Result<(), Report> {
        if let Err(msg) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/config/validation-slash_in_group_name.toml",
        )?) {
            assert_eq!(
                msg.to_string(),
                "Group name cannot include the '/' character",
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because a group name contains slash"))
        }
    }

    #[test]
    fn source_item_referencing_parent() -> Result<(), Report> {
        if let Err(msg) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/config/validation-source_item_referencing_parent.toml",
        )?) {
            assert_eq!(
                msg.to_string(),
                "Source item cannot reference parent directory",
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because a source item references parent directory"))
        }
    }

    #[test]
    fn source_item_is_absolute() -> Result<(), Report> {
        if let Err(msg) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/config/validation-source_item_is_absolute.toml",
        )?) {
            assert_eq!(
                msg.to_string(),
                "Source item cannot be an absolute path",
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because a source item is an absolute path"))
        }
    }

    #[test]
    fn except_dot_asterisk_glob() -> Result<(), Report> {
        if let Err(msg) = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/config/validation-except_dot_asterisk_glob.toml",
        )?) {
            assert_eq!(
                msg.to_string(),
                "Do not use globbing patterns like '.*', because it also matches current directory (.) and parent directory (..)",
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because it contains bad globs (.* and /.*)"))
        }
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 21 2021, 01:14 [CST]
