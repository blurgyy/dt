use path_clean::PathClean;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

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
) -> std::io::Result<PathBuf> {
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
        + hostname_sep
        + gethostname::gethostname()
            .to_str()
            .expect("Failed extracting string from `gethostname`");

    Ok(path.with_file_name(hs_filename))
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Oct 03 2021, 02:54 [CST]
