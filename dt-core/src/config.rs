use std::path::PathBuf;

use color_eyre::{eyre::eyre, Report};
use serde::Deserialize;

#[derive(Default, Clone, Deserialize, Debug)]
pub struct DTConfig {
    pub local: Option<Vec<LocalSyncConfig>>,
}

/// Configures how local items (files/directories) are synced.
///
/// Each item should satisfy one of the following:
///     - is relative to the path from which the executable is being run
///     - is an absolute path
#[derive(Default, Clone, Deserialize, Debug)]
pub struct LocalSyncConfig {
    /// Paths to the items to be synced.
    pub sources: Vec<PathBuf>,

    /// The parent dir of the final synced items.
    ///
    /// For example, if a file `/source/file` is to be synced to `/tar/get/file`, then `target`
    /// should be `/tar/get`; if a directory `source/dir` is to be synced to `targ/et/dir`, then
    /// `target` should be `targ/et`.
    pub target: PathBuf,
    // // The pattern specified in `match_begin` is matched against all
    // match_begin: String,
    // replace_begin: String,
    // match_end: String,
    // replace_end: String,
}

impl DTConfig {
    /// Loads configuration from a file.
    pub fn from_pathbuf(path: PathBuf) -> Result<DTConfig, Report> {
        let confstr = std::fs::read_to_string(path)?;
        let ret: DTConfig = DTConfig::from_str(&confstr)?;

        Ok(ret)
    }

    /// Loads configuration from string.
    pub fn from_str(s: &str) -> Result<DTConfig, Report> {
        let ret: DTConfig = toml::from_str(s)?;
        ret.validate()?;
        Ok(ret)
    }

    fn validate(self: &DTConfig) -> Result<(), Report> {
        if let Some(local) = &self.local {
            for group in local {
                if group.target.exists() && !group.target.is_dir() {
                    return Err(eyre!(
                        "Target path exists and not a directory"
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod config_validating {
    use std::{path::PathBuf, str::FromStr};

    use color_eyre::{eyre::eyre, Report};

    use crate::config;

    #[test]
    fn s_file_t_file_from_str() -> Result<(), Report> {
        // Paths are relative to directory `dt-core`.
        let confstr = r#"[[local]]
sources = ["../testroot/README.md"]
target = "../testroot/README.md"
        "#;
        if let Ok(config) = config::DTConfig::from_str(&confstr) {
            Err(eyre!(
                "This config should not be loaded because target is not a directory: {:#?}",
                config
            ))
        } else {
            Ok(())
        }
    }

    #[test]
    fn s_file_t_file() -> Result<(), Report> {
        if let Ok(config) = config::DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/s_file_t_file.toml",
        )?) {
            Err(eyre!(
                "This config should not be loaded because target is not a directory: {:#?}",
                config
            ))
        } else {
            Ok(())
        }
    }

    #[test]
    fn s_file_t_dir() -> Result<(), Report> {
        if let Ok(_config) = config::DTConfig::from_pathbuf(
            PathBuf::from_str("../testroot/configs/s_file_t_dir.toml")?,
        ) {
            Ok(())
        } else {
            Err(eyre!(
                "This config should be loaded because target is a directory"
            ))
        }
    }

    #[test]
    fn s_dir_t_dir() -> Result<(), Report> {
        if let Ok(_config) = config::DTConfig::from_pathbuf(
            PathBuf::from_str("../testroot/configs/s_dir_t_dir.toml")?,
        ) {
            Ok(())
        } else {
            Err(eyre!(
                "This config should be loaded because target is a directory"
            ))
        }
    }

    #[test]
    fn s_dir_t_file() -> Result<(), Report> {
        if let Ok(config) = config::DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/s_dir_t_file.toml",
        )?) {
            Err(eyre!(
                "This config should not be loaded because target is not a directory: {:#?}",
                config
            ))
        } else {
            Ok(())
        }
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 21 2021, 01:14 [CST]
