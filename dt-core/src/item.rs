use std::{
    os::unix::prelude::PermissionsExt,
    path::{Path, PathBuf},
    rc::Rc,
};

use path_clean::PathClean;
use serde::Serialize;
use url::Url;

use crate::{
    config::{Group, LocalGroup, RenamingRule, SyncMethod},
    error::{Error as AppError, Result},
    registry::Register,
    utils,
};

/// Defines shared behaviours for an item (a path to a file) used in [DT].
///
/// [DT]: https://github.com/blurgyy/dt
#[allow(unused_variables)]
pub trait Operate
where
    Self: Sized,
{
    /// Checks if the item is for another machine.
    fn is_for_other_host(&self, hostname_sep: &str) -> bool {
        unimplemented!()
    }
    /// Gets the absolute location of `self`, if applicable.
    fn absolute(self) -> Result<Self> {
        unimplemented!()
    }
    /// Gets the host-specific counterpart of `self`, if applicable.  If
    /// `self` is already host-specific, returns `self` directly.
    fn host_specific(self, hostname_sep: &str) -> Self {
        unimplemented!()
    }
    /// Gets the non-host-specific counterpart of `self`, if applicable.  If
    /// `self` is already non-host-specific, returns `self` directly.
    fn non_host_specific(self, hostname_sep: &str) -> Self {
        unimplemented!()
    }
    /// Checks whether any of the component above `self` is readonly.
    fn is_parent_readonly(&self) -> bool {
        unimplemented!()
    }
    /// Checks whether any of the component refernces its parent.
    fn is_twisted(&self) -> bool {
        unimplemented!()
    }
    /// Given a `hostname_sep`, a `base`, a `targetbase`, and optionally a
    /// list of [renaming rule]s, creates the path where `self` would be
    /// synced to.  Renaming rules are applied after host-specific suffixes
    /// are stripped.
    fn make_target<P>(
        self,
        hostname_sep: &str,
        base: &Self,
        targetbase: P,
        renaming_rules: Vec<RenamingRule>,
    ) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        unimplemented!()
    }
    /// Renders this item with given context to the `dest` path.
    fn render<S: Serialize, T: Register>(
        &self,
        registry: &Rc<T>,
        ctx: &Rc<S>,
    ) -> Result<Vec<u8>> {
        unimplemented!()
    }
    /// Populate this item with given group config.  The given group config is
    /// expected to be the group where this item belongs to.
    fn populate<T: Register>(
        &self,
        group: Rc<Group<Self>>,
        registry: Rc<T>,
    ) -> Result<()> {
        unimplemented!()
    }
    /// Show what is to be done if this item is to be populated with given
    /// group config.  The given group config is expected to be the group
    /// where this item belongs to.
    fn populate_dry(&self, group: Rc<LocalGroup>) -> Result<()> {
        unimplemented!()
    }
}

impl Operate for PathBuf {
    /// Checks if the item is for another machine (by checking its name).
    ///
    /// A host-specific item is considered for another machine, when its
    /// filename contains only 1 [`hostname_sep`], and after the
    /// [`hostname_sep`] should not be current machine's hostname.
    ///
    /// A non-host-specific item is always considered **not** for another
    /// machine (because it is non-host-specific, i.e. for all machines).
    ///
    /// An item with filename containing more than 1 [`hostname_sep`] causes
    /// this function to panic.
    ///
    /// [`hostname_sep`]: crate::config::GlobalConfig::hostname_sep
    fn is_for_other_host(&self, hostname_sep: &str) -> bool {
        let filename = self
            .file_name()
            .unwrap_or_else(|| {
                panic!(
                    "Failed extracting file name from path '{}'",
                    self.display(),
                )
            })
            .to_str()
            .unwrap_or_else(|| {
                panic!(
                    "Failed converting &OsStr to &str for path '{}'",
                    self.display(),
                )
            });
        let splitted: Vec<_> = filename.split(hostname_sep).collect();

        assert!(
        splitted.len() <= 2,
        "There appears to be more than 1 occurrences of hostname_sep ({}) in this path: {}",
        hostname_sep,
        self.display(),
    );
        assert!(
            !splitted.first().unwrap().is_empty(),
            "hostname_sep ({}) appears to be a prefix of this path: {}",
            hostname_sep,
            self.display(),
        );

        splitted.len() > 1
            && splitted.last() != gethostname::gethostname().to_str().as_ref()
    }

