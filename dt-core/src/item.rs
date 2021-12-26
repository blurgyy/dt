use std::path::{Path, PathBuf};

use path_clean::PathClean;

use crate::{config::RenamingRule, error, utils};

type Result<T> = std::result::Result<T, error::Error>;

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
    /// [renaming rules]: crate::config::RenamingRule
    fn make_target<T>(
        &self,
        hostname_sep: &str,
        basedir: T,
        targetbase: T,
        renaming_rules: Option<Vec<RenamingRule>>,
    ) -> Result<Self>
    where
        T: Into<Self> + AsRef<Path>,
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
                log::trace!("Before renaming: {}", tail.display());

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
}

impl<'a> DTItem<'a> for PathBuf {}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Oct 29 2021, 22:56 [CST]
