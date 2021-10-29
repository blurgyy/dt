use std::path::{Path, PathBuf};

/// Gets the default config file path, with last component specified by
/// `filename`.
pub fn default_config_path(filename: impl AsRef<Path>) -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| panic!("Cannot determine default config path"))
        .join("dt")
        .join(filename)
}

pub fn host_specific_suffix(hostname_sep: &str) -> String {
    hostname_sep.to_owned()
        + gethostname::gethostname()
            .to_str()
            .expect("Failed getting hostname")
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Oct 03 2021, 02:54 [CST]
