use color_eyre::Report;
use std::path::PathBuf;
use std::str::FromStr;

use dt_core::{config::DTConfig, utils};

#[test]
fn empty_ignored_array() -> Result<(), Report> {
    let config = DTConfig::from_pathbuf(PathBuf::from_str(
        "../testroot/configs/empty_ignored_array.toml",
    )?)?;
    for group in &config.local {
        let expected_sources = vec![utils::to_absolute(PathBuf::from_str(
            "../testroot/README.md",
        )?)?];
        assert_eq!(group.sources, expected_sources);
        assert_eq!(
            group.target,
            utils::to_absolute(PathBuf::from_str(".")?)?,
        );
        assert_eq!(group.ignored, Some(Vec::<String>::new()));
    }

    Ok(())
}

#[test]
fn empty_source_array() -> Result<(), Report> {
    let config = DTConfig::from_pathbuf(PathBuf::from_str(
        "../testroot/configs/empty_source_array.toml",
    )?)?;
    for group in &config.local {
        let expected_sources: Vec<PathBuf> = vec![];
        assert_eq!(group.sources, expected_sources);
        assert_eq!(
            group.target,
            utils::to_absolute(PathBuf::from_str(".")?)?,
        );
        assert_eq!(group.ignored, Some(vec!["README.md".to_owned()]));
    }

    Ok(())
}

#[test]
fn partial_filename() -> Result<(), Report> {
    let config = DTConfig::from_pathbuf(PathBuf::from_str(
        "../testroot/configs/partial_filename.toml",
    )?)?;
    for group in &config.local {
        let expected_sources = vec![
            utils::to_absolute(PathBuf::from_str("../Cargo.lock")?)?,
            utils::to_absolute(PathBuf::from_str("../Cargo.toml")?)?,
        ];
        assert_eq!(group.sources, expected_sources);
        assert_eq!(
            group.target,
            utils::to_absolute(PathBuf::from_str(".")?)?
        );
        assert_eq!(group.ignored, Some(vec![".lock".to_owned()]));
    }

    Ok(())
}

#[test]
fn regular_ignore() -> Result<(), Report> {
    let config = DTConfig::from_pathbuf(PathBuf::from_str(
        "../testroot/configs/regular_ignore.toml",
    )?)?;
    for group in &config.local {
        let expected_sources =
            vec![utils::to_absolute(PathBuf::from_str("../Cargo.lock")?)?];
        assert_eq!(group.sources, expected_sources);
        assert_eq!(
            group.target,
            utils::to_absolute(PathBuf::from_str(".")?)?,
        );
        assert_eq!(group.ignored, Some(vec!["Cargo.toml".to_owned()]));
    }

    Ok(())
}

#[test]
fn no_per_host_items() -> Result<(), Report> {
    let config = DTConfig::from_pathbuf(PathBuf::from_str(
        "../testroot/configs/no_per_host_items.toml",
    )?)?;
    for group in config.local {
        assert_eq!(
            group.sources,
            vec![
                utils::to_absolute(PathBuf::from_str(
                    "../testroot/items/no_per_host_items/authorized_keys"
                )?)?,
                utils::to_absolute(PathBuf::from_str(
                    "../testroot/items/no_per_host_items/config"
                )?)?,
            ]
        )
    }

    Ok(())
}
