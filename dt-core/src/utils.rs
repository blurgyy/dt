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
///  std::env::set_var("HOME", "/tmp/john");
///  std::env::set_var("XDG_CONFIG_HOME", "/tmp/watson/.config");
///  assert_eq!(
///      default_config_path("cli.toml"),
///      PathBuf::from_str("/tmp/watson/.config/dt/cli.toml").unwrap(),
///  );
///
///  std::env::remove_var("XDG_CONFIG_HOME");
///  assert_eq!(
///      default_config_path("cli.toml"),
///      PathBuf::from_str("/tmp/john/.config/dt/cli.toml").unwrap(),
///  );
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

#[cfg(test)]
pub(crate) mod testing {
    use std::{
        fs::Permissions, os::unix::prelude::PermissionsExt, path::PathBuf,
    };

    use color_eyre::Report;

    const TESTROOT: &str = "/tmp/dt-testing/syncing";

    pub fn get_testroot() -> PathBuf {
        TESTROOT.into()
    }

    pub fn prepare_directory(
        abspath: PathBuf,
        mode: u32,
    ) -> Result<PathBuf, Report> {
        std::fs::create_dir_all(&abspath)?;
        std::fs::set_permissions(&abspath, Permissions::from_mode(mode))?;
        Ok(abspath)
    }

    pub fn prepare_file(
        abspath: PathBuf,
        mode: u32,
    ) -> Result<PathBuf, Report> {
        if let Some(parent) = abspath.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(
            &abspath,
            "Created by: `dt_core::syncing::tests::prepare_file`\n",
        )?;
        std::fs::set_permissions(&abspath, Permissions::from_mode(mode))?;
        Ok(abspath)
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Oct 03 2021, 02:54 [CST]
