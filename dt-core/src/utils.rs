use color_eyre::Report;
use path_clean::PathClean;
use std::{
    ops::Not,
    path::{Path, PathBuf},
};

/// Gets the default config file path, with last component specified by
/// `filename`.
pub fn default_config_path(filename: impl AsRef<Path>) -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| panic!("Cannot determine default config path"))
        .join("dt")
        .join(filename)
}

fn host_specific_suffix(hostname_sep: &str) -> Result<String, Report> {
    Ok(hostname_sep.to_owned()
        + gethostname::gethostname()
            .to_str()
            .expect("Failed getting hostname"))
}

/// Checks if the item is for another machine (by checking its name).
///
/// A host-specific item is considered for another machine, when its filename
/// contains only 1 `hostname_sep`, and after the `hostname_sep` should not be
/// current machine's hostname.
///
/// A non-host-specific item is always considered **not** for another machine.
///
/// An item with filename containing more than 1 `hostname_sep` causes this
/// function to panic.
pub fn is_for_other_host(path: impl AsRef<Path>, hostname_sep: &str) -> bool {
    let path = path.as_ref();
    let filename = path
        .file_name()
        .unwrap_or_else(|| {
            panic!("Failed extracting file name from path {}", path.display())
        })
        .to_str()
        .unwrap_or_else(|| {
            panic!(
                "Failed converting &OsStr to &str for path: {}",
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
        splitted.first().unwrap().is_empty().not(),
        "hostname_sep ({}) appears to be a prefix of this path: {}",
        hostname_sep,
        path.display(),
    );

    splitted.len() > 1
        && splitted.last() != gethostname::gethostname().to_str().as_ref()
}

/// Convert a path relative to the current directory to an absolute one.
///
/// <https://stackoverflow.com/a/54817755/13482274>
pub fn to_absolute(path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    let path = path.as_ref();

    let absolute_path = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    }
    .clean();

    Ok(absolute_path)
}

/// Converts a path to a host-specific path.  If the input path is already
/// host-specific, returns itself;  Otherwise returns the path's name appended
/// with "${hostname_sep}$(hostname)".
pub fn to_host_specific(
    path: impl AsRef<Path>,
    hostname_sep: &str,
) -> Result<PathBuf, Report> {
    let path = path.as_ref();

    if path.ends_with(host_specific_suffix(hostname_sep)?) {
        Ok(path.to_owned())
    } else {
        let hs_filename = path
            .file_name()
            .unwrap_or_else(|| {
                panic!(
                    "Failed extracting file name from path {}",
                    path.display(),
                )
            })
            .to_str()
            .unwrap_or_else(|| {
                panic!(
                    "Failed converting &OsStr to &str for path: {}",
                    path.display(),
                )
            })
            .to_owned()
            + &host_specific_suffix(hostname_sep)?;

        Ok(path.with_file_name(hs_filename))
    }
}

/// Converts a path to a non-host-specific path.  If the input path is already
/// non-host-specific, returns itself;  Otherwise returns a path where every
/// component of the path is converted to non-host-specific one.
///
/// ```rust
/// # use color_eyre::Report;
/// # use std::path::PathBuf;
/// # use std::str::FromStr;
/// # use dt_core::utils;
/// #
/// # fn main() -> Result<(), Report> {
///     let p: PathBuf = "/some@@watson/long/path@@watson".into();
///     let h = utils::to_non_host_specific(p, "@@")?;
///
///     assert_eq!(h, PathBuf::from_str("/some/long/path")?);
///
/// #     Ok(())
/// # }
/// ```
pub fn to_non_host_specific(
    path: impl AsRef<Path>,
    hostname_sep: &str,
) -> Result<PathBuf, Report> {
    let path = path.as_ref();
    Ok(path
        .iter()
        .map(std::ffi::OsStr::to_str)
        .map(|s| {
            s.unwrap_or_else(|| {
                panic!(
                    "Failed extracting path components from {}",
                    path.display()
                )
            })
        })
        .map(|s| {
            s.split(hostname_sep)
                .collect::<Vec<_>>()
                .first()
                .unwrap_or_else(|| panic!("Failed extracting basename from component {} of path {}", s, path.display()))
                .to_owned()
        })
        .collect())
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Oct 03 2021, 02:54 [CST]
