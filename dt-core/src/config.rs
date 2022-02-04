use std::{
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
    item::DTItem,
};

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
/// Helper type for config key `allow_overwrite`
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
/// Helper type for config key `hostname_sep`
///
/// [`hostname_sep`]: GlobalConfig::hostname_sep
#[derive(Clone, Debug, Deserialize)]
pub struct HostnameSeparator(pub String);
impl Default for HostnameSeparator {
    fn default() -> Self {
        Self("@@".to_owned())
    }
}
/// Helper type for config key `rename`
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
        let confstr = std::fs::read_to_string(path).unwrap_or_else(|_| {
            panic!("Could not load config from '{}'", path.display())
        });
        Self::from_str(&confstr)
    }

    /// Construct another [`DTConfig`] object with only groups with given
    /// names remaining, unmatched given names are ignored.
    pub fn filter_names(self, group_names: Vec<String>) -> Self {
        Self {
            global: self.global,
            context: self.context,
            local: self
                .local
                .iter()
                .filter(|l| group_names.iter().any(|n| l.name == *n))
                .map(|l| l.to_owned())
                .collect(),
            remote: self
                .remote
                .iter()
                .filter(|l| group_names.iter().any(|n| l.name == *n))
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
                    staging.0.to_str().unwrap_or_else(|| {
                        panic!(
                            "Failed expanding tilde in `global.staging` ({})",
                            staging.0.display(),
                        )
                    }),
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
                    group.base.to_str().unwrap_or_else(|| {
                        panic!(
                            "Failed expanding tilde in `local.base` '{}'",
                            group.base.display(),
                        )
                    }),
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
                    group.target.to_str().unwrap_or_else(|| {
                        panic!(
                            "Failed expanding tilde in `local.target` '{}'",
                            group.target.display(),
                        )
                    }),
                ))
                .unwrap()
            };
        }

        ret
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

    /// Whether to allow overwriting existing files.
    ///
    /// This alters syncing behaviours when the target file exists.  If set
    /// to `true`, no errors/warnings will be omitted when the target
    /// file exists; otherwise reports error and skips the existing item.
    /// Using dry run to spot the existing files before syncing is
    /// recommended.
    #[serde(default)]
    pub allow_overwrite: AllowOverwrite,

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
pub struct Group<BaseType> {
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
    pub name: String,

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
    pub base: BaseType,

    /// Paths (relative to [`base`]) to the items to be synced.
    ///
    /// [`base`]: Group::base
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
    /// ```toml [[local]]
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

    /// (Optional) Syncing method, overrides [`global.method`] key.
    ///
    /// [`global.method`]: GlobalConfig::method
    pub method: Option<SyncMethod>,

    /// (Optional) Renaming rules, appends to [`global.rename`].
    ///
    /// [`global.rename`]: GlobalConfig::rename
    #[serde(default)]
    pub rename: RenamingRules,
}

impl<BaseType> Group<BaseType> {
    /// Gets the [`allow_overwrite`] key from a `Group` object,
    /// falls back to the `allow_overwrite` from provided global config.
    ///
    /// [`allow_overwrite`]: Group::allow_overwrite
    pub fn is_overwrite_allowed(&self) -> bool {
        match self.allow_overwrite {
            Some(allow_overwrite) => allow_overwrite.0,
            _ => self.global.allow_overwrite.0,
        }
    }

    /// Gets the [`method`] key from a `Group` object, falls back
    /// to the `method` from provided global config.
    ///
    /// [`method`]: Group::method
    pub fn get_method(&self) -> SyncMethod {
        match self.method {
            Some(method) => method,
            _ => self.global.method,
        }
    }

    /// Gets the [`hostname_sep`] key from a `Group` object, falls
    /// back to the [`hostname_sep`] from provided global config.
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
    /// [`local.rename`], used in [`DTItem::make_target`] to rename the item.
    ///
    /// [renaming rules]: Group::rename
    /// [`global.rename`]: GlobalConfig::rename
    /// [`local.rename`]: Group::rename
    /// [`DTItem::make_target`]: crate::item::DTItem::make_target
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

