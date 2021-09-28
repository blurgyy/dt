use std::path::PathBuf;

use color_eyre::{eyre::eyre, Report};
use serde::Deserialize;

#[derive(Default, Clone, Deserialize, Debug)]
pub struct DTConfig {
    pub local: Vec<LocalSyncConfig>,
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
        let raw: DTConfig = DTConfig::from_str(&confstr)?;

        // Expand tilde and globs in "sources" and manifest new config object.
        let globbing_options = glob::MatchOptions {
            case_sensitive: true,
            require_literal_separator: true,
            require_literal_leading_dot: true,
        };
        let mut ret = DTConfig { local: vec![] };
        for original in &raw.local {
            let mut next = LocalSyncConfig {
                sources: vec![],
                target: original.target.to_owned(),
            };
            for s in &original.sources {
                let s = shellexpand::tilde(s.to_str().unwrap());
                let mut s = glob::glob_with(&s, globbing_options)?
                    .map(|x| {
                        x.expect(&format!(
                            "Failed globbing source path {}",
                            &s
                        ))
                    })
                    .collect();
                next.sources.append(&mut s);
            }
            ret.local.push(next);
        }

        Ok(ret)
    }

    /// Loads configuration from string.
    pub fn from_str(s: &str) -> Result<DTConfig, Report> {
        let ret: DTConfig = toml::from_str(s)?;
        ret.validate()?;
        Ok(ret)
    }

    fn validate(self: &DTConfig) -> Result<(), Report> {
        for group in &self.local {
            if group.target.exists() && !group.target.is_dir() {
                return Err(eyre!("Target path exists and not a directory"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod validating {
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

#[cfg(test)]
mod paths_expansion {
    use color_eyre::Report;

    #[test]
    fn tilde() -> Result<(), Report> {
        Ok(())
    }

    #[test]
    fn glob() -> Result<(), Report> {
        Ok(())
    }

    #[test]
    fn tilde_with_glob() -> Result<(), Report> {
        Ok(())
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 21 2021, 01:14 [CST]
