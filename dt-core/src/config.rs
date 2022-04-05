use std::{
    fmt::Display,
    path::{Path, PathBuf},
    rc::Rc,
    str::FromStr,
};

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_regex;
use serde_tuple::Deserialize_tuple;
use url::Url;

use crate::{
    error::{Error as AppError, Result},
    item::Operate,
};

/// Helper type for a group's [name]
///
/// [name]: Group::name
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct GroupName(pub PathBuf);
impl Display for GroupName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string_lossy())
    }
}
impl GroupName {
    /// Gets the first component of this name, components are separated by
    /// slashes.
    pub fn main(&self) -> String {
        let first_comp: PathBuf = self.0.components().take(1).collect();
        first_comp.to_string_lossy().to_string()
    }
    /// Checks if this name is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use dt_core::{config:: GroupName, error::Error as AppError};
    /// assert!(GroupName("a".into()).validate().is_ok());
    /// assert!(GroupName("a/b/c".into()).validate().is_ok());
    /// assert!(GroupName("/starts/with/slash".into()).validate().is_err());
    /// assert!(GroupName("relative/../path".into()).validate().is_err());
    /// # Ok::<(), AppError>(())
    /// ```
    pub fn validate(&self) -> Result<()> {
        if self
            .0
            .components()
            .any(|comp| comp.as_os_str().to_string_lossy() == "..")
        {
            Err(AppError::ConfigError(
                "Group name should not contain relative component".to_owned(),
            ))
        } else if self.0.starts_with("/") {
            Err(AppError::ConfigError(
                "Group name should not start with slash".to_owned(),
            ))
        } else if self.0 == PathBuf::from_str("").unwrap() {
            Err(AppError::ConfigError(
                "Group name should not be empty".to_owned(),
            ))
        } else {
            Ok(())
        }
    }
    /// Returns a PathBuf, which adds a [`subgroup_prefix`] to each of the
    /// components other than the main component.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::{str::FromStr, path::PathBuf};
    /// # use dt_core::config::GroupName;
    /// # use pretty_assertions::assert_eq;
    /// let gn = GroupName("gui/gtk".into());
    /// assert_eq!(
    ///     gn.with_subgroup_prefix("#"),
    ///     PathBuf::from_str("gui/#gtk").unwrap(),
    /// );
    /// ```
    ///
    /// [`subgroup_prefix`]: SubgroupPrefix
    pub fn with_subgroup_prefix(&self, subgroup_prefix: &str) -> PathBuf {
        PathBuf::from(self.main()).join(
            self.0
                .iter()
                .skip(1)
                .map(|comp| {
                    subgroup_prefix.to_owned() + &comp.to_string_lossy()
                })
                .collect::<PathBuf>(),
        )
    }
}
/// Helper type for config key [`staging`]
///
/// [`staging`]: GlobalConfig::staging
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct StagingPath(pub PathBuf);
impl Default for StagingPath {
    fn default() -> Self {
        if let Some(cache_dir) = dirs::data_dir() {
            Self(cache_dir.join("dt").join("staging"))
        } else {
            panic!("Cannot infer default staging directory, set either XDG_DATA_HOME or HOME to solve this.");
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
/// Helper type for config key [`subgroup_prefix`]
///
/// [`subgroup_prefix`]: GlobalConfig::subgroup_prefix
#[derive(Clone, Debug, Deserialize)]
pub struct SubgroupPrefix(pub String);
impl Default for SubgroupPrefix {
    fn default() -> Self {
        Self("#".to_owned())
    }
}
/// Helper type for config key [`allow_overwrite`]
///
/// [`allow_overwrite`]: GlobalConfig::allow_overwrite
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct AllowOverwrite(pub bool);
#[allow(clippy::derivable_impls)]
impl Default for AllowOverwrite {
    fn default() -> Self {
        Self(false)
    }
}
/// Helper type for config key [`ignore_failure`]
///
/// [`ignore_failure`]: GlobalConfig::ignore_failure
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct IgnoreFailure(pub bool);
#[allow(clippy::derivable_impls)]
impl Default for IgnoreFailure {
    fn default() -> Self {
        Self(false)
    }
}
/// Helper type for config key [`renderable`]
///
/// [`renderable`]: GlobalConfig::renderable
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Renderable(pub bool);
#[allow(clippy::derivable_impls)]
impl Default for Renderable {
    fn default() -> Self {
        Self(true)
    }
}
/// Helper type for config key [`hostname_sep`]
///
/// [`hostname_sep`]: GlobalConfig::hostname_sep
#[derive(Clone, Debug, Deserialize)]
pub struct HostnameSeparator(pub String);
impl Default for HostnameSeparator {
    fn default() -> Self {
        Self("@@".to_owned())
    }
}
/// Helper type for config key [`rename`]
///
/// [`rename`]: GlobalConfig::rename
#[derive(Clone, Debug, Deserialize)]
pub struct RenamingRules(pub Vec<RenamingRule>);
#[allow(clippy::derivable_impls)]
impl Default for RenamingRules {
    fn default() -> Self {
        Self(Vec::new())
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
/// has the highest priority, later defined groups have lower priorities.
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
/// base = "/path/to/your/xdg/config/directory"
/// sources = ["*"]
/// target = "~/.config"
/// ```
///
/// Let's say after some weeks or months, you have decided to also include
/// `/usr/share/fontconfig/conf.avail/11-lcdfilter-default.conf` to your
/// fontconfig directory, which is `~/.config/fontconfig/conf.d`, you do so by
/// adding another `[[local]]` group into your config file for DT:
///
/// ```toml
/// [[local]]
/// name = "fontconfig-system"
/// base = "/usr/share/fontconfig/conf.avail"
/// sources = ["11-lcdfilter-default.conf"]
/// target = "~/.config/fontconfig/conf.d"
/// ```
///
/// A problem arises when you also maintain a version of
/// `11-lcdfilter-default.conf` of your own: If DT syncs the
/// `fontconfig-system` group last, the resulting config file in your
/// `$XDG_CONFIG_HOME` is the system version;  While if DT syncs the
/// `xdg_config_home` group last, that file ended up being your previously
/// maintained version.
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
/// `11-lcdfilter-default.conf` (if it exists) from group `xdg_config_home`,
/// then perform its syncing process.
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

/// The configuration object deserialized from configuration file, every
/// field of it is optional.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct DTConfig {
    /// Sets fallback behaviours.
    pub global: GlobalConfig,

    /// Defines values for templating.
    pub context: ContextConfig,

    /// Groups containing local files.
    pub local: Vec<LocalGroup>,

    /// Groups containing remote files.
    pub remote: Vec<RemoteGroup>,
}

impl FromStr for DTConfig {
    type Err = AppError;

    /// Loads configuration from string.
    fn from_str(s: &str) -> Result<Self> {
        toml::from_str::<Self>(s)?.expand_tilde().validate()
    }
}

impl DTConfig {
    /// Loads configuration from a file.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Ok(confstr) = std::fs::read_to_string(path) {
            Self::from_str(&confstr)
        } else {
            Err(AppError::ConfigError(format!(
                "Could not load config from '{}'",
                path.display(),
            )))
        }
    }

    /// Construct another [`DTConfig`] object with groups that match given
    /// filters.  Groups are matched hierarchically, e.g. a filter `a/b` will
    /// select `a/b/c` and `a/b/d`, but not `a/bcd`.
    pub fn filter_names(self, group_names: Vec<String>) -> Self {
        Self {
            global: self.global,
            context: self.context,
            local: self
                .local
                .iter()
                .filter(|l| {
                    group_names.iter().any(|n| l.name.0.starts_with(n))
                })
                .map(|l| l.to_owned())
                .collect(),
            remote: self
                .remote
                .iter()
                .filter(|l| {
                    group_names.iter().any(|n| l.name.0.starts_with(n))
                })
                .map(|l| l.to_owned())
                .collect(),
        }
    }

    /// Validates config object.  After this, the original `global` and
    /// `context` sections are referenced by each group via an [Rc] and can be
    /// safely ignored in further processing.
    ///
    /// [Rc]: std::rc::Rc
    fn validate(self) -> Result<Self> {
        if !self.context.0.is_table() {
            return Err(AppError::ConfigError(
                "`context` is expected to be a table".to_owned(),
            ));
        }

        let global_ref = Rc::new(self.global.to_owned());
        let context_ref = Rc::new(self.context.to_owned());

        let mut ret: Self = self;

        for group in &mut ret.local {
            group.global = Rc::clone(&global_ref);
            group.context = Rc::clone(&context_ref);
            group.validate()?;
        }
        for group in &mut ret.remote {
            group.global = Rc::clone(&global_ref);
            group.context = Rc::clone(&context_ref);
            group.validate()?;
        }

        Ok(ret)
    }

    fn expand_tilde(self) -> Self {
        let mut ret = self;

        // Expand tilde in `global.staging`
        let staging = &mut ret.global.staging;
        *staging = if *staging == StagingPath("".into()) {
            log::warn!("Empty staging path is replaced to '.'");
            StagingPath(".".into())
        } else {
            StagingPath(
                PathBuf::from_str(&shellexpand::tilde(
                    &staging.0.to_string_lossy(),
                ))
                .unwrap(),
            )
        };

        // Expand tilde in `base` and `target` of `local`
        for group in &mut ret.local {
            // `local.base`
            group.base = if group.base == PathBuf::from_str("").unwrap() {
                log::warn!("[{}]: Empty base is replaced to '.'", group.name);
                ".".into()
            } else {
                PathBuf::from_str(&shellexpand::tilde(
                    &group.base.to_string_lossy(),
                ))
                .unwrap()
            };

            // `local.target`
            group.target = if group.target == PathBuf::from_str("").unwrap() {
                log::warn!(
                    "[{}]: Empty target is replaced to '.'",
                    group.name,
                );
                ".".into()
            } else {
                PathBuf::from_str(&shellexpand::tilde(
                    &group.target.to_string_lossy(),
                ))
                .unwrap()
            };
        }

        ret
    }
}

/// A single renaming rule, used for generating names for target files which
/// are different from their sources.
#[derive(Clone, Debug, Deserialize_tuple)]
pub struct RenamingRule {
    /// A regular expression, specifies the pattern against which item names
    /// are matched.  Regular expression's capture groups (indexed or named)
    /// are supported.  See the [documentation] for more instructions on
    /// this.
    ///
    /// [documentation]: https://dt.cli.rs/features/03-filename-manipulating.html
    #[serde(deserialize_with = "serde_regex::deserialize")]
    pub pattern: Regex,

    /// The substitution rule to apply if pattern matches an item,
    /// indexed/named capture groups are allowed.
    pub substitution: String,
}

/// Configures default behaviours.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct GlobalConfig {
    /// The staging root directory.
    ///
    /// Only works when [`method`] (see below) is set to [`Symlink`].  When
    /// syncing with [`Symlink`] method, items will be copied to their
    /// staging directory (composed by joining staging root
    /// directory with their group name), then symlinked (as of `ln -sf`)
    /// from their staging directory to the target directory.
    ///
    /// Default to `$XDG_DATA_HOME/dt/staging` if `XDG_DATA_HOME` is set,
    /// or `$HOME/.cache/dt/staging` if `HOME` is set.  Panics when
    /// neither `XDG_DATA_HOME` nor `HOME` is set and config file does
    /// not specify this.
    ///
    /// [`method`]: GlobalConfig::method
    /// [`Symlink`]: SyncMethod::Symlink
    #[serde(default)]
    pub staging: StagingPath,

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
    #[serde(default)]
    pub method: SyncMethod,

    /// A string to be prepended to a subgroup's name when creating its
    /// staging directory with the [`Symlink`] syncing method.
    ///
    /// [`Symlink`]: SyncMethod::Symlink
    #[serde(default)]
    pub subgroup_prefix: SubgroupPrefix,

    /// Whether to allow overwriting existing files.
    ///
    /// This alters syncing behaviours when the target file exists.  If set
    /// to `true`, no errors/warnings will be omitted when the target
    /// file exists; otherwise reports error and skips the existing item.
    /// Using dry run to spot the existing files before syncing is
    /// recommended.
    #[serde(default)]
    pub allow_overwrite: AllowOverwrite,

    /// Whether to treat errors omitted during syncing as warnings.  It has a
    /// [per-group counterpart] to set per-group behaviours.  Note that errors
    /// occured before or after syncing are NOT affected.
    ///
    /// [per-group counterpart]: Group::ignore_failure
    #[serde(default)]
    pub ignore_failure: IgnoreFailure,

    /// Whether to enable templating.  It has a [per-group counterpart] to set
    /// if a group is to be rendered.
    ///
    /// [per-group counterpart]: Group::renderable
    #[serde(default)]
    pub renderable: Renderable,

    /// The hostname separator.
    ///
    /// Specifies default value when [`Group::hostname_sep`] is not set.
    ///
    /// [`Group::hostname_sep`]: Group::hostname_sep
    #[serde(default)]
    pub hostname_sep: HostnameSeparator,

    /// Global item renaming rules.
    ///
    /// Rules defined here will be prepended to renaming rules of each group.
    /// See [`Group::rename`].
    ///
    /// [`Group::rename`]: Group::rename
    #[serde(default)]
    pub rename: RenamingRules,
}

/// Templating values are defined in this section.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContextConfig(toml::Value);
impl Default for ContextConfig {
    fn default() -> Self {
        Self(toml::map::Map::new().into())
    }
}

/// Configures how items are grouped.
#[derive(Default, Clone, Deserialize, Debug)]
pub struct Group<T>
where
    T: Operate,
{
    /// The global config object loaded from DT's config file.  This field
    /// _does not_ appear in the config file, but is only used by DT
    /// internally.  Skipping deserializing is achieved via serde's
    /// [`skip_deserializing`] attribute, which fills a default value when
    /// deserializing.
    ///
    /// [`skip_deserializing`]: https://serde.rs/field-attrs.html#skip_deserializing
    #[serde(skip_deserializing)]
    pub global: Rc<GlobalConfig>,

    /// The context config object loaded from config file.  Like
    /// [`Group::global`], this field _does not_ appear in the config, but is
    /// only used by DT internally.
    ///
    /// [`Group::global`]: Group::global
    #[serde(skip_deserializing)]
    pub context: Rc<ContextConfig>,

    /// Name of this group, used as namespace in staging root directory.
    pub name: GroupName,

    /// The priority of this group, used to resolve possibly duplicated
    /// items.  See [`DTScope`] for details.
    ///
    /// [`DTScope`]: DTScope
    #[serde(default)]
    pub scope: DTScope,

    /// The base directory of all source items.  This simplifies
    /// configuration files with common prefixes in the [`sources`]
    /// array.
    ///
    /// [`sources`]: Group::sources
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
    /// base = "dt/dt-cli"
    /// sources = ["*"]
    /// target = "."
    /// ```
    ///
    /// It will only sync `src/main.rs` to the configured target directory
    /// (in this case, the directory where [DT] is being executed).
    ///
    /// [DT]: https://github.com/blurgyy/dt
    pub base: T,

    /// Paths (relative to [`base`]) to the items to be synced.
    ///
    /// [`base`]: Group::base
    pub sources: Vec<T>,

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
    pub ignored: Option<RenamingRules>,

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
    /// [`hostname_sep`]: Group::hostname_sep
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
    /// ```toml
    /// [[local]]
    /// ...
    /// hostname_sep = "@@"
    ///
    /// base = "~/.ssh"
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
    pub hostname_sep: Option<HostnameSeparator>,

    /// (Optional) Whether to allow overwriting existing files.  Dead
    /// symlinks are treated as non-existing, and are always overwritten
    /// (regardless of this option).
    pub allow_overwrite: Option<AllowOverwrite>,

    /// (Optional) Whether to treat errors omitted during syncing of this
    /// group as warnings.  Note that errors occured before or after syncing
    /// are NOT affected.
    pub ignore_failure: Option<IgnoreFailure>,

    /// (Optional) Whether to enable templating for source files in this
    /// group.
    #[serde(default)]
    pub renderable: Option<Renderable>,

    /// (Optional) Syncing method, overrides [`global.method`] key.
    ///
    /// [`global.method`]: GlobalConfig::method
    pub method: Option<SyncMethod>,

    /// A string to be prepended to a subgroup's name when creating its
    /// staging directory with the [`Symlink`] syncing method, overrides
    /// [`global.subgroup_prefix`] key.
    ///
    /// [`Symlink`]: SyncMethod::Symlink
    /// [`global.subgroup_prefix`]: GlobalConfig::subgroup_prefix
    pub subgroup_prefix: Option<SubgroupPrefix>,

    /// (Optional) Renaming rules, appends to [`global.rename`].
    ///
    /// [`global.rename`]: GlobalConfig::rename
    #[serde(default)]
    pub rename: RenamingRules,
}

impl<T> Group<T>
where
    T: Operate,
{
    /// Gets the [`allow_overwrite`] key from a `Group` object, falls back to
    /// the `allow_overwrite` from its parent global config.
    ///
    /// [`allow_overwrite`]: Group::allow_overwrite
    pub fn is_overwrite_allowed(&self) -> bool {
        match self.allow_overwrite {
            Some(AllowOverwrite(allow_overwrite)) => allow_overwrite,
            _ => self.global.allow_overwrite.0,
        }
    }

    /// Gets the [`ignore_failure`] key from a `Group` object, falls back to
    /// the `ignore_failure` from its parent global config.
    ///
    /// [`ignore_failure`]: Group::ignore_failure
    pub fn is_failure_ignored(&self) -> bool {
        match self.ignore_failure {
            Some(IgnoreFailure(ignore_failure)) => ignore_failure,
            _ => self.global.ignore_failure.0,
        }
    }

    /// Gets the absolute path to this group's staging directory, with the
    /// subgroup components padded with configured [`subgroup_prefix`]es.
    ///
    /// [`subgroup_prefix`]: Group::subgroup_prefix
    pub fn get_staging_dir(&self) -> PathBuf {
        self.global
            .staging
            .0
            .join(self.name.with_subgroup_prefix(&self.get_subgroup_prefix()))
    }

    /// Gets the [`method`] key from a `Group` object, falls back to the
    /// `method` from its parent global config.
    ///
    /// [`method`]: Group::method
    pub fn get_method(&self) -> SyncMethod {
        match self.method {
            Some(method) => method,
            _ => self.global.method,
        }
    }

    /// Gets the [`subgroup_prefix`] key from a `Group` object, falls back to
    /// the `subgroup_prefix` from its parent global config.
    ///
    /// [`subgroup_prefix`]: Group::subgroup_prefix
    pub fn get_subgroup_prefix(&self) -> String {
        match &self.subgroup_prefix {
            Some(prefix) => prefix.0.to_owned(),
            _ => self.global.subgroup_prefix.0.to_owned(),
        }
    }

    /// Gets the [`hostname_sep`] key from a `Group` object, falls back to the
    /// [`hostname_sep`] from its parent global config.
    ///
    /// [`hostname_sep`]: Group::hostname_sep
    pub fn get_hostname_sep(&self) -> String {
        match &self.hostname_sep {
            Some(hostname_sep) => hostname_sep.0.to_owned(),
            _ => self.global.hostname_sep.0.to_owned(),
        }
    }

    /// Gets the list of [renaming rules] of this group, which is an array
    /// of (REGEX, SUBSTITUTION) tuples composed of [`global.rename`] and
    /// [`group.rename`], used in [`Operate::make_target`] to rename the item.
    /// The returned list is a combination of the rules from global config and
    /// the group's own rules.
    ///
    /// [renaming rules]: Group::rename
    /// [`global.rename`]: GlobalConfig::rename
    /// [`group.rename`]: Group::rename
    /// [`Operate::make_target`]: crate::item::Operate::make_target
    pub fn get_renaming_rules(&self) -> Vec<RenamingRule> {
        let mut ret: Vec<RenamingRule> = Vec::new();
        for r in &self.global.rename.0 {
            ret.push(r.to_owned());
        }
        for r in &self.rename.0 {
            ret.push(r.to_owned());
        }
        ret
    }

    /// Check if this group is renderable according to the cascaded config
    /// options.
    pub fn is_renderable(&self) -> bool {
        match self.renderable {
            Some(Renderable(renderable)) => renderable,
            _ => self.global.renderable.0,
        }
    }

    /// Validates this group with readonly access to the filesystem.  The
    /// following cases are denied:
    ///
    ///   1. Invalid group name
    ///   2. Source item referencing parent (because items are first populated
    ///      to the [`staging`] directory, and the structure under the
    ///      [`staging`] directory depends on their original relative path to
    ///      their [`base`])
    ///   3. TODO: Current group contains unimplemented [`ignored`] field
    ///
    /// NOTE: When [`base`] is empty, sources will be looked up in the cwd of
    /// the process.
    ///
    /// [`ignored`]: Group::ignored
    /// [`base`]: Group::base
    fn _validate_no_fs_query(&self) -> Result<()> {
        // 1. Invalid group name
        self.name.validate()?;
        // 2. Source item referencing parent
        if self.sources.iter().any(|s| s.is_twisted()) {
            return Err(AppError::ConfigError(format!(
                "source item references parent directory in group '{}'",
                self.name,
            )));
        }
        // 3. Current group contains unimplemented `ignored` field
        if self.ignored.is_some() {
            todo!("`ignored` array works poorly and I decided to implement it in the future");
        }

        Ok(())
    }

    /// Validates this group via querying the filesystem.  The following cases
    /// are denied:
    ///
    ///   1. Wrong type of existing [`staging`] path (if using the
    ///      [`Symlink`] method)
    ///   2. Path to staging root contains readonly parent directory (if
    ///      using the [`Symlink`] method)
    ///   3. Wrong type of existing [`target`] path
    ///   4. Path to [`target`] contains readonly parent directory
    ///
    /// [`staging`]: GlobalConfig::staging
    /// [`Symlink`]: SyncMethod::Symlink
    /// [`target`]: LocalGroup::target
    fn _validate_with_fs_query(&self) -> Result<()> {
        if self.get_method() == SyncMethod::Symlink {
            let staging_path: PathBuf = self.global.staging.0.to_owned();

            // 1. Wrong type of existing staging path
            if staging_path.exists() && !staging_path.is_dir() {
                return Err(AppError::ConfigError(
                    "staging root path exists but is not a valid directory"
                        .to_owned(),
                ));
            }

            // 2. Path to staging root contains readonly parent directory
            // NOTE: Must convert to an absolute path before checking readonly
            if staging_path.absolute()?.is_parent_readonly() {
                return Err(AppError::ConfigError(
                    "staging root path cannot be created due to insufficient permissions"
                        .to_owned(),
                ));
            }
        }

        // 3. Wrong type of existing target path
        if self.target.exists() && !self.target.is_dir() {
            return Err(AppError::ConfigError(format!(
                "target path exists but is not a valid directory in group '{}'",
                self.name,
            )));
        }

        // 4. Path to target contains readonly parent directory
        // NOTE: Must convert to an absolute path before checking readonly
        if self.target.to_owned().absolute()?.is_parent_readonly() {
            return Err(AppError::ConfigError(format!(
                "target path cannot be created due to insufficient permissions in group '{}'",
                self.name,
            )));
        }

        Ok(())
    }
}

/// Configures how local items are grouped.
pub type LocalGroup = Group<PathBuf>;

impl LocalGroup {
    /// Validates this local group, the following cases are denied:
    ///
    /// - Checks without querying the filesystem
    ///
    ///   1. Empty [group name]
    ///   2. Source item referencing parent (because items are first populated
    ///      to the [`staging`] directory, and the structure under the
    ///      [`staging`] directory depends on their original relative path to
    ///      their [`base`])
    ///   3. Current group contains unimplemented [`ignored`] field
    ///
    ///   4. Target and base are the same
    ///   5. Base contains [`hostname_sep`]
    ///   6. Source item is absolute (same reason as above)
    ///   7. Source item contains bad globbing pattern
    ///   8. Source item contains [`hostname_sep`]
    ///
    /// - Checks that need to query the filesystem
    ///
    ///   1. Wrong type of existing [`staging`] path (if using the
    ///      [`Symlink`] method)
    ///   2. Path to staging root contains readonly parent directory (if
    ///      using the [`Symlink`] method)
    ///   3. Wrong type of existing [`target`] path
    ///   4. Path to [`target`] contains readonly parent directory
    ///
    ///   5. Base is unreadable
    ///
    /// [group name]: LocalGroup::name
    /// [`base`]: LocalGroup::base
    /// [`target`]: LocalGroup::target
    /// [`staging`]: GlobalConfig::staging
    /// [`ignored`]: Group::ignored
    /// [`hostname_sep`]: LocalGroup::hostname_sep
    /// [`Symlink`]: SyncMethod::Symlink
    pub fn validate(&self) -> Result<()> {
        // - Checks without querying the filesystem --------------------------
        // 1-4
        self._validate_no_fs_query()?;

        // 5. Target and base are the same
        if self.base == self.target {
            return Err(AppError::ConfigError(format!(
                "base directory and its target are the same in group '{}'",
                self.name,
            )));
        }

        // 6. Base contains hostname_sep
        let hostname_sep = self.get_hostname_sep();
        if self.base.to_string_lossy().contains(&hostname_sep) {
            return Err(AppError::ConfigError(format!(
                "base directory contains hostname_sep ({}) in group '{}'",
                hostname_sep, self.name,
            )));
        }

        // 7. Source item is absolute
        if self
            .sources
            .iter()
            .any(|s| s.starts_with("/") || s.starts_with("~"))
        {
            return Err(AppError::ConfigError(format!(
                "source array contains absolute path in group '{}'",
                self.name,
            )));
        }

        // 8. Source item contains bad globbing pattern
        if self.sources.iter().any(|s| {
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

        // 9. Source item contains hostname_sep
        if self.sources.iter().any(|s| {
            let s = s.to_string_lossy();
            s.contains(&hostname_sep)
        }) {
            return Err(AppError::ConfigError(format!(
                "a source item contains hostname_sep ({}) in group '{}'",
                hostname_sep, self.name,
            )));
        }

        // - Checks that need to query the filesystem ------------------------
        // 1-4
        self._validate_with_fs_query()?;

        // 5. Base is unreadable
        if self.base.exists() {
            // Check read permission of `base`
            if let Err(e) = std::fs::read_dir(&self.base) {
                log::error!("Could not read base '{}'", self.base.display());
                return Err(e.into());
            }
        }

        Ok(())
    }
}

/// Configures how remote items are grouped.
pub type RemoteGroup = Group<Url>;

impl RemoteGroup {
    /// Validates this remote group, the following cases are denied:
    ///
    /// - Checks without querying the filesystem
    ///
    ///   1. Empty [group name]
    ///   2. Empty [`target`]
    ///   3. Source item referencing parent (because items are first populated
    ///      to the [`staging`] directory, and the structure under the
    ///      [`staging`] directory depends on their original relative path to
    ///      their [`base`])
    ///   4. Current group contains unimplemented [`ignored`] field
    ///
    /// - Checks that need to query the filesystem
    ///
    ///   1. Wrong type of existing [`staging`] path (if using the
    ///      [`Symlink`] method)
    ///   2. Path to staging root contains readonly parent directory (if
    ///      using the [`Symlink`] method)
    ///   3. Wrong type of existing [`target`] path
    ///   4. Path to [`target`] contains readonly parent directory
    ///
    /// [group name]: LocalGroup::name
    /// [`base`]: LocalGroup::base
    /// [`target`]: LocalGroup::target
    /// [`staging`]: GlobalConfig::staging
    /// [`ignored`]: Group::ignored
    /// [`Symlink`]: SyncMethod::Symlink
    fn validate(&self) -> Result<()> {
        // - Checks without querying the filesystem --------------------------
        // 1-5
        self._validate_no_fs_query()?;

        // - Checks that need to query the filesystem ------------------------
        // 1-4
        self._validate_with_fs_query()?;

        Ok(())
    }
}

#[cfg(test)]
mod overriding_global {
    use std::str::FromStr;

    use super::{DTConfig, SyncMethod};
    use color_eyre::Report;
    use pretty_assertions::assert_eq;

    #[test]
    fn allow_overwrite_no_global() -> Result<(), Report> {
        let config = DTConfig::from_str(
            r#"
[[local]]
name = "placeholder"
base = "~"
sources = ["*"]
target = "."
allow_overwrite = true"#,
        )?;
        for group in config.local {
            assert_eq!(group.is_overwrite_allowed(), true);
        }
        Ok(())
    }

    #[test]
    fn allow_overwrite_with_global() -> Result<(), Report> {
        let config = DTConfig::from_str(
            r#"
[global]
method = "Copy"  # Default value because not testing this key
allow_overwrite = true

[[local]]
name = "placeholder"
base = "~"
sources = ["*"]
target = "."
allow_overwrite = false"#,
        )?;
        for group in config.local {
            assert_eq!(group.is_overwrite_allowed(), false);
        }
        Ok(())
    }

    #[test]
    fn both_allow_overwrite_and_method_no_global() -> Result<(), Report> {
        let config = DTConfig::from_str(
            r#"
[[local]]
name = "placeholder"
base = "~"
sources = ["*"]
target = "."
method = "Copy"
allow_overwrite = true"#,
        )?;
        for group in config.local {
            assert_eq!(group.get_method(), SyncMethod::Copy);
            assert_eq!(group.is_overwrite_allowed(), true);
        }
        Ok(())
    }

    #[test]
    fn both_allow_overwrite_and_method_with_global() -> Result<(), Report> {
        let config = DTConfig::from_str(
            r#"
[global]
method = "Copy"
allow_overwrite = true

[[local]]
name = "placeholder"
base = "~"
sources = ["*"]
target = "."
method = "Symlink"
allow_overwrite = false"#,
        )?;
        for group in config.local {
            assert_eq!(group.get_method(), SyncMethod::Symlink);
            assert_eq!(group.is_overwrite_allowed(), false);
        }
        Ok(())
    }

    #[test]
    fn hostname_sep_no_global() -> Result<(), Report> {
        let config = DTConfig::from_str(
            r#"
[[local]]
name = "hostname_sep no global test"
hostname_sep = "@-@"
base = "~"
sources = []
target = ".""#,
        )?;
        for group in config.local {
            assert_eq!(group.get_hostname_sep(), "@-@");
        }
        Ok(())
    }

    #[test]
    fn hostname_sep_with_global() -> Result<(), Report> {
        let config = DTConfig::from_str(
            r#"
[global]
hostname_sep = "@-@"

[[local]]
name = "hostname_sep fall back to global"
base = "~"
sources = []
target = ".""#,
        )?;
        for group in config.local {
            assert_eq!(group.get_hostname_sep(), "@-@");
        }
        Ok(())
    }

    #[test]
    fn method_no_global() -> Result<(), Report> {
        let config = DTConfig::from_str(
            r#"
[[local]]
name = "placeholder"
base = "~"
sources = ["*"]
target = "."
method = "Copy""#,
        )?;
        for group in config.local {
            assert_eq!(group.get_method(), SyncMethod::Copy)
        }
        Ok(())
    }

    #[test]
    fn method_with_global() -> Result<(), Report> {
        let config = DTConfig::from_str(
            r#"
[global]
method = "Copy"
allow_overwrite = false # Default value because not testing this key

[[local]]
name = "placeholder"
base = "~"
sources = ["*"]
target = "."
method = "Symlink""#,
        )?;
        for group in config.local {
            assert_eq!(group.get_method(), SyncMethod::Symlink)
        }
        Ok(())
    }
}

#[cfg(test)]
mod tilde_expansion {
    use std::str::FromStr;

    use color_eyre::Report;
    use pretty_assertions::assert_eq;

    use super::DTConfig;

    #[test]
    fn all() -> Result<(), Report> {
        let config = DTConfig::from_str(
            r#"
[global]
staging = "~"
method = "Symlink"
allow_overwrite = false


[[local]]
name = "expand tilde in base and target"
base = "~"
sources = []
target = "~/dt/target""#,
        )?;
        dbg!(&config.global.staging.0);
        assert_eq!(Some(config.global.staging.0), dirs::home_dir());
        config.local.iter().all(|group| {
            dbg!(&group.base);
            dbg!(&group.target);
            assert_eq!(Some(group.to_owned().base), dirs::home_dir());
            assert_eq!(
                Some(group.to_owned().target),
                dirs::home_dir()
                    .map(|p| p.join("dt"))
                    .map(|p| p.join("target")),
            );
            true
        });
        Ok(())
    }
}

#[cfg(test)]
mod validation {
    use std::str::FromStr;

    use color_eyre::{eyre::eyre, Report};
    use pretty_assertions::assert_eq;

    use super::DTConfig;
    use crate::error::Error as AppError;

    #[test]
    fn relative_component_in_group_name() -> Result<(), Report> {
        if let Err(err) = DTConfig::from_str(
            r#"
[[local]]
name = "a/../b"
base = "~"
sources = []
target = ".""#,
        ) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "Group name should not contain relative component"
                        .to_owned(),
                ),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because a group's name contains relative component"))
        }
    }

