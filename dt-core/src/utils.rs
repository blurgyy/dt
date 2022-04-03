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
        path::PathBuf,
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

    pub fn gethostname() -> OsString {
        match std::env::var("DT_TEST_HOSTNAME_OVERRIDE") {
            Ok(hostname) => hostname.into(),
            _ => gethostname::gethostname(),
        }
    }

    pub fn get_current_uid() -> users::uid_t {
        match std::env::var("DT_TEST_UID_OVERRIDE") {
            Ok(uid) => uid.parse().unwrap(),
            _ => users::get_current_uid(),
        }
    }

    pub fn get_current_username() -> Option<OsString> {
        match std::env::var("DT_TEST_USERNAME_OVERRIDE") {
            Ok(username) => Some(username.into()),
            _ => users::get_current_username(),
        }
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Oct 03 2021, 02:54 [CST]
