//! Reverse collection functionality for DT.
//!
//! This module provides the ability to detect changes in target directories
//! and sync them back to the source repository.

use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{Read},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    config::{DTConfig, LocalGroup},
    error::{Error as AppError, Result},
    item::Operate,
};

/// State file format for tracking file checksums
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CollectionState {
    /// Version of the state file format
    pub version: u32,
    /// Last time collection was performed
    pub last_collection: Option<String>,
    /// Map of relative path to file state
    pub files: HashMap<String, FileState>,
}

/// State for an individual file
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FileState {
    /// SHA-256 checksum of the file content
    pub checksum: String,
    /// Last time this file was seen
    pub last_seen: String,
}

/// Types of changes that can be detected
#[derive(Clone, Debug, PartialEq)]
pub enum ChangeType {
    /// File exists in target but not in state (new file)
    New,
    /// File checksum differs from state (modified file)
    Modified,
    /// File in state but not on disk (deleted file)
    Deleted,
}

/// A detected change with potential conflict info
#[derive(Clone, Debug)]
pub struct Change {
    /// Type of change
    pub change_type: ChangeType,
    /// Relative path from the group's base
    pub relative_path: PathBuf,
    /// Absolute path to the target file
    pub target_path: PathBuf,
    /// Absolute path to the source file
    pub source_path: PathBuf,
    /// Group name this change belongs to
    pub group_name: String,
}

/// Information about a source file conflict
#[derive(Clone, Debug)]
pub struct Conflict {
    /// The change that would be applied
    pub change: Change,
    /// Path to the git repository root
    pub repo_path: PathBuf,
    /// Git status output (if available)
    pub git_status: Option<String>,
}

impl std::fmt::Display for Conflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "  [CONFLICT] {}\n    Source: {} (uncommitted changes)\n    Target: {} ({:?})",
            self.change.relative_path.display(),
            self.change.source_path.display(),
            self.change.target_path.display(),
            self.change.change_type
        )
    }
}

/// Collection result with detailed status
#[derive(Clone, Debug)]
pub struct CollectionResult {
    /// Changes that were successfully applied
    pub changes: Vec<Change>,
    /// Conflicts that prevented collection
    pub conflicts: Vec<Conflict>,
    /// Number of files skipped due to --skip-dirty
    pub skipped: usize,
}

impl CollectionResult {
    /// Returns true if there were no conflicts
    pub fn is_success(&self) -> bool {
        self.conflicts.is_empty()
    }
    
    /// Returns true if any changes were applied
    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }
}

/// Validate that collect_sources patterns are a subset of sources patterns.
/// Returns Ok(()) if valid, Err if collect_sources would match files outside sources.
pub fn validate_collect_sources(group: &LocalGroup) -> Result<()> {
    let sources_patterns: Vec<&str> = group.sources.iter()
        .map(|p| p.to_str().unwrap_or("*"))
        .collect();
    let collect_patterns = group.get_collect_sources();
    
    // For each collect pattern, check if it's covered by any source pattern
    for collect_pat in &collect_patterns {
        // Simple subset check: if collect pattern is more specific than a source pattern, it's ok
        // e.g., sources=["*.md"], collect=["20*.md"] -> ok
        // e.g., sources=["config-*.toml"], collect=["*.toml"] -> not ok (wider)
        
        // Check if collect pattern is potentially wider than any source pattern
        let is_covered = sources_patterns.iter().any(|src_pat| {
            pattern_covers(src_pat, collect_pat)
        });
        
        if !is_covered {
            // Check if collect pattern could match files that sources wouldn't match
            // This is a heuristic - we check if collect is a "superset" pattern
            if could_be_wider_pattern(collect_pat, &sources_patterns) {
                return Err(AppError::ConfigError(format!(
                    "collect_sources pattern '{}' in group '{}' may match files outside of sources. \
                    collect_sources must be a subset of sources.",
                    collect_pat, group.name
                )));
            }
        }
    }
    
    Ok(())
}

