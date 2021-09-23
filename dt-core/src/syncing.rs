use std::path::PathBuf;

use color_eyre::{eyre::eyre, Report};

use crate::config::DTConfig;

pub fn sync(config: &DTConfig) -> Result<(), Report> {
    if let Some(locals) = &config.local {
        for local in locals {
            for spath in &local.sources {
                _sync_recursive(spath, &local.target)?;
            }
        }
    }
    Ok(())
}

/// Recursively sync `spath` to a directory `tparent`.
///
/// Args:
///   - `spath`: Path to source item.
///   - `tparent`: Path to the parent dir of the disired sync destination.
fn _sync_recursive(spath: &PathBuf, tparent: &PathBuf) -> Result<(), Report> {
    let sname = spath.file_name().unwrap();
    let tpath = tparent.join(sname);
    if spath.is_file() {
        if tpath.is_dir() {
            return Err(
                eyre!(
                    "A directory ({:?}) exists at the target path of a source file ({:?})",
                    tpath,
                    spath,
                )
            );
        }
        std::fs::copy(spath, tpath)?;
    } else if spath.is_dir() {
        if tpath.is_file() {
            return Err(
                eyre!(
                    "A file ({:?}) exists at the target path of a source directory ({:?})",
                    tpath,
                    spath,
                )
            );
        }
        std::fs::create_dir_all(&tpath)?;
        for item in std::fs::read_dir(spath)? {
            let item = item?;
            _sync_recursive(&item.path(), &tpath)?;
        }
    }
    Ok(())
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 23 2021, 00:05 [CST]
