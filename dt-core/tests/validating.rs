use std::{path::PathBuf, str::FromStr};

use color_eyre::{eyre::eyre, Report};

use dt_core::config::DTConfig;

#[test]
fn s_file_t_file() -> Result<(), Report> {
    if let Err(msg) = DTConfig::from_pathbuf(PathBuf::from_str(
        "../testroot/configs/s_file_t_file.toml",
    )?) {
        assert_eq!(
            msg.to_string(),
            "Target path exists and is not a directory",
        );
        Ok(())
    } else {
        Err(eyre!(
                "This config should not be loaded because target is not a directory",
            ))
    }
}

#[test]
fn s_file_t_dir() -> Result<(), Report> {
    if let Ok(_config) = DTConfig::from_pathbuf(PathBuf::from_str(
        "../testroot/configs/s_file_t_dir.toml",
    )?) {
        Ok(())
    } else {
        Err(eyre!(
            "This config should be loaded because target is a directory"
        ))
    }
}

#[test]
fn s_dir_t_dir() -> Result<(), Report> {
    if let Ok(_config) = DTConfig::from_pathbuf(PathBuf::from_str(
        "../testroot/configs/s_dir_t_dir.toml",
    )?) {
        Ok(())
    } else {
        Err(eyre!(
            "This config should be loaded because target is a directory"
        ))
    }
}

#[test]
fn s_dir_t_file() -> Result<(), Report> {
    if let Err(msg) = DTConfig::from_pathbuf(PathBuf::from_str(
        "../testroot/configs/s_dir_t_file.toml",
    )?) {
        assert_eq!(
            msg.to_string(),
            "Target path exists and is not a directory",
        );
        Ok(())
    } else {
        Err(eyre!(
                "This config should not be loaded because target is not a directory",
            ))
    }
}
