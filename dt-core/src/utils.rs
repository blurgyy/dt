use color_eyre::Report;
use path_clean::PathClean;
use std::path::{Path, PathBuf};

fn host_specific_suffix(hostname_sep: &str) -> Result<String, Report> {
    Ok(hostname_sep.to_owned()
        + gethostname::gethostname()
            .to_str()
            .expect("Failed getting hostname"))
}

/// Checks if the item is for another machine (by checking its name).
///
/// A host-specific item is considered for another machine, when its filename contains only 1
/// `hostname_sep`, and after the `hostname_sep` should not be current machine's hostname.
///
/// A non-host-specific item is always considered not for another machine.
///
/// An item with filename containing more than 1 `hostname_sep` causes this function to panic.
pub fn is_for_other_host(path: impl AsRef<Path>, hostname_sep: &str) -> bool {
    let path = path.as_ref();
    let filename = path
        .file_name()
        .expect(&format!(
            "Failed extracting file name from path {}",
            path.display()
        ))
        .to_str()
        .expect(&format!(
            "Failed converting &OsStr to &str for path: {}",
            path.display(),
        ));
    let splitted: Vec<_> = filename.split(hostname_sep).collect();

    assert!(splitted.len() <= 2);

    splitted.last() == gethostname::gethostname().to_str().as_ref()
}

/// Convert a path relative to the current directory to an absolute one.
///
/// Reference: https://stackoverflow.com/a/54817755/13482274
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

/// Converts a path to a host-specific path.  If the input path is already host-specific, returns
/// itself;  Otherwise returns the path's name appended with "${hostname_sep}$(hostname)".
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
            .expect(&format!(
                "Failed extracting file name from path {}",
                path.display(),
            ))
            .to_str()
            .expect(&format!(
                "Failed converting &OsStr to &str for path: {}",
                path.display(),
            ))
            .to_owned()
            + &host_specific_suffix(hostname_sep)?;

        Ok(path.with_file_name(hs_filename))
    }
}

/// Converts a path to a non-host-specific path.  If the input path is already non-host-specific,
/// returns itself;  Otherwise returns the path's name before `hostname_sep`.
pub fn to_non_host_specific(
    path: impl AsRef<Path>,
    hostname_sep: &str,
) -> Result<PathBuf, Report> {
    let path = path.as_ref();
    Ok(path.with_file_name(
        path.file_name()
            .expect(&format!(
                "Failed extracting file name from path {}",
                path.display(),
            ))
            .to_str()
            .expect(&format!(
                "Failed converting &OsStr to &str for path: {}",
                path.display(),
            ))
            .split(hostname_sep)
            // First item from separated filename
            .nth(0)
            .expect("Cannot get non-host-specific path"),
    ))
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Oct 03 2021, 02:54 [CST]