/// Check if source pattern covers collect pattern.
/// e.g., "*.md" covers "20*.md", "*" covers everything
fn pattern_covers(source: &str, collect: &str) -> bool {
    // Exact match
    if source == collect {
        return true;
    }
    
    // If source is "*", it covers everything
    if source == "*" {
        return true;
    }
    
    // If source ends with wildcard and collect starts with the same prefix
    if source.ends_with('*') {
        let src_prefix = &source[..source.len()-1];
        if collect.starts_with(src_prefix) || src_prefix.is_empty() {
            // "*.md" covers "20*.md" because both end with .md
            // Check if collect also ends with the same suffix pattern
            if source.contains('.') && collect.contains('.') {
                let src_ext = source.rsplit('.').next().unwrap_or("");
                let collect_ext = collect.rsplit('.').next().unwrap_or("");
                if src_ext == "*" || src_ext == collect_ext {
                    return true;
                }
            }
        }
    }
    
    false
}

/// Heuristic to check if collect pattern could be wider than source patterns.
fn could_be_wider_pattern(collect: &str, sources: &[&str]) -> bool {
    // If collect is "*" and no source is "*", it's wider
    if collect == "*" && !sources.contains(&"*") {
        return true;
    }
    
    // If collect has fewer restrictions (e.g., "*.toml" vs "config-*.toml")
    if collect.starts_with("*.") {
        let collect_ext = &collect[2..];
        // Check if all sources are more specific
        let all_more_specific = sources.iter().all(|s| {
            !s.ends_with(&format!("*.{}", collect_ext)) || s.len() > collect.len()
        });
        if all_more_specific && !sources.is_empty() {
            return true;
        }
    }
    
    false
}