    /// Gets the absolute path of `self`, **without** traversing symlinks.
    ///
    /// Reference: <https://stackoverflow.com/a/54817755/13482274>
    fn absolute(self) -> Result<Self> {
        let absolute_path = if self.is_absolute() {
            self.to_owned()
        } else {
            std::env::current_dir()?.join(self)
        }
        .clean();

        Ok(absolute_path)
    }

    /// Gets the host-specific counterpart of `self`.  If `self` is already
    /// host-specific, returns `self` directly.
    fn host_specific(self, hostname_sep: &str) -> Self {
        if self.ends_with(utils::host_specific_suffix(hostname_sep)) {
            self.into()
        } else {
            let hs_filename = self
                .file_name()
                .unwrap_or_else(|| {
                    panic!(
                        "Failed extracting file name from path '{}'",
                        self.display(),
                    )
                })
                .to_str()
                .unwrap_or_else(|| {
                    panic!(
                        "Failed converting &OsStr to &str for path: '{}'",
                        self.display(),
                    )
                })
                .to_owned()
                + &utils::host_specific_suffix(hostname_sep);

            self.with_file_name(hs_filename)
        }
    }

    /// Converts a path to a non-host-specific path.  If the input path is
    /// already non-host-specific, returns itself;  Otherwise returns a
    /// path where _every component_ of the path is converted to a
    /// non-host-specific one.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use dt_core::item::Operate;
    /// # use std::path::PathBuf;
    /// # use std::str::FromStr;
    /// let itm: PathBuf = "/some/long/path".into();
    /// assert_eq!(
    ///     itm.non_host_specific("@@"),
    ///     PathBuf::from_str("/some/long/path").unwrap(),
    /// );
    ///
    /// let itm: PathBuf = "/some@@john/long/path@@watson".into();
    /// assert_eq!(
    ///     itm.non_host_specific("@@"),
    ///     PathBuf::from_str("/some/long/path").unwrap(),
    /// );
    /// ```
    fn non_host_specific(self, hostname_sep: &str) -> Self {
        self
            .iter()
            .map(std::ffi::OsStr::to_str)
            .map(|s| {
                s.unwrap_or_else(|| {
                    panic!(
                        "Failed extracting path components from '{}'",
                        self.display(),
                    )
                })
            })
            .map(|s| {
                s.split(hostname_sep)
                    .collect::<Vec<_>>()
                    .first()
                    .unwrap_or_else(|| {
                        panic!(
                            "Failed extracting basename from component '{}' of path '{}'",
                            s,
                            self.display(),
                        )
                    })
                    .to_owned()
            })
            .collect::<PathBuf>()
    }

