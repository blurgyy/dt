use std::{
    panic,
    path::{Path, PathBuf},
    str::FromStr,
};

use regex::Regex;
use serde::Deserialize;
use serde_regex;
use serde_tuple::Deserialize_tuple;

use crate::error::{Error as AppError, Result};

/// Fallback value for config key [`hostname_sep`]
///
/// [`hostname_sep`]: GlobalConfig::hostname_sep
pub const DEFAULT_HOSTNAME_SEPARATOR: &str = "@@";
/// Fallback value for config key [`allow_overwrite`]
///
/// [`allow_overwrite`]: GlobalConfig::allow_overwrite
pub const DEFAULT_ALLOW_OVERWRITE: bool = false;

/// The configuration object constructed from configuration file.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct DTConfig {
    /// (Optional) Sets fallback behaviours.
    pub global: Option<GlobalConfig>,

    /// Local items groups.
    pub local: Vec<LocalGroup>,
}

impl FromStr for DTConfig {
    type Err = AppError;

    /// Loads configuration from string.
    fn from_str(s: &str) -> Result<Self> {
        let mut ret = toml::from_str::<Self>(s)?;
        ret.expand_tilde();
        ret.validate()
    }
}

impl DTConfig {
    /// Loads configuration from a file.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let confstr = std::fs::read_to_string(path).unwrap_or_else(|_| {
            panic!("Could not load config from '{}'", path.display())
        });
        Self::from_str(&confstr)
    }

    /// Construct another [`DTConfig`] object with only groups with matched
    /// names remaining.
    pub fn filter_names(self, group_names: Vec<String>) -> Self {
        Self {
            global: self.global,
            local: self
                .local
                .iter()
                .filter(|l| group_names.iter().any(|n| l.name == *n))
                .map(|l| l.to_owned())
                .collect(),
        }
    }

    /// Validates config object **without** touching the filesystem.
    fn validate(self) -> Result<Self> {
        let mut group_name_rec: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for group in &self.local {
            // Empty group name
            if group.name.is_empty() {
                return Err(AppError::ConfigError(
                    "empty group name".to_owned(),
                ));
            }
            // Empty basedir
            if group.basedir.to_str().unwrap().is_empty() {
                return Err(AppError::ConfigError(format!(
                    "empty basedir in group '{}'",
                    group.name,
                )));
            }
            // Empty target
            if group.target.to_str().unwrap().is_empty() {
                return Err(AppError::ConfigError(format!(
                    "empty target in group '{}'",
                    group.name,
                )));
            }

            // Duplicated group name
            if group_name_rec.get(&group.name).is_some() {
                return Err(AppError::ConfigError(format!(
                    "duplicated local group name '{}'",
                    group.name,
                )));
            }
            group_name_rec.insert(group.name.to_owned());

            // Slash in group name
            if group.name.contains('/') {
                return Err(AppError::ConfigError(format!(
                    "group name '{}' contains the '/' character",
                    group.name,
                )));
            }

            // Target and basedir are the same
            if group.basedir == group.target {
                return Err(AppError::ConfigError(format!(
                    "base directory and its target are the same in group '{}'",
                    group.name,
                )));
            }

            // basedir contains hostname_sep
            let hostname_sep = group.get_hostname_sep(
                &self.global.to_owned().unwrap_or_default(),
            );
            if group.basedir.to_str().unwrap().contains(&hostname_sep) {
                return Err(AppError::ConfigError(format!(
                    "base directory contains hostname_sep ({}) in group '{}'",
                    hostname_sep, group.name,
                )));
            }

            // Source item referencing parent
            if group.sources.iter().any(|s| s.starts_with("../")) {
                return Err(AppError::ConfigError(format!(
                    "source item references parent directory in group '{}'",
                    group.name,
                )));
            }

            // Source item is absolute
            if group
                .sources
                .iter()
                .any(|s| s.starts_with("/") || s.starts_with("~"))
            {
                return Err(AppError::ConfigError(format!(
                    "source array contains absolute path in group '{}'",
                    group.name,
                )));
            }

            // Source item contains bad globbing pattern
            if group.sources.iter().any(|s| {
                s.to_str()
                    .unwrap()
                    .split('/')
                    .any(|component| component == ".*")
            }) {
                log::error!(
                    "'.*' is prohibited for globbing sources because it also matches the parent directory.",
                );
                log::error!(
                    "If you want to match all items that starts with a dot, use ['.[!.]*', '..?*'] as sources.",
                );
                return Err(AppError::ConfigError(
                    "bad globbing pattern".to_owned(),
                ));
            }

            // Source item contains hostname_sep
            if group.sources.iter().any(|s| {
                let s = s.to_str().unwrap();
                s.contains(&hostname_sep)
            }) {
                return Err(AppError::ConfigError(format!(
                    "a source item contains hostname_sep ({}) in group '{}'",
                    hostname_sep, group.name,
                )));
            }

            if group.ignored.is_some() {
                todo!("`ignored` array works poorly and I decided to implement it in the future");
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
                .unwrap_or_else(|_| {
                    panic!(
                        "Failed expanding tilde in `global.staging` ({})",
                        staging.display(),
                    )
                });
            }
        }

        // Expand tilde in fields of `local`
        for ref mut group in &mut self.local {
            // `local.basedir`
            group.basedir = PathBuf::from_str(&shellexpand::tilde(
                group.basedir.to_str().unwrap(),
            ))
            .unwrap_or_else(|_| {
                panic!(
                    "Failed expanding tilde in `local.basedir` '{}'",
                    group.basedir.display(),
                )
            });

            // `local.target`
            group.target = PathBuf::from_str(&shellexpand::tilde(
                group.target.to_str().unwrap(),
            ))
            .unwrap_or_else(|_| {
                panic!(
                    "Failed expanding tilde in `local.target` '{}'",
                    group.target.display(),
                )
            });
        }
    }
}

