use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

use content_inspector::inspect;
use minijinja::Environment;
use path_clean::PathClean;
use serde::Serialize;

use crate::{
    config::{LocalGroup, RenamingRule, SyncMethod},
    error::{Error as AppError, Result},
    utils,
};

/// Defines behaviours for an item (a path) used in [DT].
///
/// [DT]: https://github.com/blurgyy/dt
pub trait DTItem<'a>
where
    Self: AsRef<Path> + From<&'a Path> + From<PathBuf>,
{
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
        let path = self.as_ref();
        let filename = path
            .file_name()
            .unwrap_or_else(|| {
                panic!(
                    "Failed extracting file name from path '{}'",
                    path.display(),
                )
            })
            .to_str()
            .unwrap_or_else(|| {
                panic!(
                    "Failed converting &OsStr to &str for path '{}'",
                    path.display(),
                )
            });
        let splitted: Vec<_> = filename.split(hostname_sep).collect();

        assert!(
        splitted.len() <= 2,
        "There appears to be more than 1 occurrences of hostname_sep ({}) in this path: {}",
        hostname_sep,
        path.display(),
    );
        assert!(
            !splitted.first().unwrap().is_empty(),
            "hostname_sep ({}) appears to be a prefix of this path: {}",
            hostname_sep,
            path.display(),
        );

        splitted.len() > 1
            && splitted.last() != gethostname::gethostname().to_str().as_ref()
    }

    /// Gets the absolute path of `self`, **without** traversing symlinks.
    ///
    /// Reference: <https://stackoverflow.com/a/54817755/13482274>
    fn absolute(&self) -> Result<Self> {
        let path = self.as_ref();

        let absolute_path = if path.is_absolute() {
            path.to_owned()
        } else {
            std::env::current_dir()?.join(path)
        }
        .clean();

        Ok(absolute_path.into())
    }

    /// Gets the host-specific counterpart of `self`.  If `self` is already
    /// host-specific, returns `self` directly.
    fn host_specific(&'a self, hostname_sep: &'a str) -> Self {
        let path = self.as_ref();

        if path.ends_with(utils::host_specific_suffix(hostname_sep)) {
            path.into()
        } else {
            let hs_filename = path
                .file_name()
                .unwrap_or_else(|| {
                    panic!(
                        "Failed extracting file name from path '{}'",
                        path.display(),
                    )
                })
                .to_str()
                .unwrap_or_else(|| {
                    panic!(
                        "Failed converting &OsStr to &str for path: '{}'",
                        path.display(),
                    )
                })
                .to_owned()
                + &utils::host_specific_suffix(hostname_sep);

            path.with_file_name(hs_filename).into()
        }
    }

    /// Converts a path to a non-host-specific path.  If the input path is
    /// already non-host-specific, returns itself;  Otherwise returns a
    /// path where every component of the path is converted to a
    /// non-host-specific one.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use dt_core::item::DTItem;
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
    fn non_host_specific(&self, hostname_sep: &str) -> Self {
        let path = self.as_ref();
        path
            .iter()
            .map(std::ffi::OsStr::to_str)
            .map(|s| {
                s.unwrap_or_else(|| {
                    panic!(
                        "Failed extracting path components from '{}'",
                        path.display(),
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
                            path.display(),
                        )
                    })
                    .to_owned()
            })
            .collect::<PathBuf>().into()
    }

    /// Checks whether any of the component above `self` is readonly.
    fn parent_readonly(&self) -> bool {
        let mut p = self.as_ref();
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

    /// Given a `hostname_sep`, a `basedir`, a `targetbase`, and optionally a
    /// list of [renaming rules], create the path where `self` would be synced
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
    /// #   item::DTItem
    /// # };
    /// # use std::path::PathBuf;
    /// # use std::str::FromStr;
    /// let itm: PathBuf = "/path/to/source@@john/item".into();
    /// let basedir: PathBuf = "/path/to/source".into();
    /// let targetbase: PathBuf = "/path/to/target".into();
    ///
    /// assert_eq!(
    ///     itm.make_target("@@", basedir, targetbase, None)?,
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
    /// #   item::DTItem
    /// # };
    /// # use std::path::PathBuf;
    /// # use std::str::FromStr;
    /// let itm: PathBuf = "/path/to/source@@john/_dot_item".into();
    /// let basedir: PathBuf = "/path/to/source".into();
    /// let targetbase: PathBuf = "/path/to/target".into();
    /// let rules = vec![
    ///     RenamingRule{
    ///         pattern: regex::Regex::new("^_dot_").unwrap(),
    ///         substitution: ".".into(),
    ///     },
    /// ];
    ///
    /// assert_eq!(
    ///     itm.make_target("@@", basedir, targetbase, Some(rules))?,
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
    /// #   item::DTItem
    /// # };
    /// # use std::path::PathBuf;
    /// # use std::str::FromStr;
    /// let itm: PathBuf = "/path/to/source@@john/_dot_item.ext".into();
    /// let basedir: PathBuf = "/path/to/source".into();
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
    ///     itm.make_target("@@", basedir, targetbase, Some(rules))?,
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
    /// #   item::DTItem
    /// # };
    /// # use std::path::PathBuf;
    /// # use std::str::FromStr;
    /// let itm: PathBuf = "/path/to/source@@john/_dot_item.ext".into();
    /// let basedir: PathBuf = "/path/to/source".into();
    /// let targetbase: PathBuf = "/path/to/target".into();
    ///
    /// let named_capture = RenamingRule{
    ///     // Named capture group, captures "dot" into a group with name
    ///     // "prefix".
    ///     pattern: regex::Regex::new("^_(?P<prefix>.*)_").unwrap(),
    ///     substitution: ".${prefix}.".into(),
    /// };
    /// assert_eq!(
    ///     itm.make_target("@@", &basedir, &targetbase, Some(vec![named_capture]))?,
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
    ///     itm.make_target("@@", basedir, targetbase, Some(vec![numbered_capture]))?,
    ///     PathBuf::from_str("/path/to/target/_dot_item_ext_.ext").unwrap(),
    /// );
    /// # Ok::<(), AppError>(())
    /// ```
    ///
    /// [renaming rules]: crate::config::RenamingRule
    fn make_target<T>(
        &self,
        hostname_sep: &str,
        basedir: T,
        targetbase: T,
        renaming_rules: Option<Vec<RenamingRule>>,
    ) -> Result<Self>
    where
        T: Into<PathBuf> + AsRef<Path>,
    {
        // Get non-host-specific counterpart of `self`
        let nhself = self.non_host_specific(hostname_sep);

        // Get non-host-specific counterpart of `basedir`
        let basedir = basedir.into().non_host_specific(hostname_sep);

        // The tail of the target path, which is the non-host-specific `self`
        // without its `basedir` prefix path
        let mut tail = nhself.as_ref().strip_prefix(basedir)?.to_owned();

        // Apply renaming rules to the tail component
        if let Some(renaming_rules) = renaming_rules {
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
        }

        // The target is the target base appended with `tail`
        Ok(targetbase.as_ref().join(tail).into())
    }

    /// Render this item with given context to the `dest` path.
    ///
    /// Rendering only happens if this item is considered as a plain text
    /// file.  If this item is considered as a binary file, it's original
    /// content is returned.  The content type is inspected via the
    /// [`content_inspector`] crate.  Although it can correctly determine if
    /// an item is binary or text mostly of the time, it is just a heuristic
    /// check and can fail in some cases, e.g. NUL byte in the first 1024
    /// bytes of a UTF-8-encoded text file, etc..  See [the crate's home page]
    /// for the full caveats.
    ///
    /// [`content_inspector`]: https://crates.io/crates/content_inspector
    /// [the crate's home page]: https://github.com/sharkdp/content_inspector
    // TODO: Add `force_rendering` or something to also render binary files.
    fn render<S: Serialize>(&self, ctx: &Rc<S>) -> Result<Vec<u8>> {
        let name = self.as_ref().to_str().unwrap();
        let mut env = Environment::new();
        let original_content = std::fs::read(self.as_ref())?;
        if inspect(&original_content).is_text() {
            env.add_template(name, std::str::from_utf8(&original_content)?)?;
            Ok(env.get_template(name)?.render(&**ctx)?.into())
        } else {
            Ok(original_content)
        }
    }

    /// Populate this item with given group config.  The given group config is
    /// expected to be the group where this item belongs to.
    fn populate(&self, group: Rc<LocalGroup>) -> Result<()> {
        // Create possibly missing parent directories along target's path.
        let tpath = self.make_target(
            &group.get_hostname_sep(),
            &group.basedir,
            &group.target,
            Some(group.get_renaming_rules()),
        )?;
        std::fs::create_dir_all(tpath.as_ref().parent().unwrap())?;

        match group.get_method() {
            SyncMethod::Copy => {
                // `self` is _always_ a file.  If its target path `tpath` is a
                // directory, we should return an error.
                if tpath.as_ref().is_dir() {
                    return Err(
                        AppError::SyncingError(format!(
                            "a directory '{}' exists at the target path of a source file '{}'",
                            tpath.as_ref().display(),
                            self.as_ref().display(),
                        ))
                    );
                }
                if tpath.as_ref().is_symlink() {
                    log::trace!(
                        "SYNC::COPY [{}]> '{}' is a symlink, removing it",
                        group.name,
                        tpath.as_ref().display(),
                    );
                    std::fs::remove_file(tpath.as_ref())?;
                }
                // Render the template
                let src_content: Vec<u8> = self.render(&group.context)?;
                if let Ok(dest_content) = std::fs::read(tpath.as_ref()) {
                    if src_content == dest_content {
                        log::trace!(
                            "SYNC::COPY::SKIP [{}]> '{}' has identical content as '{}'",
                            group.name,
                            tpath.as_ref().display(),
                            self.as_ref().display(),
                        );
                    } else if std::fs::write(tpath.as_ref(), &src_content)
                        .is_err()
                    {
                        log::warn!(
                            "SYNC::COPY::OVERWRITE [{}]> '{}' seems to be readonly, trying to remove it first ..",
                            group.name,
                            tpath.as_ref().display(),
                        );
                        std::fs::remove_file(tpath.as_ref())?;
                        log::trace!(
                            "SYNC::COPY::OVERWRITE [{}]> '{}' => '{}'",
                            group.name,
                            self.as_ref().display(),
                            tpath.as_ref().display(),
                        );
                        std::fs::write(tpath.as_ref(), src_content)?;
                    }
                } else if tpath.as_ref().exists() {
                    log::warn!(
                        "SYNC::COPY::OVERWRITE [{}]> Could not read content of target file ('{}'), trying to remove it first ..",
                        group.name,
                        tpath.as_ref().display(),
                    );
                    std::fs::remove_file(tpath.as_ref())?;
                    log::trace!(
                        "SYNC::COPY::OVERWRITE [{}]> '{}' => '{}'",
                        group.name,
                        self.as_ref().display(),
                        tpath.as_ref().display(),
                    );
                    std::fs::write(tpath.as_ref(), src_content)?;
                } else {
                    log::trace!(
                        "SYNC::COPY [{}]> '{}' => '{}'",
                        group.name,
                        self.as_ref().display(),
                        tpath.as_ref().display(),
                    );
                    std::fs::write(tpath.as_ref(), src_content)?;
                }
            }
            SyncMethod::Symlink => {
                let staging_path = self.make_target(
                    &group.get_hostname_sep(),
                    &group.basedir,
                    &group
                        .global
                        .staging
                        .as_ref()
                        .unwrap()
                        .join(PathBuf::from(&group.name)),
                    None, // Do not apply renaming on staging path
                )?;
                std::fs::create_dir_all(
                    staging_path.as_ref().parent().unwrap(),
                )?;

                // `self` is _always_ a file.  If its target path `tpath` is a
                // directory, we should return an error.
                if tpath.as_ref().is_dir() {
                    return Err(
                        AppError::SyncingError(format!(
                            "a directory '{}' exists at the target path of a source file '{}'",
                            tpath.as_ref().display(),
                            self.as_ref().display(),
                        ))
                    );
                }

                if tpath.as_ref().exists() && !group.get_allow_overwrite() {
                    log::warn!(
                        "SYNC::SKIP [{}]> Target path ('{}') exists while `allow_overwrite` is set to false",
                        group.name,
                        tpath.as_ref().display(),
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
                    let src_content: Vec<u8> = self.render(&group.context)?;
                    if let Ok(dest_content) = std::fs::read(&staging_path) {
                        if src_content == dest_content {
                            log::trace!(
                                "SYNC::STAGE::SKIP [{}]> '{}' has identical content as '{}'",
                                group.name,
                                staging_path.as_ref().display(),
                                self.as_ref().display(),
                            );
                        } else if std::fs::write(
                            staging_path.as_ref(),
                            &src_content,
                        )
                        .is_err()
                        {
                            log::warn!(
                                "SYNC::STAGE::OVERWRITE [{}]> '{}' seems to be readonly, trying to remove it first ..",
                                group.name,
                                staging_path.as_ref().display(),
                            );
                            std::fs::remove_file(staging_path.as_ref())?;
                            log::trace!(
                                "SYNC::STAGE [{}]> '{}' => '{}'",
                                group.name,
                                self.as_ref().display(),
                                staging_path.as_ref().display(),
                            );
                            std::fs::write(
                                staging_path.as_ref(),
                                src_content,
                            )?;
                        }
                    }
                    // If read of staging file failed but it does exist, then
                    // the staging file is probably unreadable, so try to
                    // remove it first, then copy content to `staging_path`.
                    else if staging_path.as_ref().exists() {
                        log::warn!(
                            "SYNC::STAGE::OVERWRITE [{}]> Could not read content of staging file ('{}'), trying to remove it first ..",
                            group.name,
                            staging_path.as_ref().display(),
                        );
                        std::fs::remove_file(staging_path.as_ref())?;
                        log::trace!(
                            "SYNC::STAGE::OVERWRITE [{}]> '{}' => '{}'",
                            group.name,
                            self.as_ref().display(),
                            staging_path.as_ref().display(),
                        );
                        std::fs::write(staging_path.as_ref(), src_content)?;
                    }
                    // If the staging file does not exist --- this is the
                    // simplest case --- we just copy this file to the
                    // `staging_path`.
                    else {
                        log::trace!(
                            "SYNC::STAGE [{}]> '{}' => '{}'",
                            group.name,
                            self.as_ref().display(),
                            staging_path.as_ref().display(),
                        );
                        std::fs::write(staging_path.as_ref(), src_content)?;
                    }

                    // 2. Symlinking
                    //
                    // Do not remove target file if it is already a symlink
                    // that points to the correct location.
                    if let Ok(dest) = std::fs::read_link(&tpath) {
                        if dest == staging_path.as_ref() {
                            log::trace!(
                                "SYNC::SYMLINK::SKIP [{}]> '{}' is already a symlink pointing to '{}'",
                                group.name,
                                tpath.as_ref().display(),
                                staging_path.as_ref().display(),
                            );
                        } else {
                            log::trace!(
                                "SYNC::SYMLINK::OVERWRITE [{}]> '{}' => '{}'",
                                group.name,
                                staging_path.as_ref().display(),
                                tpath.as_ref().display(),
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
                    else if tpath.as_ref().exists() {
                        log::trace!(
                            "SYNC::SYMLINK::OVERWRITE [{}]> '{}' => '{}'",
                            group.name,
                            staging_path.as_ref().display(),
                            tpath.as_ref().display(),
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
                            staging_path.as_ref().display(),
                            tpath.as_ref().display(),
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
        let tpath = self.make_target(
            &group.get_hostname_sep(),
            &group.basedir,
            &group.target,
            Some(group.get_renaming_rules()),
        )?;
        if tpath.as_ref().exists() {
            if group.get_allow_overwrite() {
                if tpath.as_ref().is_dir() {
                    log::error!(
                        "DRYRUN [{}]> A directory ('{}') exists at the target path of a source file ('{}')",
                        group.name,
                        tpath.as_ref().display(),
                        self.as_ref().display(),
                    );
                } else {
                    log::debug!(
                        "DRYRUN [{}]> '{}' -> '{}'",
                        group.name,
                        self.as_ref().display(),
                        tpath.as_ref().display(),
                    );
                }
            } else {
                log::error!(
                    "DRYRUN [{}]> Target path ('{}') exists while `allow_overwrite` is set to false",
                    group.name,
                    tpath.as_ref().display(),
                );
            }
        } else {
            log::debug!(
                "DRYRUN [{}]> '{}' -> '{}'",
                group.name,
                self.as_ref().display(),
                tpath.as_ref().display(),
            );
        }

        Ok(())
    }
}

impl<'a> DTItem<'a> for PathBuf {}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Oct 29 2021, 22:56 [CST]
