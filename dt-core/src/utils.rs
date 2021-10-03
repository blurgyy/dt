use color_eyre::Report;
use path_clean::PathClean;
use std::path::{Path, PathBuf};

fn host_specific_suffix(hostname_sep: &str) -> Result<String, Report> {
    Ok(hostname_sep.to_owned()
        + gethostname::gethostname()
            .to_str()
            .expect("Failed getting hostname"))
}

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

pub fn to_host_specific(
    path: impl AsRef<Path>,
    hostname_sep: &str,
) -> Result<PathBuf, Report> {
    let path = path.as_ref();

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