/// Scope of a group, used to resolve _priority_ of possibly duplicated items,
/// to ensure every target path is pointed from only one source item.
///
/// The order of priority is:
///
/// [`Dropin`] > [`App`] > [`General`]
///
/// Within the same scope, the first defined group in the config file for DT
/// has the highest priority, later defined groups have lower priority.
///
/// Groups without a given scope are treated as of [`General`] scope.
///
/// [`Dropin`]: DTScope::Dropin
/// [`App`]: DTScope::App
/// [`General`]: DTScope::General
///
/// # Example
///
/// When you want to populate all your config files for apps that follows [the
/// XDG standard], you might write a config file for DT that looks like this:
///
/// [the XDG standard]: https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html
///
/// ```toml
/// [[local]]
/// name = "xdg_config_home"
/// basedir = "/path/to/your/xdg/config/directory"
/// sources = ["*"]
/// target = "~/.config"
/// ```
///
/// Let's say after some weeks or months, you have decided to also include
/// `/usr/share/fontconfig/conf.avail/10-sub-pixel-rgb.conf` to your
/// fontconfig directory, which is `~/.config/fontconfig/conf.d`, you do so by
/// adding another `[[local]]` group into your config file for DT:
///
/// ```toml
/// [[local]]
/// name = "fontconfig-system"
/// basedir = "/usr/share/fontconfig/conf.avail"
/// sources = ["10-sub-pixel-rgb.conf"]
/// target = "~/.config/fontconfig/conf.d"
/// ```
///
/// A problem arises when you also maintain a version of
/// `10-sub-pixel-rgb.conf` of your own: If DT syncs the `fontconfig-system`
/// group last, the resulting config file in your `$XDG_CONFIG_HOME` is the
/// system version;  While if DT syncs the `xdg_config_home` group last, that
/// file ended up being your previously maintained version.
///
/// Actually, DT is quite predictable: it only performs operations in the
/// order defined in the config file for your groups.  By defining the
/// `fontconfig-system` group last, you can completely avoid the ambiguity
/// above.
///
/// However, since the config file was written by you, a human, and humans are
/// notorious for making mistakes, it would be great if DT could always know
/// what to do when duplicated items are discovered in the config file.
/// Instead of putting the groups with higher priority at the end of your
/// config file, you could simply define `scope`s in their definitions:
///
/// ```toml
/// [[local]]
/// name = "fontconfig-system"
/// scope = "Dropin"
/// ...
/// [[local]]
/// name = "xdg_config_home"
/// scope = "General"
/// ...
/// ```
///
/// Now, with the `scope` being set, DT will first remove the source item
/// `10-sub-pixel-rgb.conf` (if it exists) from group `xdg_config_home`, then
/// perform its syncing process.
///
/// This is also useful with `dt-cli`'s `-l|--local-name` option, which gives
/// you more granular control over how items are synced.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum DTScope {
    /// The scope with lowest priority, this is the default scope,
    /// recommended for directories that contains config files for many
    /// un-categorized applications.
    General,

    /// The scope for a specific app, it's priority is higher than
    /// [`General`] while lower than [`Dropin`].
    ///
    /// [`General`]: DTScope::General
    /// [`Dropin`]: DTScope::Dropin
    App,

    /// The scope for drop-in replacements, it has the highest priority.
    Dropin,
}