    /// Checks whether any of the component above `self` is readonly.
    fn is_parent_readonly(&self) -> bool {
        let mut p: &Path = self.as_ref();
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

    /// Checks whether any of the component refernces its parent.
    fn is_twisted(&self) -> bool {
        self.iter().any(|comp| comp == "..")
    }

    /// Given a `hostname_sep`, a `base`, a `targetbase`, and optionally a
    /// list of [renaming rule]s, create the path where `self` would be synced
    /// to.  Renaming rules are applied after host-specific suffixes are
    /// stripped.
    ///
    /// # Example
    ///
    /// ## No renaming rule
    ///
    /// ```rust
    /// # use dt_core::{
    /// #   config::RenamingRule,
    /// #   error::Error as AppError,
    /// #   item::Operate
    /// # };
    /// # use std::path::PathBuf;
    /// # use std::str::FromStr;
    /// let itm: PathBuf = "/path/to/source@@john/item".into();
    /// let base: PathBuf = "/path/to/source".into();
    /// let targetbase: PathBuf = "/path/to/target".into();
    ///
    /// assert_eq!(
    ///     itm.make_target("@@", &base, &targetbase, vec![])?,
    ///     PathBuf::from_str("/path/to/target/item").unwrap(),
    /// );
    /// # Ok::<(), AppError>(())
    /// ```
    ///
    /// ## Single renaming rule
    ///
    /// ```rust
    /// # use dt_core::{
    /// #   config::RenamingRule,
    /// #   error::Error as AppError,
    /// #   item::Operate
    /// # };
    /// # use std::path::PathBuf;
    /// # use std::str::FromStr;
    /// let itm: PathBuf = "/path/to/source@@john/_dot_item".into();
    /// let base: PathBuf = "/path/to/source".into();
    /// let targetbase: PathBuf = "/path/to/target".into();
    /// let rules = vec![
    ///     RenamingRule{
    ///         pattern: regex::Regex::new("^_dot_").unwrap(),
    ///         substitution: ".".into(),
    ///     },
    /// ];
    ///
    /// assert_eq!(
    ///     itm.make_target("@@", &base, &targetbase, rules)?,
    ///     PathBuf::from_str("/path/to/target/.item").unwrap(),
    /// );
    /// # Ok::<(), AppError>(())
    /// ```
    ///
    /// ## Multiple renaming rules
    ///
    /// When multiple renaming rules are supplied, they are applied one after
    /// another.
    ///
    /// ```rust
    /// # use dt_core::{
    /// #   config::RenamingRule,
    /// #   error::Error as AppError,
    /// #   item::Operate
    /// # };
    /// # use std::path::PathBuf;
    /// # use std::str::FromStr;
    /// let itm: PathBuf = "/path/to/source@@john/_dot_item.ext".into();
    /// let base: PathBuf = "/path/to/source".into();
    /// let targetbase: PathBuf = "/path/to/target".into();
    /// let rules = vec![
    ///     RenamingRule{
    ///         pattern: regex::Regex::new("^_dot_").unwrap(),
    ///         substitution: ".".into(),
    ///     },
    ///     RenamingRule{
    ///         pattern: regex::Regex::new("^.").unwrap(),
    ///         substitution: "_dotted_".into(),
    ///     },
    /// ];
    ///
    /// assert_eq!(
    ///     itm.make_target("@@", &base, &targetbase, rules)?,
    ///     PathBuf::from_str("/path/to/target/_dotted_item.ext").unwrap(),
    /// );
    /// # Ok::<(), AppError>(())
    /// ```
    ///
    /// ## Capture groups
    ///
    /// ```rust
    /// # use dt_core::{
    /// #   config::RenamingRule,
    /// #   error::Error as AppError,
    /// #   item::Operate
    /// # };
    /// # use std::path::PathBuf;
    /// # use std::str::FromStr;
    /// let itm: PathBuf = "/path/to/source@@john/_dot_item.ext".into();
    /// let base: PathBuf = "/path/to/source".into();
    /// let targetbase: PathBuf = "/path/to/target".into();
    ///
    /// let named_capture = RenamingRule{
    ///     // Named capture group, captures "dot" into a group with name
    ///     // "prefix".
    ///     pattern: regex::Regex::new("^_(?P<prefix>.*)_").unwrap(),
    ///     substitution: ".${prefix}.".into(),
    /// };
    /// assert_eq!(
    ///     itm.to_owned().make_target(
    ///         "@@",
    ///         &base,
    ///         &targetbase,
    ///         vec![named_capture]
    ///     )?,
    ///     PathBuf::from_str("/path/to/target/.dot.item.ext").unwrap(),
    /// );
    ///
    /// let numbered_capture = RenamingRule{
    ///     // Numbered capture group, where `${0}` references the whole match,
    ///     // other groups are indexed from 1.
    ///     pattern: regex::Regex::new(r#"\.(.*?)$"#).unwrap(),
    ///     substitution: "_${1}_${0}".into(),
    /// };
    /// assert_eq!(
    ///     itm.to_owned().make_target(
    ///         "@@",
    ///         &base,
    ///         &targetbase,
    ///         vec![numbered_capture]
    ///     )?,
    ///     PathBuf::from_str("/path/to/target/_dot_item_ext_.ext").unwrap(),
    /// );
    /// # Ok::<(), AppError>(())
    /// ```
    ///
    /// [renaming rule]: crate::config::RenamingRule
    fn make_target<P: AsRef<Path>>(
        self,
        hostname_sep: &str,
        base: &Self,
        targetbase: P,
        renaming_rules: Vec<RenamingRule>,
    ) -> Result<Self> {
        // Get non-host-specific counterpart of `self`
        let nhself = self.to_owned().non_host_specific(hostname_sep);

        // Get non-host-specific counterpart of `base`
        let base = base.to_owned().non_host_specific(hostname_sep);

        // The tail of the target path, which is the non-host-specific `self`
        // without its `base` prefix path
        let mut tail = nhself.strip_prefix(base)?.to_owned();

        // Apply renaming rules to the tail component
        for rr in renaming_rules {
            log::trace!("Processing renaming rule: {:#?}", rr);
            log::trace!("Before renaming: '{}'", tail.display());

            let RenamingRule {
                pattern,
                substitution,
            } = rr;
            tail = tail
                .iter()
                .map(|comp| {
                    pattern
                        .replace(comp.to_str().unwrap(), &substitution)
                        .into_owned()
                })
                .collect();

            log::trace!("After renaming: '{}'", tail.display());
        }

        // The target is the target base appended with `tail`
        Ok(targetbase.as_ref().join(tail))
    }

    fn render<S: Serialize, R: Register>(
        &self,
        registry: &Rc<R>,
        ctx: &Rc<S>,
    ) -> Result<Vec<u8>> {
        let name = self.to_str().unwrap();
        registry.render(name, ctx)
    }

    /// Populate this item with given group config.  The given group config is
    /// expected to be the group where this item belongs to.
    fn populate<T: Register>(
        &self,
        group: Rc<LocalGroup>,
        registry: Rc<T>,
    ) -> Result<()> {
        // Create possibly missing parent directories along target's path.
        let tpath = self.to_owned().make_target(
            &group.get_hostname_sep(),
            &group.base,
            &group.target,
            group.get_renaming_rules(),
        )?;
        std::fs::create_dir_all(tpath.parent().unwrap())?;
        if group.target.canonicalize()? == group.base.canonicalize()? {
            return Err(AppError::PathError(format!(
                "base directory and its target point to the same path in group '{}'",
                group.name,
            )));
        }

        match group.get_method() {
            SyncMethod::Copy => {
                // `self` is _always_ a file.  If its target path `tpath` is a
                // directory, we should return an error.
                if tpath.is_dir() {
                    return Err(
                        AppError::SyncingError(format!(
                            "a directory '{}' exists at the target path of a source file '{}'",
                            tpath.display(),
                            self.display(),
                        ))
                    );
                }
                if tpath.is_symlink() {
                    log::trace!(
                        "SYNC::COPY [{}]> '{}' is a symlink, removing it",
                        group.name,
                        tpath.display(),
                    );
                    std::fs::remove_file(&tpath)?;
                }
                // Render the template
                let src_content: Vec<u8> = if group.is_templated() {
                    log::trace!(
                        "RENDER [{}]> '{}' with context: {:#?}",
                        group.name,
                        self.display(),
                        group.context,
                    );
                    self.render(&registry, &group.context)?
                } else {
                    log::trace!(
                        "RENDER::SKIP [{}]> '{}'",
                        group.name,
                        self.display(),
                    );
                    std::fs::read(self)?
                };

                if let Ok(dest_content) = std::fs::read(&tpath) {
                    // Check target file's contents, if it has identical
                    // contents as self, there is no need to write to it.
                    if src_content == dest_content {
                        log::trace!(
                            "SYNC::COPY::SKIP [{}]> '{}' has identical content as '{}'",
                            group.name,
                            tpath.display(),
                            self.display(),
                        );
                    } else if std::fs::write(&tpath, &src_content).is_err() {
                        // Contents of target file differs from content of
                        // self, but writing to it failed.  It might be due to
                        // target file being readonly. Attempt to remove it
                        // and try again.
                        log::warn!(
                            "SYNC::COPY::OVERWRITE [{}]> '{}' seems to be readonly, trying to remove it first ..",
                            group.name,
                            tpath.display(),
                        );
                        std::fs::remove_file(&tpath)?;
                        log::trace!(
                            "SYNC::COPY::OVERWRITE [{}]> '{}' => '{}'",
                            group.name,
                            self.display(),
                            tpath.display(),
                        );
                        std::fs::write(&tpath, src_content)?;
                    }
                } else if tpath.exists() {
                    // If read of target file failed but it does exist, then
                    // the target file is probably unreadable. Attempt to
                    // remove it first, then write contents to `tpath`.
                    log::warn!(
                        "SYNC::COPY::OVERWRITE [{}]> Could not read content of target file ('{}'), trying to remove it first ..",
                        group.name,
                        tpath.display(),
                    );
                    std::fs::remove_file(&tpath)?;
                    log::trace!(
                        "SYNC::COPY::OVERWRITE [{}]> '{}' => '{}'",
                        group.name,
                        self.display(),
                        tpath.display(),
                    );
                    std::fs::write(&tpath, src_content)?;
                }
                // If the target file does not exist --- this is the simplest
                // case --- we just write the contents to `tpath`.
                else {
                    log::trace!(
                        "SYNC::COPY [{}]> '{}' => '{}'",
                        group.name,
                        self.display(),
                        tpath.display(),
                    );
                    std::fs::write(&tpath, src_content)?;
                }

                // Copy permissions to target if permission bits do not match.
                let src_perm = self.metadata()?.permissions();
                let dest_perm = tpath.metadata()?.permissions();
                if dest_perm != src_perm {
                    log::trace!(
                        "SYNC::COPY::SETPERM [{}]> source('{:o}') => target('{:o}')",
                        group.name,
                        src_perm.mode(),
                        dest_perm.mode()
                    );
                    if let Err(e) = std::fs::set_permissions(tpath, src_perm)
                    {
                        log::warn!("Could not set permission: {}", e);
                    }
                }
            }
            SyncMethod::Symlink => {
                let staging_path = self.to_owned().make_target(
                    &group.get_hostname_sep(),
                    &group.base,
                    &group.get_staging_dir(),
                    Vec::new(), // Do not apply renaming on staging path
                )?;
                std::fs::create_dir_all(staging_path.parent().unwrap())?;
                if group.global.staging.0.canonicalize()?
                    == group.base.canonicalize()?
                {
                    return Err(AppError::PathError(format!(
                        "base directory and its target point to the same path in group '{}'",
                        group.name,
                    )));
                }
                if group.global.staging.0.canonicalize()?
                    == group.target.canonicalize()?
                {
                    return Err(AppError::PathError(format!(
                        "target directory and staging directory point to the same path in group '{}'",
                        group.name,
                    )));
                }

                // `self` is _always_ a file.  If its target path `tpath` is a
                // directory, we should return an error.
                if tpath.is_dir() {
                    return Err(
                        AppError::SyncingError(format!(
                            "a directory '{}' exists at the target path of a source file '{}'",
                            tpath.display(),
                            self.display(),
                        ))
                    );
                }

                if tpath.exists() && !group.is_overwrite_allowed() {
                    log::warn!(
                        "SYNC::SKIP [{}]> Target path ('{}') exists while `allow_overwrite` is set to false",
                        group.name,
                        tpath.display(),
                    );
                } else {
                    // In this block, either:
                    //
                    //  - `tpath` does not exist
                    //  - `allow_overwrite` is true
                    //
                    // or both are true.
                    //
                    // 1. Staging:
                    //
                    // Check if the content of destination is already the
                    // same as source first.  When the file is large, this
                    // operation is significantly faster than copying to an
                    // existing target file.

                    // Render the template
                    let src_content: Vec<u8> = if group.is_templated() {
                        log::trace!(
                            "RENDER [{}]> '{}' with context: {:#?}",
                            group.name,
                            self.display(),
                            group.context,
                        );
                        self.render(&registry, &group.context)?
                    } else {
                        log::trace!(
                            "RENDER::SKIP [{}]> '{}'",
                            group.name,
                            self.display(),
                        );
                        std::fs::read(self)?
                    };

                    if let Ok(dest_content) = std::fs::read(&staging_path) {
                        // Check staging file's contents, if it has identical
                        // contents as self, there is no need to write to it.
                        if src_content == dest_content {
                            log::trace!(
                                "SYNC::STAGE::SKIP [{}]> '{}' has identical content as '{}'",
                                group.name,
                                staging_path.display(),
                                self.display(),
                            );
                        } else if std::fs::write(&staging_path, &src_content)
                            .is_err()
                        {
                            // Contents of staging file differs from content
                            // of self, but writing to it failed.  It might be
                            // due to staging file being readonly. Attempt to
                            // remove it and try again.
                            log::warn!(
                                "SYNC::STAGE::OVERWRITE [{}]> '{}' seems to be readonly, trying to remove it first ..",
                                group.name,
                                staging_path.display(),
                            );
                            std::fs::remove_file(&staging_path)?;
                            log::trace!(
                                "SYNC::STAGE [{}]> '{}' => '{}'",
                                group.name,
                                self.display(),
                                staging_path.display(),
                            );
                            std::fs::write(&staging_path, src_content)?;
                        }
                    } else if staging_path.exists() {
                        // If read of staging file failed but it does exist,
                        // then the staging file is probably unreadable.
                        // Attempt to remove it first, then write contents to
                        // `staging_path`.
                        log::warn!(
                            "SYNC::STAGE::OVERWRITE [{}]> Could not read content of staging file ('{}'), trying to remove it first ..",
                            group.name,
                            staging_path.display(),
                        );
                        std::fs::remove_file(&staging_path)?;
                        log::trace!(
                            "SYNC::STAGE::OVERWRITE [{}]> '{}' => '{}'",
                            group.name,
                            self.display(),
                            staging_path.display(),
                        );
                        std::fs::write(&staging_path, src_content)?;
                    }
                    // If the staging file does not exist --- this is the
                    // simplest case --- we just write the contents to
                    // `staging_path`.
                    else {
                        log::trace!(
                            "SYNC::STAGE [{}]> '{}' => '{}'",
                            group.name,
                            self.display(),
                            staging_path.display(),
                        );
                        std::fs::write(&staging_path, src_content)?;
                    }

                    // Copy permissions to staging file if permission bits do
                    // not match.
                    let src_perm = self.metadata()?.permissions();
                    let dest_perm = staging_path.metadata()?.permissions();
                    if dest_perm != src_perm {
                        log::trace!(
                            "SYNC::STAGE::SETPERM [{}]> source('{:o}') => staging('{:o}')",
                            group.name,
                            src_perm.mode(),
                            dest_perm.mode()
                        );
                        if let Err(e) =
                            std::fs::set_permissions(&staging_path, src_perm)
                        {
                            log::warn!("Could not set permission: {}", e);
                        }
                    }

                    // 2. Symlinking
                    //
                    // Do not remove target file if it is already a symlink
                    // that points to the correct location.
                    if let Ok(dest) = std::fs::read_link(&tpath) {
                        if dest == staging_path {
                            log::trace!(
                                "SYNC::SYMLINK::SKIP [{}]> '{}' is already a symlink pointing to '{}'",
                                group.name,
                                tpath.display(),
                                staging_path.display(),
                            );
                        } else {
                            log::trace!(
                                "SYNC::SYMLINK::OVERWRITE [{}]> '{}' => '{}'",
                                group.name,
                                staging_path.display(),
                                tpath.display(),
                            );
                            std::fs::remove_file(&tpath)?;
                            std::os::unix::fs::symlink(
                                &staging_path,
                                &tpath,
                            )?;
                        }
                    }
                    // If target file exists but is not a symlink, try to
                    // remove it first, then make a symlink from
                    // `staging_path` to `tpath`.
                    else if tpath.exists() {
                        log::trace!(
                            "SYNC::SYMLINK::OVERWRITE [{}]> '{}' => '{}'",
                            group.name,
                            staging_path.display(),
                            tpath.display(),
                        );
                        std::fs::remove_file(&tpath)?;
                        std::os::unix::fs::symlink(&staging_path, &tpath)?;
                    }
                    // The final case is that when `tpath` does not exist
                    // yet, we can then directly create a symlink.
                    else {
                        log::trace!(
                            "SYNC::SYMLINK [{}]> '{}' => '{}'",
                            group.name,
                            staging_path.display(),
                            tpath.display(),
                        );
                        std::os::unix::fs::symlink(&staging_path, &tpath)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Show what is to be done if this item is to be populated with given
    /// group config.  The given group config is expected to be the group
    /// where this item belongs to.
    fn populate_dry(&self, group: Rc<LocalGroup>) -> Result<()> {
        let tpath = self.to_owned().make_target(
            &group.get_hostname_sep(),
            &group.base,
            &group.target,
            group.get_renaming_rules(),
        )?;
        if tpath.exists() {
            if group.is_overwrite_allowed() {
                if tpath.is_dir() {
                    log::error!(
                        "DRYRUN [{}]> A directory ('{}') exists at the target path of a source file ('{}')",
                        group.name,
                        tpath.display(),
                        self.display(),
                    );
                } else {
                    log::debug!(
                        "DRYRUN [{}]> '{}' -> '{}'",
                        group.name,
                        self.display(),
                        tpath.display(),
                    );
                }
            } else {
                log::error!(
                    "DRYRUN [{}]> Target path ('{}') exists while `allow_overwrite` is set to false",
                    group.name,
                    tpath.display(),
                );
            }
        } else {
            log::debug!(
                "DRYRUN [{}]> '{}' -> '{}'",
                group.name,
                self.display(),
                tpath.display(),
            );
        }

        Ok(())
    }
}

impl Operate for Url {}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Oct 29 2021, 22:56 [CST]
