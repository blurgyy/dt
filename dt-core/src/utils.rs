use std::path::{Path, PathBuf};

use crate::error::{Error as AppError, Result};

/// Gets config path from environment variables, or infer one.
///
/// 1. If the environment variable indexed by `env_for_file`'s value is
/// present, that environment variable's value is returned as the config file
/// path.
///
/// # Example
///
/// ```
/// # use dt_core::utils::default_config_path;
/// # use std::path::PathBuf;
/// # use std::str::FromStr;
/// std::env::set_var("DT_CLI_CONFIG_PATH", "/tmp/dt/configuration.toml");
/// assert_eq!(
///     default_config_path::<&str>("DT_CLI_CONFIG_PATH", "", &[]),
///     Ok(PathBuf::from_str("/tmp/dt/configuration.toml").unwrap()),
/// );
/// ```
///
/// 2. Otherwise, if the environment variable indexed by `env_for_dir`'s value
/// is present, that environment variable's value is considered the parent
/// directory of the returned config path, filenames within `search_list` will
/// be checked in order and the first existing file's path will be returned.
/// If none of the `search_list` exists, a fallback filename `config.toml`
/// will be used.
///
/// # Example
///
/// ```
/// # use dt_core::utils::default_config_path;
/// # use std::path::PathBuf;
/// # use std::str::FromStr;
/// std::env::set_var("DT_CONFIG_DIR", "/tmp/d/t");
/// assert_eq!(
///     default_config_path::<&str>(
///         "some_non_existing_var",
///         "DT_CONFIG_DIR",
///         &[],
///     ),
///     Ok(PathBuf::from_str("/tmp/d/t/config.toml").unwrap()),
/// );
/// ```
///
/// 3. When neither of `env_for_file`'s and `env_for_dir`'s corresponding
/// environment variable exists, the parent directory of returned path is
/// inferred as `$XDG_CONFIG_HOME/dt`, or `$HOME/.config/dt` if
/// XDG_CONFIG_HOME is not set in the runtime environment.
///
/// # Example
///
/// ```
/// # use dt_core::utils::default_config_path;
/// # use std::path::PathBuf;
/// # use std::str::FromStr;
/// std::env::set_var("XDG_CONFIG_HOME", "/tmp/confighome");
/// assert_eq!(
///     default_config_path::<&str>(
///         "some_non_existing_var",
///         "some_other_non_existing_var",
///         &[],
///     ),
///     Ok(PathBuf::from_str("/tmp/confighome/dt/config.toml").unwrap()),
/// );
///
/// std::env::remove_var("XDG_CONFIG_HOME");
/// std::env::set_var("HOME", "/tmp/home");
/// assert_eq!(
///     default_config_path::<&str>(
///         "some_non_existing_var",
///         "some_other_non_existing_var",
///         &[],
///     ),
///     Ok(PathBuf::from_str("/tmp/home/.config/dt/config.toml").unwrap()),
/// );
/// ```
pub fn default_config_path<P: AsRef<Path>>(
    env_for_file: &str,
    env_for_dir: &str,
    search_list: &[P],
) -> Result<PathBuf> {
    if let Ok(file_path) = std::env::var(env_for_file) {
        log::debug!(
            "Using config file '{}' (from environment variable `{}`)",
            file_path,
            env_for_file,
        );
        Ok(file_path.into())
    } else {
        let dir_path = match std::env::var(env_for_dir) {
            Ok(dir_path) => {
                log::debug!(
                    "Using config directory '{}' (from environment variable `{}`)",
                    dir_path,
                    env_for_dir,
                );
                dir_path.into()
            }
            _ => {
                if let Some(dir_path) = dirs::config_dir() {
                    log::debug!(
                        "Using config directory '{}' (inferred)",
                        dir_path.display(),
                    );
                    dir_path.join("dt")
                } else {
                    return Err(AppError::ConfigError(
                        "Could not infer directory to config file".to_owned(),
                    ));
                }
            }
        };
        let mut file_path = dir_path.join("config.toml");
        for p in search_list {
            let candidate = dir_path.join(p);
            if candidate.exists() {
                file_path = candidate;
                break;
            }
        }
        log::debug!("Using config file '{}' (inferred)", file_path.display());
        Ok(file_path)
    }
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
        ffi::OsString, fs::Permissions, os::unix::prelude::PermissionsExt,
        path::PathBuf, str::FromStr,
    };

    use color_eyre::Report;

    const TESTROOT: &str = "/tmp/dt-testing";

    pub fn get_testroot(top_level: &str) -> PathBuf {
        PathBuf::from_str(TESTROOT).unwrap().join(top_level)
    }

    pub fn prepare_directory(
        abspath: PathBuf,
        mode: u32,
    ) -> std::result::Result<PathBuf, Report> {
        std::fs::create_dir_all(&abspath)?;
        std::fs::set_permissions(&abspath, Permissions::from_mode(mode))?;
        Ok(abspath)
    }

    pub fn prepare_file(
        abspath: PathBuf,
        mode: u32,
    ) -> std::result::Result<PathBuf, Report> {
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

    pub fn gethostname() -> OsString {
        "r2d2".into()
    }

    pub fn get_current_uid() -> users::uid_t {
        418
    }

    pub fn get_current_username() -> Option<OsString> {
        Some("luke".into())
    }

    pub fn linux_os_release(
    ) -> crate::error::Result<sys_info::LinuxOSReleaseInfo> {
        let info = sys_info::LinuxOSReleaseInfo {
            id: Some("dt".into()),
            id_like: Some("DotfileTemplater".into()),
            name: Some("dt".into()),
            pretty_name: Some("DT".into()),
            version: Some("latest".into()),
            version_id: Some("0.99.99".into()),
            version_codename: Some("dummy-version_codename".into()),
            ansi_color: Some("dummy-ansi_color".into()),
            logo: Some("Buzz Lightyear".into()),
            cpe_name: Some("dummy-cpe_name".into()),
            build_id: Some("#somethingsomething".into()),
            variant: Some("dummy-variant".into()),
            variant_id: Some("dummy-variant_id".into()),
            home_url: Some("https://github.com/blurgyy/dt/".into()),
            documentation_url: Some("https://dt.cli.rs/".into()),
            support_url: Some("https://github.com/blurgyy/dt/issues".into()),
            bug_report_url: Some(
                "https://github.com/blurgyy/dt/issues".into(),
            ),
            privacy_policy_url: Some(
                "https://github.com/blurgyy/dt/blob/main/CODE_OF_CONDUCT.md"
                    .into(),
            ),
        };
        Ok(info)
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Oct 03 2021, 02:54 [CST]