impl Default for DTScope {
    fn default() -> Self {
        DTScope::General
    }
}

impl<'a> Default for &'a DTScope {
    fn default() -> &'a DTScope {
        &DTScope::General
    }
}

/// A single enaming rule, used for configuring differente names between
/// source items and their target.
#[derive(Clone, Debug, Deserialize_tuple)]
pub struct RenamingRule {
    /// A regular expression, specifies the pattern which item names are
    /// matched against.  Regular expression's capture groups (named or not)
    /// are supported.  See the [documentation] for more instructions on
    /// this.
    ///
    /// [documentation]: https://dt.cli.rs/
    #[serde(deserialize_with = "serde_regex::deserialize")]
    pub pattern: Regex,

    /// The substitution rule to apply if pattern matches an item,
    /// indexed/named capture groups are allowed.
    pub substitution: String,
}

/// Configures how local items (files/directories) are synced.
#[derive(Default, Clone, Deserialize, Debug)]
pub struct LocalGroup {
    /// Name of this group, used as namespace in staging root directory.
    pub name: String,

    /// The priority of this group, used to resolve possibly duplicated
    /// items.  See [`DTScope`] for details.
    ///
    /// [`DTScope`]: DTScope
    pub scope: Option<DTScope>,

    /// The base directory of all source items.  This simplifies
    /// configuration files with common prefixes in the [`sources`]
    /// array.
    ///
    /// [`sources`]: LocalGroup::sources
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
    /// It will only sync `src/main.rs` to the configured target directory
    /// (in this case, the directory where [DT] is being executed).
    ///
    /// [DT]: https://github.com/blurgyy/dt
    pub basedir: PathBuf,

    /// Paths (relative to [`basedir`]) to the items to be synced.
    ///
    /// [`basedir`]: LocalGroup::basedir
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
    /// will sync `/source/file` to `/tar/get/file` (creating non-existing
    /// directories along the way), while
    ///
    /// ```toml
    /// source = ["/source/dir"]
    /// target = "/tar/get/dir"
    /// ```
    ///
    /// will sync `source/dir` to `/tar/get/dir/dir` (creating non-existing
    /// directories along the way).
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
    /// With this setting, all files or directories with their basename as
    /// ".git" will be skipped.
    ///
    /// Cannot contain slash in any of the patterns.
    pub ignored: Option<Vec<String>>,

