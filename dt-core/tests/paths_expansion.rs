use std::path::PathBuf;
use std::str::FromStr;

use color_eyre::{eyre::eyre, Report};

use dt_core::utils;

use dt_core::config::DTConfig;

#[test]
fn except_dot_asterisk_glob() -> Result<(), Report> {
    if let Err(msg) = DTConfig::from_pathbuf(PathBuf::from_str(
        "../testroot/configs/except_dot_asterisk_glob.toml",
    )?) {
        assert_eq!(
                msg.to_string(),
                "Do not use globbing patterns like '.*', because it also matches curent directory (.) and parent directory (..)",
            );
        Ok(())
    } else {
        Err(eyre!("This config should not be loaded because it contains bad globs (.* and /.*)"))
    }
}

#[test]
fn tilde() -> Result<(), Report> {
    if let Ok(home) = std::env::var("HOME") {
        let config = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/expand_tilde.toml",
        )?)?;
        for group in &config.local {
            assert_eq!(group.basedir.to_str(), Some(home.as_str()));
            assert_eq!(group.target.to_str(), Some(home.as_str()));
        }
        Ok(())
    } else {
        Err(eyre!(
            "Set the `HOME` environment variable to complete this test"
        ))
    }
}

#[test]
fn glob() -> Result<(), Report> {
    let config = DTConfig::from_pathbuf(PathBuf::from_str(
        "../testroot/configs/expand_glob.toml",
    )?)?;
    for group in &config.local {
        assert_eq!(
            vec![
                utils::to_absolute(PathBuf::from_str("../Cargo.lock")?)?,
                utils::to_absolute(PathBuf::from_str("../Cargo.toml")?)?,
            ],
            group.sources
        );
    }
    Ok(())
}

#[test]
fn tilde_with_glob() -> Result<(), Report> {
    if let Ok(home) = std::env::var("HOME") {
        let config = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/expand_tilde_with_glob.toml",
        )?)?;
        let entries = std::fs::read_dir(&home)?
            .map(|x| x.expect("Failed reading dir entry"))
            .map(|x| {
                utils::to_absolute(x.path()).expect(&format!(
                    "Failed converting to absolute path: {}",
                    x.path().display(),
                ))
            })
            .collect::<Vec<_>>();
        for group in &config.local {
            assert_eq!(entries.len(), group.sources.len());
            for s in &group.sources {
                assert!(entries.contains(s));
            }
        }
        Ok(())
    } else {
        Err(eyre!(
            "Set the `HOME` environment variable to complete this test"
        ))
    }
}

#[test]
fn basedir() -> Result<(), Report> {
    if let Some(home) = dirs::home_dir() {
        let config = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/basedir.toml",
        )?)?;
        for group in &config.local {
            assert_eq!(group.basedir, home);
        }

        Ok(())
    } else {
        Err(eyre!("Cannot determine home dir for unit testing"))
    }
}

#[test]
fn staging() -> Result<(), Report> {
    if let Ok(home) = std::env::var("HOME") {
        let config = DTConfig::from_pathbuf(PathBuf::from_str(
            "../testroot/configs/staging.toml",
        )?)?;
        assert_eq!(
            config.global.unwrap().staging.unwrap().to_str().unwrap(),
            PathBuf::from_str(&home)?
                .join(".cache")
                .join("dt")
                .join("staging")
                .to_str()
                .unwrap(),
        );
        Ok(())
    } else {
        Err(eyre!(
            "Set the `HOME` environment variable to complete this test"
        ))
    }
}

#[test]
fn sorting_and_deduping() -> Result<(), Report> {
    let config = DTConfig::from_pathbuf(PathBuf::from_str(
        "../testroot/configs/sorting_and_deduping.toml",
    )?)?;
    for group in config.local {
        assert_eq!(group.sources.len(), 6);
        assert_eq!(
            group.sources,
            vec![
                utils::to_absolute(PathBuf::from_str(
                    "../testroot/items/sorting_and_deduping/A-a"
                )?)?,
                utils::to_absolute(PathBuf::from_str(
                    "../testroot/items/sorting_and_deduping/A-b"
                )?)?,
                utils::to_absolute(PathBuf::from_str(
                    "../testroot/items/sorting_and_deduping/A-c"
                )?)?,
                utils::to_absolute(PathBuf::from_str(
                    "../testroot/items/sorting_and_deduping/B-a"
                )?)?,
                utils::to_absolute(PathBuf::from_str(
                    "../testroot/items/sorting_and_deduping/B-b"
                )?)?,
                utils::to_absolute(PathBuf::from_str(
                    "../testroot/items/sorting_and_deduping/B-c"
                )?)?,
            ]
        );
    }
    Ok(())
}