    #[test]
    fn prefix_slash_in_group_name() -> Result<(), Report> {
        if let Err(err) = DTConfig::from_str(
            r#"
[[local]]
name = "/a/b/c/d"
base = "~"
sources = []
target = ".""#,
        ) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "Group name should not start with slash".to_owned(),
                ),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because a group's name starts with a slash"))
        }
    }

    #[test]
    fn empty_group_name() -> Result<(), Report> {
        if let Err(err) = DTConfig::from_str(
            r#"
[[local]]
name = ""
base = "~"
sources = []
target = ".""#,
        ) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "Group name should not be empty".to_owned(),
                ),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because a group's name is empty"))
        }
    }

    #[test]
    fn base_is_target() -> Result<(), Report> {
        if let Err(err) = DTConfig::from_str(
            r#"
[[local]]
name = "base is target"
base = "~"
sources = []
target = "~""#,
        ) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "base directory and its target are the same in group 'base is target'"
                        .to_owned(),
                ),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because base and target are the same"))
        }
    }

    #[test]
    fn base_contains_hostname_sep() -> Result<(), Report> {
        if let Err(err) = DTConfig::from_str(
            r#"
[[local]]
name = "base contains hostname_sep"
hostname_sep = "@@"
base = "~/.config/sytemd/user@@elbert"
sources = []
target = ".""#,
        ) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "base directory contains hostname_sep (@@) in group 'base contains hostname_sep'"
                        .to_owned(),
                ),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!("This config should not be loaded because a base contains hostname_sep"))
        }
    }

    #[test]
    fn source_item_referencing_parent() -> Result<(), Report> {
        if let Err(err) = DTConfig::from_str(
            r#"
[[local]]
name = "source item references parent dir"
base = "."
sources = ["../Cargo.toml"]
target = "target""#,
        ) {
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
        if let Err(err) = DTConfig::from_str(
            r#"
[[local]]
name = "source item is absolute"
base = "~"
sources = ["/usr/share/gdb-dashboard/.gdbinit"]
target = "/tmp""#,
        ) {
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
        if let Err(err) = DTConfig::from_str(
            r#"
[[local]]
name = "placeholder"
base = "~"
sources = [".*"]
target = ".""#,
        ) {
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
        if let Err(err) = DTConfig::from_str(
            r#"
[[local]]
name = "@@ in source item"
base = "~/.config/nvim"
sources = ["init.vim@@elbert"]
target = ".""#,
        ) {
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

#[cfg(test)]
mod validation_physical {
    use std::str::FromStr;

    use color_eyre::{eyre::eyre, Report};

    use super::DTConfig;
    use crate::error::Error as AppError;
    use crate::utils::testing::{
        get_testroot, prepare_directory, prepare_file,
    };

    #[test]
    fn non_existent_relative_staging_and_target() -> Result<(), Report> {
        if let Err(err) = DTConfig::from_str(
            r#"
[global]
staging = "staging-882b842397c5b44929b9c5f4e83130c9-dir"

[[local]]
name = "readable relative non-existent target"
base = "base-7f2f7ff8407a330751f13dc5ec86db1b-dir"
sources = ["b1db25c31c23950132a44f6faec2005c"]
target = "target-ce59cb1aea35e22e43195d4a444ff2e7-dir""#,
        ) {
            Err(eyre!("Non-existent, relative but readable staging/target path should be loaded without error (got error :'{}')", err))
        } else {
            Ok(())
        }
    }

    #[test]
    fn staging_is_file() -> Result<(), Report> {
        let staging_path = prepare_file(
            get_testroot("validation_physical")
                .join("staging_is_file")
                .join("staging-but-file"),
            0o644,
        )?;
        let base = prepare_directory(
            get_testroot("validation_physical")
                .join("staging_is_file")
                .join("base"),
            0o755,
        )?;
        let target = prepare_directory(
            get_testroot("validation_physical")
                .join("staging_is_file")
                .join("target"),
            0o755,
        )?;

        if let Err(err) = DTConfig::from_str(&format!(
            r#"
[global]
staging = "{}"

[[local]]
name = "staging is file"
base = "{}"
sources = []
target = "{}""#,
            staging_path.display(),
            base.display(),
            target.display(),
        )) {
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
                "This config should not be validated because staging is not a directory",
            ))
        }
    }

    #[test]
    fn staging_readonly() -> Result<(), Report> {
        let staging_path = prepare_directory(
            get_testroot("validation_physical")
                .join("staging_readonly")
                .join("staging-but-readonly"),
            0o555,
        )?;
        let base = prepare_directory(
            get_testroot("validation_physical")
                .join("staging_readonly")
                .join("base"),
            0o755,
        )?;
        let target_path = prepare_directory(
            get_testroot("validation_physical")
                .join("staging_readonly")
                .join("target"),
            0o755,
        )?;

        if let Err(err) = DTConfig::from_str(&format!(
            r#"
[global]
staging = "{}"

[[local]]
name = "staging is readonly"
base = "{}"
sources = []
target = "{}""#,
            staging_path.display(),
            base.display(),
            target_path.display(),
        )) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "staging root path cannot be created due to insufficient permissions"
                        .to_owned(),
                ),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!(
                "This config should not be validated because staging path is readonly",
            ))
        }
    }
    #[test]
    fn target_is_file() -> Result<(), Report> {
        let target_path = prepare_file(
            get_testroot("validation_physical")
                .join("target_is_file")
                .join("target-but-file"),
            0o755,
        )?;
        if let Err(err) = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "target path is absolute"
base = "."
sources = []
target = "{}""#,
            target_path.display(),
        )) {
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
                "This config should not be validated because target is not a directory",
            ))
        }
    }

    #[test]
    fn target_readonly() -> Result<(), Report> {
        // setup
        let base = prepare_directory(
            get_testroot("validation_physical")
                .join("target_readonly")
                .join("base"),
            0o755,
        )?;
        let target_path = prepare_directory(
            get_testroot("validation_physical")
                .join("target_readonly")
                .join("target-but-readonly"),
            0o555,
        )?;

        if let Err(err) = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "target is readonly"
base = "{}"
sources = []
target = "{}""#,
            base.display(),
            target_path.display(),
        )) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "target path cannot be created due to insufficient permissions in group 'target is readonly'"
                        .to_owned(),
                ),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!(
                "This config should not be validated because target path is readonly",
            ))
        }
    }

    #[test]
    fn identical_configured_base_and_target_in_local() -> Result<(), Report> {
        // setup
        let base = prepare_directory(
            get_testroot("validation_physical")
                .join("local_group_has_same_base_and_target")
                .join("base-and-target"),
            0o755,
        )?;
        let target_path = base.clone();

        if let Err(err) = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "same base and target"
base = "{}"
sources = []
target = "{}"
"#,
            base.display(),
            target_path.display(),
        )) {
            assert_eq!(
                err,
                AppError::ConfigError(
                    "base directory and its target are the same in group 'same base and target'"
                        .to_owned(),
                ),
                "{}",
                err,
            );
            Ok(())
        } else {
            Err(eyre!(
                "This config should not be validated because a local group's base and target are identical"
            ))
        }
    }

    #[test]
    fn base_unreadable() -> Result<(), Report> {
        let base = prepare_file(
            get_testroot("validation_physical")
                .join("base_unreadable")
                .join("base-but-file"),
            0o311,
        )?;
        if let Err(err) = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "base unreadable (not a directory)"
base = "{}"
sources = []
target = ".""#,
            base.display(),
        )) {
            assert_eq!(
                err,
                AppError::IoError("Not a directory (os error 20)".to_owned(),),
                "{}",
                err,
            );
        } else {
            return Err(eyre!(
                "This config should not be loaded because base is not a directory",
            ));
        }

        let base = prepare_directory(
            get_testroot("validation_physical")
                .join("base_unreadable")
                .join("base-unreadable"),
            0o311,
        )?;
        if let Err(err) = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "base unreadable (permission denied)"
base = "{}"
sources = []
target = ".""#,
            base.display(),
        )) {
            assert_eq!(
                err,
                AppError::IoError(
                    "Permission denied (os error 13)".to_owned(),
                ),
                "{}",
                err,
            );
        } else {
            return Err(eyre!(
                "This config should not be loaded because insufficient permissions to base",
            ));
        }

        Ok(())
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 21 2021, 01:14 [CST]