    /// (Optional) Separator for per-host settings, default to `@@`.
    ///
    /// An additional item with `${hostname_sep}$(hostname)` appended to the
    /// original item name will be checked first, before looking for the
    /// original item.  If the appended item is found, use this item
    /// instead of the configured one.
    ///
    /// Also ignores items that are meant for other hosts by checking if the
    /// string after [`hostname_sep`] matches current machine's hostname.
    ///
    /// [`hostname_sep`]: LocalGroup::hostname_sep
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
    /// On a machine with hostname set to `watson`, the below configuration
    /// (extraneous keys are omitted here)
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
    /// Where `/tmp/sshconfig/config` mirrors the content of
    /// `~/.ssh/config@watson`.
    pub hostname_sep: Option<String>,

    /// (Optional) Whether to allow overwriting existing files.  Dead
    /// symlinks are treated as non-existing, and are always overwrited
    /// (regardless of this option).
    pub allow_overwrite: Option<bool>,

    /// (Optional) Syncing method, overrides [`global.method`] key.
    ///
    /// [`global.method`]: GlobalConfig::method
    pub method: Option<SyncMethod>,

    /// (Optional) Renaming rules, appends to [`global.rename`].
    ///
    /// [`global.rename`]: GlobalConfig::rename
    pub rename: Option<Vec<RenamingRule>>,
}

impl LocalGroup {
    /// Gets the [`allow_overwrite`] key from a `LocalGroup` object,
    /// falls back to the `allow_overwrite` from provided global config.
    ///
    /// [`allow_overwrite`]: LocalGroup::allow_overwrite
    pub fn get_allow_overwrite(&self, global_config: &GlobalConfig) -> bool {
        match self.allow_overwrite {
            Some(allow_overwrite) => allow_overwrite,
            _ => global_config
                .allow_overwrite
                .unwrap_or(DEFAULT_ALLOW_OVERWRITE),
        }
    }

    /// Gets the [`method`] key from a `LocalGroup` object, falls back
    /// to the `method` from provided global config.
    ///
    /// [`method`]: LocalGroup::method
    pub fn get_method(&self, global_config: &GlobalConfig) -> SyncMethod {
        match self.method {
            Some(method) => method,
            _ => global_config.method.unwrap_or_default(),
        }
    }

    /// Gets the [`hostname_sep`] key from a `LocalGroup` object, falls
    /// back to the [`hostname_sep`] from provided global config.
    ///
    /// [`hostname_sep`]: LocalGroup::hostname_sep
    pub fn get_hostname_sep(&self, global_config: &GlobalConfig) -> String {
        match &self.hostname_sep {
            Some(hostname_sep) => hostname_sep.to_owned(),
            _ => global_config
                .hostname_sep
                .to_owned()
                .unwrap_or_else(|| DEFAULT_HOSTNAME_SEPARATOR.to_owned()),
        }
    }

    /// Gets the list of [renaming rules] of this group, which is an array
    /// of (REGEX, SUBSTITUTION) tuples composed of [`global.rename`] and
    /// [`local.rename`], used in [`DTItem::make_target`] to rename the item.
    ///
    /// [renaming rules]: LocalGroup::rename
    /// [`global.rename`]: GlobalConfig::rename
    /// [`local.rename`]: LocalGroup::rename
    /// [`DTItem::make_target`]: crate::item::DTItem::make_target
    pub fn get_renaming_rules(
        &self,
        global_config: &GlobalConfig,
    ) -> Vec<RenamingRule> {
        let mut ret: Vec<RenamingRule> = Vec::new();
        if let Some(ref global_renaming_rules) = global_config.rename {
            for rs in global_renaming_rules {
                ret.push(rs.to_owned());
            }
        }
        if let Some(ref group_renaming_rules) = self.rename {
            for rs in group_renaming_rules {
                ret.push(rs.to_owned());
            }
        }
        ret
    }
}

