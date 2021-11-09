use std::path::{Path, PathBuf};

/// Gets the default config file path, according to `$XDG_CONFIG_HOME` or
/// `$HOME`, with last component specified by `filename`.
///
/// # Example
///
/// ```rust
/// # use dt_core::utils::default_config_path;
/// # use std::path::PathBuf;
/// # use std::str::FromStr;
/// #
/// # fn main() {
///     std::env::set_var("HOME", "/tmp/john");
///     std::env::set_var("XDG_CONFIG_HOME", "/tmp/watson/.config");
///     assert_eq!(
///         default_config_path("cli.toml"),
///         PathBuf::from_str("/tmp/watson/.config/dt/cli.toml").unwrap(),
///     );
///
///     std::env::remove_var("XDG_CONFIG_HOME");
///     assert_eq!(
///         default_config_path("cli.toml"),
///         PathBuf::from_str("/tmp/john/.config/dt/cli.toml").unwrap(),
///     );
/// # }
/// ```
pub fn default_config_path(filename: impl AsRef<Path>) -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| panic!("Cannot determine default config path"))
        .join("dt")
        .join(filename)
}

/// Gets the host-specific suffix, according to given [`hostname_sep`] and
/// current machine's hostname.
///
/// [`hostname_sep`]: crate::config::GlobalConfig::hostname_sep
pub fn host_specific_suffix(hostname_sep: &str) -> String {
    hostname_sep.to_owned()
        + gethostname::gethostname()
            .to_str()
            .expect("Failed getting hostname")
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Oct 03 2021, 02:54 [CST]