/// Expand glob patterns in a directory to a set of relative paths.
fn expand_globs_in_dir(base_dir: &Path, patterns: &[String]) -> Result<HashSet<PathBuf>> {
    let mut files = HashSet::new();
    
    if !base_dir.exists() {
        return Ok(files);
    }
    
    for pattern in patterns {
        let full_pattern = base_dir.join(pattern);
        let pattern_str = full_pattern.to_string_lossy();
        
        match glob::glob(&pattern_str) {
            Ok(paths) => {
                for entry in paths.flatten() {
                    if entry.is_file() {
                        // Get relative path from base_dir
                        if let Ok(rel_path) = entry.strip_prefix(base_dir) {
                            files.insert(rel_path.to_path_buf());
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("Invalid glob pattern '{}': {}", pattern_str, e);
            }
        }
    }
    
    Ok(files)
}

/// Compute source path from target path by reversing the target computation.
/// This is an approximation - it reverses the hostname stripping but not renaming rules.
fn compute_source_path(
    target_path: &Path,
    target_base: &Path,
    source_base: &Path,
    _hostname_sep: &str,
) -> Result<PathBuf> {
    // Get relative path from target base
    let rel_path = target_path.strip_prefix(target_base)
        .map_err(|_| AppError::PathError(format!(
            "Target path '{}' is not under target base '{}'",
            target_path.display(), target_base.display()
        )))?;
    
    // Compute source path - note: this doesn't reverse renaming rules
    // For now, we assume 1:1 mapping for new file discovery
    Ok(source_base.join(rel_path))
}
/// Returns Ok(true) if clean (no uncommitted changes)
/// Returns Ok(false) if dirty
/// Returns Err if git check fails
fn check_source_clean(source_path: &Path) -> Result<(bool, Option<PathBuf>, Option<String>)> {
    // Find git repository root
    let mut current = source_path.parent();
    let repo_root = loop {
        match current {
            Some(dir) => {
                let git_dir = dir.join(".git");
                if git_dir.exists() {
                    break Some(dir.to_path_buf());
                }
                current = dir.parent();
            }
            None => break None,
        }
    };
    
    let repo_root = match repo_root {
        Some(r) => r,
        None => {
            // Not in a git repo, treat as clean
            return Ok((true, None, None));
        }
    };
    
    // Run git diff --quiet to check for uncommitted changes
    let output = std::process::Command::new("git")
        .args(&["diff", "--quiet", source_path.to_string_lossy().as_ref()])
        .current_dir(&repo_root)
        .output()
        .map_err(|e| AppError::ProcessError(format!("Failed to run git diff: {}", e)))?;
    
    if output.status.code() == Some(1) {
        // Exit code 1 means there are uncommitted changes
        // Get git status for better error message
        let status_output = std::process::Command::new("git")
            .args(&["status", "--short", source_path.to_string_lossy().as_ref()])
            .current_dir(&repo_root)
            .output();
        
        let git_status = status_output.ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .filter(|s| !s.is_empty());
        
        return Ok((false, Some(repo_root), git_status));
    }
    
    // Exit code 0 means no uncommitted changes (clean)
    Ok((true, Some(repo_root), None))
}

/// Calculate SHA-256 checksum of a file
fn calculate_checksum(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Load collection state from file
pub fn load_state(state_path: &Path) -> Result<CollectionState> {
    if !state_path.exists() {
        return Ok(CollectionState {
            version: 1,
            last_collection: None,
            files: HashMap::new(),
        });
    }

    let content = fs::read_to_string(state_path)?;
    let state: CollectionState = serde_json::from_str(&content)
        .map_err(|e| AppError::ParseError(format!("Failed to parse state file: {}", e)))?;
    
    Ok(state)
}

/// Save collection state to file
pub fn save_state(state_path: &Path, state: &CollectionState) -> Result<()> {
    let content = serde_json::to_string_pretty(state)
        .map_err(|e| AppError::ParseError(format!("Failed to serialize state: {}", e)))?;
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = state_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    fs::write(state_path, content)?;
    Ok(())
}

/// Detect changes for a single group
pub fn detect_group_changes(
    group: &LocalGroup,
    state: &mut CollectionState,
) -> Result<Vec<Change>> {
    let mut changes = Vec::new();
    let group_name = group.name.to_string();
    
    // Get the current timestamp
    let now = chrono::Utc::now().to_rfc3339();
    
    let base = &group.base;
    let target = &group.target;
    let hostname_sep = &group.get_hostname_sep();
    let renaming_rules = group.get_renaming_rules();
    
    // Iterate over expanded sources (these are absolute paths)
    for source_path in &group.sources {
        // Skip if source is not a file
        if !source_path.is_file() {
            continue;
        }
        
        // Compute relative path from base
        let relative_path = match source_path.strip_prefix(base) {
            Ok(rp) => rp.to_path_buf(),
            Err(_) => {
                log::warn!("Source '{}' is not under base '{}'", source_path.display(), base.display());
                continue;
            }
        };
        
        // Check if file is excluded from collection
        let filename = relative_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if group.is_excluded(filename) {
            log::debug!("Skipping excluded file: {}", relative_path.display());
            continue;
        }
        
        // Compute target path using make_target (handles hostname suffixes and renaming)
        let target_path = source_path.clone().make_target(
            hostname_sep,
            base,
            target,
            renaming_rules.clone(),
        )?;
        
        let path_key = relative_path.to_string_lossy().to_string();
        
        // Check if target file exists
        if !target_path.exists() {
            // File might have been deleted
            if state.files.contains_key(&path_key) {
                changes.push(Change {
                    change_type: ChangeType::Deleted,
                    relative_path: relative_path.clone(),
                    target_path: target_path.clone(),
                    source_path: source_path.clone(),
                    group_name: group_name.clone(),
                });
                state.files.remove(&path_key);
            }
            continue;
        }
        
        // Calculate current checksum of target file
        let current_checksum = calculate_checksum(&target_path)?;
        
        match state.files.get(&path_key) {
            None => {
                // New file detected in target
                log::info!("New file detected: {}", relative_path.display());
                changes.push(Change {
                    change_type: ChangeType::New,
                    relative_path: relative_path.clone(),
                    target_path: target_path.clone(),
                    source_path: source_path.clone(),
                    group_name: group_name.clone(),
                });
                state.files.insert(
                    path_key,
                    FileState {
                        checksum: current_checksum,
                        last_seen: now.clone(),
                    },
                );
            }
            Some(file_state) if file_state.checksum != current_checksum => {
                // Modified file
                log::info!("Modified file detected: {}", relative_path.display());
                changes.push(Change {
                    change_type: ChangeType::Modified,
                    relative_path: relative_path.clone(),
                    target_path: target_path.clone(),
                    source_path: source_path.clone(),
                    group_name: group_name.clone(),
                });
                state.files.insert(
                    path_key,
                    FileState {
                        checksum: current_checksum,
                        last_seen: now.clone(),
                    },
                );
            }
            _ => {
                // No change, just update last_seen
                state.files.insert(
                    path_key,
                    FileState {
                        checksum: current_checksum,
                        last_seen: now.clone(),
                    },
                );
            }
        }
    }
    
    // Phase 2: Discover new files in target that don't exist in source (orphan files)
    // This uses collect_sources patterns to scan target directory
    let collect_patterns = group.get_collect_sources();
    let target_files = expand_globs_in_dir(target, &collect_patterns)?;
    
    // Build a set of relative paths that already have source files
    let existing_source_rels: HashSet<PathBuf> = changes.iter()
        .map(|c| c.relative_path.clone())
        .collect();
    
    // Find orphan files: exist in target but not tracked by any source
    for rel_path in target_files {
        // Skip if already tracked
        if existing_source_rels.contains(&rel_path) {
            continue;
        }
        
        let target_path = target.join(&rel_path);
        
        // Check if file is excluded from collection
        let filename = rel_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if group.is_excluded(filename) {
            log::debug!("Skipping excluded orphan file: {}", rel_path.display());
            continue;
        }
        
        // Compute corresponding source path
        let source_path = compute_source_path(&target_path, target, base, hostname_sep)?;
        
        // If source doesn't exist, this is a new orphan file
        if !source_path.exists() {
            let path_key = rel_path.to_string_lossy().to_string();
            let current_checksum = calculate_checksum(&target_path)?;
            
            log::info!("New orphan file detected: {}", rel_path.display());
            changes.push(Change {
                change_type: ChangeType::New,
                relative_path: rel_path.clone(),
                target_path: target_path.clone(),
                source_path: source_path.clone(),
                group_name: group_name.clone(),
            });
            state.files.insert(
                path_key,
                FileState {
                    checksum: current_checksum,
                    last_seen: now.clone(),
                },
            );
        }
    }
    
    // Update last_collection timestamp
    state.last_collection = Some(now.clone());
    
    Ok(changes)
}

/// Collect changes from all enabled groups with pre-check for conflicts
pub fn collect(
    config: &DTConfig,
    state_path: &Path,
    dry_run: bool,
    skip_dirty: bool,
) -> Result<CollectionResult> {
    // Pre-run validation: check collect_sources patterns
    for group in &config.local {
        if group.is_collect_enabled() {
            if let Err(e) = validate_collect_sources(group) {
                return Err(AppError::ConfigError(format!(
                    "Pre-run check failed for group '{}': {}",
                    group.name, e
                )));
            }
        }
    }
    
    let mut state = load_state(state_path)?;
    let mut all_changes = Vec::new();
    let mut conflicts = Vec::new();
    let mut skipped = 0;

    log::debug!("Total groups in config: {}", config.local.len());

    // Phase 1: Detect all changes and check for conflicts
    for group in &config.local {
        log::debug!("Processing group: '{}' (collect={:?})", group.name, group.collect);

        // Skip groups without collect enabled
        if !group.is_collect_enabled() {
            log::debug!("Skipping group '{}' (collect not enabled)", group.name);
            continue;
        }

        log::info!("Checking group '{}' for changes...", group.name);
        let changes = detect_group_changes(group, &mut state)?;

        // Check each change for source conflicts
        for change in changes {
            if !change.source_path.exists() {
                // Source doesn't exist (e.g., new file in target), no conflict
                all_changes.push(change);
                continue;
            }

            match check_source_clean(&change.source_path)? {
                (true, _, _) => {
                    // Source is clean, can collect
                    all_changes.push(change);
                }
                (false, repo_path, git_status) => {
                    // Source has uncommitted changes
                    if skip_dirty {
                        log::warn!(
                            "Skipping dirty file ({:?}): {}",
                            change.change_type,
                            change.relative_path.display()
                        );
                        skipped += 1;
                    } else {
                        conflicts.push(Conflict {
                            change: change.clone(),
                            repo_path: repo_path.unwrap_or_else(|| change.source_path.clone()),
                            git_status,
                        });
                    }
                }
            }
        }
    }

    // If there are conflicts and we're not skipping, return early with conflicts
    if !conflicts.is_empty() && !skip_dirty {
        return Ok(CollectionResult {
            changes: Vec::new(),
            conflicts,
            skipped,
        });
    }

    // Phase 2: Apply changes (if not dry-run)
    if !dry_run {
        for change in &all_changes {
            match change.change_type {
                ChangeType::Deleted => {
                    log::info!("Deleting source file: {}", change.source_path.display());
                    if change.source_path.exists() {
                        fs::remove_file(&change.source_path)?;
                    }
                }
                _ => {
                    log::info!(
                        "Collecting: {} -> {}",
                        change.target_path.display(),
                        change.source_path.display()
                    );
                    // Copy target to source
                    if let Some(parent) = change.source_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(&change.target_path, &change.source_path)?;
                }
            }
        }
        save_state(state_path, &state)?;
    }

    Ok(CollectionResult {
        changes: all_changes,
        conflicts,
        skipped,
    })
}

/// Legacy collect function for backward compatibility (no skip_dirty, no conflict checking)
pub fn collect_legacy(
    config: &DTConfig,
    state_path: &Path,
    dry_run: bool,
) -> Result<Vec<Change>> {
    let result = collect(config, state_path, dry_run, false)?;
    Ok(result.changes)
}

/// Get the default state file path
pub fn default_state_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("dt")
        .join(".dt-state.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_calculate_checksum() {
        // Create a temporary file
        let temp_dir = std::env::temp_dir().join("dt_test_checksum");
        fs::create_dir_all(&temp_dir).unwrap();
        let test_file = temp_dir.join("test.txt");
        
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"hello world").unwrap();
        drop(file);
        
        // Calculate checksum
        let checksum = calculate_checksum(&test_file).unwrap();
        
        // Verify it's a valid hex string (64 chars for SHA-256)
        assert_eq!(checksum.len(), 64);
        assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()));
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_load_save_state() {
        let temp_dir = std::env::temp_dir().join("dt_test_state");
        fs::create_dir_all(&temp_dir).unwrap();
        let state_file = temp_dir.join("test-state.json");
        
        // Create a state
        let mut state = CollectionState {
            version: 1,
            last_collection: Some("2026-02-10T14:30:00Z".to_string()),
            files: HashMap::new(),
        };
        state.files.insert(
            "config/test.txt".to_string(),
            FileState {
                checksum: "a1b2c3d4".to_string(),
                last_seen: "2026-02-10T14:30:00Z".to_string(),
            },
        );
        
        // Save state
        save_state(&state_file, &state).unwrap();
        
        // Load state
        let loaded = load_state(&state_file).unwrap();
        
        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.last_collection, Some("2026-02-10T14:30:00Z".to_string()));
        assert_eq!(loaded.files.len(), 1);
        assert_eq!(
            loaded.files.get("config/test.txt").unwrap().checksum,
            "a1b2c3d4"
        );
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