/// Configures default behaviours.
#[derive(Clone, Debug, Deserialize)]
pub struct GlobalConfig {
    /// The staging root directory.
    ///
    /// Only works when [`method`] (see below) is set to [`Symlink`].  When
    /// syncing with [`Symlink`] method, items will be copied to their
    /// staging directory (composed by joining staging root
    /// directory with their group name), then symlinked (as of `ln -sf`)
    /// from their staging directory to the target directory.
    ///
    /// Default to `$XDG_CACHE_HOME/dt/staging` if `XDG_CACHE_HOME` is set,
    /// or `$HOME/.cache/dt/staging` if `HOME` is set.  Panics when
    /// neither `XDG_CACHE_HOME` nor `HOME` is set and config file does
    /// not specify this.
    ///
    /// [`method`]: GlobalConfig::method
    /// [`Symlink`]: SyncMethod::Symlink
    pub staging: Option<PathBuf>,

    /// The syncing method.
    ///
    /// Available values are:
    ///
    /// - [`Copy`]
    /// - [`Symlink`]
    ///
    /// When [`method`] is [`Copy`], the above [`staging`] setting will be
    /// disabled.
    ///
    /// [`method`]: GlobalConfig::method
    /// [`staging`]: GlobalConfig::staging
    /// [`Copy`]: SyncMethod::Copy
    /// [`Symlink`]: SyncMethod::Symlink
    pub method: Option<SyncMethod>,

    /// Whether to allow overwriting existing files.
    ///
    /// This alters syncing behaviours when the target file exists.  If set
    /// to `true`, no errors/warnings will be omitted when the target
    /// file exists; otherwise reports error and skips the existing item.
    /// Using dry run to spot the existing files before syncing is
    /// recommended.
    pub allow_overwrite: Option<bool>,

    /// The hostname separator.
    ///
    /// Specifies default value when [`LocalGroup::hostname_sep`] is not set.
    ///
    /// [`LocalGroup::hostname_sep`]: LocalGroup::hostname_sep
    pub hostname_sep: Option<String>,

    /// Global item renaming rules.
    ///
    /// Rules defined here will be prepended to renaming rules of each group.
    /// See [`LocalGroup::rename`].
    ///
    /// [`LocalGroup::rename`]: LocalGroup::rename
    pub rename: Option<Vec<RenamingRule>>,
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
            allow_overwrite: Some(DEFAULT_ALLOW_OVERWRITE),
            hostname_sep: Some(DEFAULT_HOSTNAME_SEPARATOR.to_owned()),
            rename: None,
        }
    }
}

