use std::path::PathBuf;

use color_eyre::{eyre::eyre, Report};
use serde::Deserialize;

#[derive(Default, Clone, Deserialize, Debug)]
pub struct DTConfig {
    pub local: Option<Vec<LocalSyncConfig>>,
}

/// This struct configures how local items (files/directories) are synced.
///
/// Each path should satisfy one of the following:
///     - is relative to the path from which the executable is being run
///     - is an absolute path
#[derive(Default, Clone, Deserialize, Debug)]
pub struct LocalSyncConfig {
    /// Paths to the items to be synced.
    pub sources: Vec<PathBuf>,

    /// The parent dir of the final synced items.
    ///
    /// For example, if a file `/source/a` is to be synced to `/tar/get/a`, then `target` should be
    /// `/tar/get`; if a directory `source/dir` is to be synced to `targ/et/dir`, then `target` should
    /// be `targ/et`.
    pub target: PathBuf,
    // // The pattern specified in `match_begin` is matched against all
    // match_begin: String,
    // replace_begin: String,
    // match_end: String,
    // replace_end: String,
}

impl DTConfig {
    pub fn from_pathbuf(path: PathBuf) -> Result<DTConfig, Report> {
        let confstr = std::fs::read_to_string(path)?;
        let ret: DTConfig = DTConfig::from_str(&confstr)?;

        Ok(ret)
    }

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
mod tests {
    use color_eyre::{eyre::eyre, Report};

    use crate::config;

    #[test]
    fn target_not_directory() -> Result<(), Report> {
        let confstr = r#"[[local]]
sources = ["/home/gy/repos/FORFUN/dt/config/test.toml"]
target = "/home/gy/repos/FORFUN/dt/config/test.toml"
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
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 21 2021, 01:14 [CST]
