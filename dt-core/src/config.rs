use std::path::PathBuf;

use serde::Deserialize;

#[derive(Default, Clone, Deserialize, Debug)]
pub struct Config {
    local: Option<Vec<LocalSyncConfig>>,
}

/// This struct configures how local items (files/directories) are synced.
#[derive(Default, Clone, Deserialize, Debug)]
pub struct LocalSyncConfig {
    /// The items to be syned.
    sources: Vec<PathBuf>,

    /// The parent dir of the final synced items.
    ///
    /// For example, if a file `/source/a` is to be synced to `/tar/get/a`, then `target` should be
    /// `/tar/get`; if a directory `source/dir` is to be synced to `targ/et/dir`, then `target` should
    /// be `targ/et`.
    target: PathBuf,
    // // The pattern specified in `match_begin` is matched against all
    // match_begin: String,
    // replace_begin: String,
    // match_end: String,
    // replace_end: String,
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 21 2021, 01:14 [CST]