/// Syncing methods.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub enum SyncMethod {
    /// Instructs syncing module to directly copy each item from source to
    /// target.
    Copy,

    /// Instructs syncing module to first copy iach item from source to its
    /// staging directory, then symlink staged items from their staging
    /// directory to target.
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
    use pretty_assertions::assert_eq;

    use super::{DTConfig, SyncMethod};

    #[test]
    fn allow_overwrite_no_global() -> Result<(), Report> {
        let config = DTConfig::from_path(PathBuf::from_str(
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
        let config = DTConfig::from_path(PathBuf::from_str(
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
        let config = DTConfig::from_path(PathBuf::from_str(
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
        let config = DTConfig::from_path(PathBuf::from_str(
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
        let config= DTConfig::from_path(PathBuf::from_str(
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
        let config= DTConfig::from_path(PathBuf::from_str(
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
        let config = DTConfig::from_path(PathBuf::from_str(
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
        let config = DTConfig::from_path(PathBuf::from_str(
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
    use pretty_assertions::assert_eq;

    use super::DTConfig;

    #[test]
    fn all() -> Result<(), Report> {
        let config = DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/config/tilde_expansion-all.toml",
        )?)?;
        assert_eq!(config.global.unwrap().staging, dirs::home_dir());
        config.local.iter().all(|group| {
            assert_eq!(Some(group.to_owned().basedir), dirs::home_dir());
            assert_eq!(
                group.to_owned().target,
                dirs::home_dir()
                    .unwrap_or_else(|| panic!("Cannot determine home dir"))
                    .join("dt")
                    .join("target"),
            );
            true
        });
        Ok(())
    }
}

#[cfg(test)]
mod validation {
    use std::{path::PathBuf, str::FromStr};

    use color_eyre::{eyre::eyre, Report};
    use pretty_assertions::assert_eq;

    use super::DTConfig;
    use crate::error::Error as AppError;

    #[test]
    fn empty_group_name() -> Result<(), Report> {
        if let Err(err) = DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/config/validation-empty_group_name.toml",
        )?) {
            assert_eq!(
                err,
                AppError::ConfigError("empty group name".to_owned()),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because a group's name is empty"))
        }
    }

    #[test]
    fn empty_basedir() -> Result<(), Report> {
        if let Err(err) = DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/config/validation-empty_basedir.toml",
        )?) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "empty basedir in group 'empty basedir'".to_owned(),
                ),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because a group's basedir is empty"))
        }
    }

    #[test]
    fn empty_target() -> Result<(), Report> {
        if let Err(err) = DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/config/validation-empty_target.toml",
        )?) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "empty target in group 'empty target'".to_owned(),
                ),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because a group's basedir is empty"))
        }
    }

    #[test]
    fn same_names_in_multiple_local_groups() -> Result<(), Report> {
        if let Err(err) = DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/config/validation-same_names_in_multiple_locals.toml",
        )?) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "duplicated local group name 'wubba lubba dub dub'"
                        .to_owned()
                ),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because there are multiple local groups share the same name"))
        }
    }

    #[test]
    fn slash_in_group_name() -> Result<(), Report> {
        if let Err(err) = DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/config/validation-slash_in_group_name.toml",
        )?) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "group name 'this/group/name/contains/slash' contains the '/' character"
                        .to_owned()
                ),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because a group name contains slash"))
        }
    }

    #[test]
    fn basedir_is_target() -> Result<(), Report> {
        if let Err(err) = DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/syncing/invalid_configs-basedir_is_target.toml",
        )?) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "base directory and its target are the same in group 'basedir is target'"
                        .to_owned(),
                ),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because basedir and target are the same"))
        }
    }

    #[test]
    fn basedir_contains_hostname_sep() -> Result<(), Report> {
        if let Err(err) = DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/config/validation-basedir_contains_hostname_sep.toml",
        )?) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "base directory contains hostname_sep (@@) in group 'basedir contains hostname_sep'"
                        .to_owned(),
                ),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because a basedir contains hostname_sep"))
        }
    }

    #[test]
    fn source_item_referencing_parent() -> Result<(), Report> {
        if let Err(err) = DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/config/validation-source_item_referencing_parent.toml",
        )?) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "source item references parent directory in group 'source item references parent dir'"
                        .to_owned(),
                ),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because a source item references parent directory"))
        }
    }

    #[test]
    fn source_item_is_absolute() -> Result<(), Report> {
        if let Err(err) = DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/config/validation-source_item_is_absolute.toml",
        )?) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "source array contains absolute path in group 'source item is absolute'"
                        .to_owned(),
                ),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because a source item is an absolute path"))
        }
    }

    #[test]
    fn except_dot_asterisk_glob() -> Result<(), Report> {
        if let Err(err) = DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/config/validation-except_dot_asterisk_glob.toml",
        )?) {
            assert_eq!(
                err,
                AppError::ConfigError("bad globbing pattern".to_owned()),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because it contains bad globs (.* and /.*)"))
        }
    }

    #[test]
    fn source_item_contains_hostname_sep() -> Result<(), Report> {
        if let Err(err) = DTConfig::from_path(PathBuf::from_str(
            "../testroot/configs/config/validation-source_item_contains_hostname_sep.toml",
        )?) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "a source item contains hostname_sep (@@) in group '@@ in source item'"
                        .to_owned()
                ),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because a source item contains hostname_sep"))
        }
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 21 2021, 01:14 [CST]