    /// Check if this group is templated by checking whether the [context]
    /// section contains this group's name as a key.
    ///
    /// [context]: DTConfig::context
    pub fn is_templated(&self) -> bool {
        match self.context.0.as_table() {
            Some(map) => map.get(&self.name).is_some(),
            None => false,
        }
    }

    /// Validates this group with readonly access to the filesystem.  The
    /// following cases are denied:
    ///
    ///   1. Empty group name
    ///   2. Slash in group name
    ///   3. Source item referencing parent (because items are first populated
    ///      to the [`staging`] directory, and the structure under the
    ///      [`staging`] directory depends on their original relative path to
    ///      their [`base`])
    ///   4. TODO: Current group contains unimplemented [`ignore`] field
    ///
    /// NOTE: When [`base`] is empty, sources will be looked up in the cwd of
    /// the process.
    ///
    /// [`ignore`]: Group::ignore
    /// [`base`]: Group::base
    fn _validate_no_fs_query(&self) -> Result<()> {
        // 1. Empty group name
        if self.name.is_empty() {
            return Err(AppError::ConfigError("empty group name".to_owned()));
        }
        // 2. Slash in group name
        if self.name.contains('/') {
            return Err(AppError::ConfigError(format!(
                "group name '{}' contains the '/' character",
                self.name,
            )));
        }
        // 3. Source item referencing parent
        if self.sources.iter().any(|s| s.starts_with("../")) {
            return Err(AppError::ConfigError(format!(
                "source item references parent directory in group '{}'",
                self.name,
            )));
        }
        // 4. Current group contains unimplemented ignore field
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
            if staging_path.parent_readonly() {
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
        if self.target.parent_readonly() {
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
    ///   1. Empty group name
    ///   2. Slash in group name
    ///   3. Source item referencing parent (because items are first populated
    ///      to the [`staging`] directory, and the structure under the
    ///      [`staging`] directory depends on their original relative path to
    ///      their [`base`])
    ///   4. Current group contains unimplemented [`ignore`] field
    ///
    ///   5. Target and base are the same
    ///   6. Base contains [`hostname_sep`]
    ///   7. Source item is absolute (same reason as above)
    ///   8. Source item contains bad globbing pattern
    ///   9. Source item contains [`hostname_sep`]
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
        if self.base.to_str().unwrap().contains(&hostname_sep) {
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
            let s = s.to_str().unwrap();
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

        Ok(())
    }
}

/// Configures how remote items are grouped.
pub type RemoteGroup = Group<Url>;

impl RemoteGroup {
    /// Validates this remote group, the following cases are denied:
    ///
    ///   1. Empty group name
    ///   2. Empty target
    ///   3. Slash in group name
    ///   4. Source item referencing parent (because items are first populated
    ///      to the [`staging`] directory, and the structure under the
    ///      [`staging`] directory depends on their original relative path to
    ///      their [`base`])
    ///   5. Current group contains unimplemented [`ignore`] field
    fn validate(&self) -> Result<()> {
        // 1-5
        self._validate_no_fs_query()?;

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
    fn slash_in_group_name() -> Result<(), Report> {
        if let Err(err) = DTConfig::from_str(
            r#"
[[local]]
name = "this/group/name/contains/slash"
base = "~"
sources = []
target = "/tmp""#,
        ) {
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
    fn staging_is_file() -> Result<(), Report> {
        let staging_path = prepare_file(
            get_testroot()
                .join("staging_is_file")
                .join("staging-but-file"),
            0o644,
        )?;
        let base = prepare_directory(
            get_testroot().join("staging_is_file").join("base"),
            0o755,
        )?;
        let target = prepare_directory(
            get_testroot().join("staging_is_file").join("target"),
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
            get_testroot()
                .join("staging_readonly")
                .join("staging-but-readonly"),
            0o555,
        )?;
        let base = prepare_directory(
            get_testroot().join("staging_readonly").join("base"),
            0o755,
        )?;
        let target_path = prepare_directory(
            get_testroot().join("staging_readonly").join("target"),
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
            get_testroot()
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
            get_testroot().join("target_readonly").join("base"),
            0o755,
        )?;
        let target_path = prepare_directory(
            get_testroot()
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
            get_testroot()
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
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 21 2021, 01:14 [CST]
