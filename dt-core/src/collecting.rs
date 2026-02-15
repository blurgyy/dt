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

/// Information about a source-target mapping conflict
#[derive(Clone, Debug)]
pub struct SourceTargetConflict {
    /// The source file path
    pub source_path: PathBuf,
    /// List of (group_name, target_path) that map to this source
    pub mappings: Vec<(String, PathBuf)>,
}

impl std::fmt::Display for SourceTargetConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  Source: {}", self.source_path.display())?;
        for (group, target) in &self.mappings {
            writeln!(f, "    -> {} maps to {}", group, target.display())?;
        }
        Ok(())
    }
}

/// Validate that no source file is mapped to multiple different target paths
/// across groups with collect enabled.
/// Returns Ok(()) if no conflicts, Err with detailed conflict info otherwise.
pub fn validate_no_source_target_conflicts(config: &DTConfig) -> Result<()> {
    // Map: source_path -> Vec<(group_name, target_path)>
    let mut source_to_targets: HashMap<PathBuf, Vec<(String, PathBuf)>> = HashMap::new();

    for group in &config.local {
        if !group.is_collect_enabled() {
            continue;
        }

        let base = &group.base;
        let target = &group.target;
        let hostname_sep = &group.get_hostname_sep();
        let renaming_rules = group.get_renaming_rules();

        for source_path in &group.sources {
            if !source_path.is_file() {
                continue;
            }

            // Compute target path for this source
            let computed_target = source_path.clone().make_target(
                hostname_sep,
                base,
                target,
                renaming_rules.clone(),
            )?;

            source_to_targets
                .entry(source_path.clone())
                .or_default()
                .push((group.name.to_string(), computed_target));
        }
    }

    // Find conflicts: same source mapped to multiple different targets
    let conflicts: Vec<SourceTargetConflict> = source_to_targets
        .into_iter()
        .filter_map(|(source_path, mappings)| {
            if mappings.len() < 2 {
                return None;
            }

            // Check if all targets are the same (could be same file via different paths like symlinks)
            let first_target = &mappings[0].1;
            let all_same = mappings.iter().all(|(_, t)| t == first_target);

            if all_same {
                return None;
            }

            Some(SourceTargetConflict {
                source_path,
                mappings,
            })
        })
        .collect();

    if !conflicts.is_empty() {
        let mut msg = String::from(
            "Source-Target conflict detected: the following source files are mapped to multiple different targets:\n"
        );
        for conflict in &conflicts {
            msg.push_str(&format!("\n{}", conflict));
        }
        msg.push_str("\nPlease resolve this conflict by ensuring each source maps to at most one target, or disable collect for conflicting groups.");
        return Err(AppError::ConfigError(msg));
    }

    Ok(())
}

/// Validate that collect_sources patterns are a subset of sources patterns.
/// Returns Ok(()) if valid, Err if collect_sources would match files outside sources.
pub fn validate_collect_sources(group: &LocalGroup) -> Result<()> {
    let sources_patterns: Vec<&str> = group.sources.iter()
        .map(|p| p.to_str().unwrap_or("*"))
        .collect();
    
    // Skip validation if sources contain absolute paths (i.e., were expanded)
    // In this case, collect_sources was explicitly set to the original glob patterns
    // by expand_for_collect, so the subset check doesn't apply
    if sources_patterns.iter().any(|p| p.starts_with('/')) {
        return Ok(());
    }
    
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
    
    // If source is "*.ext", check if collect also ends with ".ext"
    if source.starts_with("*.") {
        let src_ext = &source[2..]; // Get extension after "*."
        if collect.ends_with(&format!(".{}", src_ext)) || collect.ends_with(src_ext) {
            // Additional check: collect should be more specific (have some prefix)
            // e.g., "2024*.md" starts with "2024"
            if collect.contains('*') {
                let collect_prefix = collect.split('*').next().unwrap_or("");
                // If collect has a non-empty prefix before *, it's more specific
                if !collect_prefix.is_empty() {
                    return true;
                }
            }
        }
    }
    
    // If source ends with wildcard and collect starts with the same prefix
    if source.ends_with('*') {
        let src_prefix = &source[..source.len()-1];
        if collect.starts_with(src_prefix) || src_prefix.is_empty() {
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
    // Pre-run validation: check for source-target conflicts across groups
    if let Err(e) = validate_no_source_target_conflicts(config) {
        return Err(e);
    }
    
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
    use crate::syncing::expand_for_collect;
    use std::io::Write;
    use std::str::FromStr;

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

    // =================================================================
    // Tests for validate_no_source_target_conflicts
    // =================================================================

    #[test]
    fn test_validate_no_source_target_conflicts_no_conflict_single_group() {
        // Single group, no possible conflict
        let temp_dir = std::env::temp_dir().join("dt_test_conflict_1");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let base = temp_dir.join("base");
        let target = temp_dir.join("target");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(&target).unwrap();
        
        let source_file = base.join("file.txt");
        fs::write(&source_file, "content").unwrap();
        
        let config = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "group1"
base = "{}"
sources = ["file.txt"]
target = "{}"
collect = true
"#,
            base.display(),
            target.display(),
        )).unwrap();
        
        let expanded = expand_for_collect(config).unwrap();
        assert!(validate_no_source_target_conflicts(&expanded).is_ok());
        
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_validate_no_source_target_conflicts_conflict_two_groups() {
        // Two groups with same source but different targets → conflict
        let temp_dir = std::env::temp_dir().join("dt_test_conflict_2");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let base = temp_dir.join("base");
        let target1 = temp_dir.join("target1");
        let target2 = temp_dir.join("target2");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(&target1).unwrap();
        fs::create_dir_all(&target2).unwrap();
        
        let source_file = base.join("file.txt");
        fs::write(&source_file, "content").unwrap();
        
        let config = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "group1"
base = "{}"
sources = ["file.txt"]
target = "{}"
collect = true

[[local]]
name = "group2"
base = "{}"
sources = ["file.txt"]
target = "{}"
collect = true
"#,
            base.display(),
            target1.display(),
            base.display(),
            target2.display(),
        )).unwrap();
        
        let expanded = expand_for_collect(config).unwrap();
        let result = validate_no_source_target_conflicts(&expanded);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Source-Target conflict detected"));
        assert!(err_msg.contains("group1"));
        assert!(err_msg.contains("group2"));
        
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_validate_no_source_target_conflicts_same_target_no_conflict() {
        // Two groups with same source and same target → NOT a conflict
        let temp_dir = std::env::temp_dir().join("dt_test_conflict_3");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let base = temp_dir.join("base");
        let target = temp_dir.join("target");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(&target).unwrap();
        
        let source_file = base.join("file.txt");
        fs::write(&source_file, "content").unwrap();
        
        let config = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "group1"
base = "{}"
sources = ["file.txt"]
target = "{}"
collect = true

[[local]]
name = "group2"
base = "{}"
sources = ["file.txt"]
target = "{}"
collect = true
"#,
            base.display(),
            target.display(),
            base.display(),
            target.display(),
        )).unwrap();
        
        let expanded = expand_for_collect(config).unwrap();
        // Same target path should not be a conflict
        assert!(validate_no_source_target_conflicts(&expanded).is_ok());
        
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_validate_no_source_target_conflicts_one_disabled() {
        // Two groups, but only one has collect enabled → NOT a conflict
        let temp_dir = std::env::temp_dir().join("dt_test_conflict_4");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let base = temp_dir.join("base");
        let target1 = temp_dir.join("target1");
        let target2 = temp_dir.join("target2");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(&target1).unwrap();
        fs::create_dir_all(&target2).unwrap();
        
        let source_file = base.join("file.txt");
        fs::write(&source_file, "content").unwrap();
        
        let config = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "group1"
base = "{}"
sources = ["file.txt"]
target = "{}"
collect = true

[[local]]
name = "group2"
base = "{}"
sources = ["file.txt"]
target = "{}"
collect = false
"#,
            base.display(),
            target1.display(),
            base.display(),
            target2.display(),
        )).unwrap();
        
        let expanded = expand_for_collect(config).unwrap();
        // Only group1 has collect enabled, no conflict
        assert!(validate_no_source_target_conflicts(&expanded).is_ok());
        
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_validate_no_source_target_conflicts_three_groups() {
        // Three groups with same source but different targets → conflict listing all three
        let temp_dir = std::env::temp_dir().join("dt_test_conflict_5");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let base = temp_dir.join("base");
        let target1 = temp_dir.join("target1");
        let target2 = temp_dir.join("target2");
        let target3 = temp_dir.join("target3");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(&target1).unwrap();
        fs::create_dir_all(&target2).unwrap();
        fs::create_dir_all(&target3).unwrap();
        
        let source_file = base.join("file.txt");
        fs::write(&source_file, "content").unwrap();
        
        let config = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "group1"
base = "{}"
sources = ["file.txt"]
target = "{}"
collect = true

[[local]]
name = "group2"
base = "{}"
sources = ["file.txt"]
target = "{}"
collect = true

[[local]]
name = "group3"
base = "{}"
sources = ["file.txt"]
target = "{}"
collect = true
"#,
            base.display(), target1.display(),
            base.display(), target2.display(),
            base.display(), target3.display(),
        )).unwrap();
        
        let expanded = expand_for_collect(config).unwrap();
        let result = validate_no_source_target_conflicts(&expanded);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("group1"));
        assert!(err_msg.contains("group2"));
        assert!(err_msg.contains("group3"));
        
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    // =================================================================
    // Tests for validate_collect_sources
    // =================================================================

    #[test]
    fn test_validate_collect_sources_exact_match() {
        let temp_dir = std::env::temp_dir().join("dt_test_collect_src_1");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let base = temp_dir.join("base");
        fs::create_dir_all(&base).unwrap();
        fs::write(base.join("file.txt"), "content").unwrap();
        
        let config = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "test"
base = "{}"
sources = ["*.txt"]
target = "{}"
collect = true
collect_sources = ["*.txt"]
"#,
            base.display(),
            temp_dir.join("target").display(),
        )).unwrap();
        
        let group = &config.local[0];
        assert!(validate_collect_sources(group).is_ok());
        
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_validate_collect_sources_valid_subset() {
        let temp_dir = std::env::temp_dir().join("dt_test_collect_src_2");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let base = temp_dir.join("base");
        fs::create_dir_all(&base).unwrap();
        fs::write(base.join("2024-01-01.md"), "content").unwrap();
        
        let config = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "test"
base = "{}"
sources = ["*.md"]
target = "{}"
collect = true
collect_sources = ["2024*.md"]
"#,
            base.display(),
            temp_dir.join("target").display(),
        )).unwrap();
        
        let group = &config.local[0];
        assert!(validate_collect_sources(group).is_ok());
        
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_validate_collect_sources_invalid_wider_pattern() {
        let temp_dir = std::env::temp_dir().join("dt_test_collect_src_3");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let base = temp_dir.join("base");
        fs::create_dir_all(&base).unwrap();
        fs::write(base.join("config.toml"), "content").unwrap();
        
        let config = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "test"
base = "{}"
sources = ["config-*.toml"]
target = "{}"
collect = true
collect_sources = ["*.toml"]
"#,
            base.display(),
            temp_dir.join("target").display(),
        )).unwrap();
        
        let group = &config.local[0];
        let result = validate_collect_sources(group);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("may match files outside of sources"));
        
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    // =================================================================
    // Tests for pattern_covers helper
    // =================================================================

    #[test]
    fn test_pattern_covers_exact_match() {
        assert!(pattern_covers("*.md", "*.md"));
    }

    #[test]
    fn test_pattern_covers_star_covers_all() {
        assert!(pattern_covers("*", "*.md"));
        assert!(pattern_covers("*", "any-pattern"));
    }

    #[test]
    fn test_pattern_covers_extension_match() {
        // Same extension patterns - should be covered
        assert!(pattern_covers("*.md", "2024*.md"));
        // Different extensions - not covered
        assert!(!pattern_covers("*.txt", "2024*.md"));
        // Source is generic star - covers all
        assert!(pattern_covers("*", "2024*.md"));
    }

    #[test]
    fn test_pattern_covers_not_covered() {
        assert!(!pattern_covers("config-*.toml", "*.toml"));
        assert!(!pattern_covers("*.rs", "*.md"));
    }

    // =================================================================
    // Tests for detect_group_changes
    // =================================================================

    #[test]
    fn test_detect_group_changes_new_file() {
        let temp_dir = std::env::temp_dir().join("dt_test_detect_1");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let base = temp_dir.join("base");
        let target = temp_dir.join("target");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(&target).unwrap();
        
        // Create source and target files
        fs::write(base.join("file.txt"), "source content").unwrap();
        fs::write(target.join("file.txt"), "modified content").unwrap();
        
        let config = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "test"
base = "{}"
sources = ["file.txt"]
target = "{}"
collect = true
"#,
            base.display(),
            target.display(),
        )).unwrap();
        
        let expanded = expand_for_collect(config).unwrap();
        let group = &expanded.local[0];
        let mut state = CollectionState::default();
        
        let changes = detect_group_changes(group, &mut state).unwrap();
        
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, ChangeType::New);
        assert_eq!(changes[0].relative_path, PathBuf::from("file.txt"));
        
        // State should be updated
        assert!(state.files.contains_key("file.txt"));
        
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_detect_group_changes_modified_file() {
        let temp_dir = std::env::temp_dir().join("dt_test_detect_2");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let base = temp_dir.join("base");
        let target = temp_dir.join("target");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(&target).unwrap();
        
        fs::write(base.join("file.txt"), "source content").unwrap();
        fs::write(target.join("file.txt"), "modified content").unwrap();
        
        let config = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "test"
base = "{}"
sources = ["file.txt"]
target = "{}"
collect = true
"#,
            base.display(),
            target.display(),
        )).unwrap();
        
        let expanded = expand_for_collect(config).unwrap();
        let group = &expanded.local[0];
        let mut state = CollectionState {
            version: 1,
            last_collection: None,
            files: {
                let mut map = HashMap::new();
                map.insert(
                    "file.txt".to_string(),
                    FileState {
                        checksum: calculate_checksum(&base.join("file.txt")).unwrap(),
                        last_seen: chrono::Utc::now().to_rfc3339(),
                    },
                );
                map
            },
        };
        
        let changes = detect_group_changes(group, &mut state).unwrap();
        
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, ChangeType::Modified);
        
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_detect_group_changes_deleted_file() {
        let temp_dir = std::env::temp_dir().join("dt_test_detect_3");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let base = temp_dir.join("base");
        let target = temp_dir.join("target");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(&target).unwrap();
        
        fs::write(base.join("file.txt"), "content").unwrap();
        // target file does NOT exist
        
        let config = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "test"
base = "{}"
sources = ["file.txt"]
target = "{}"
collect = true
"#,
            base.display(),
            target.display(),
        )).unwrap();
        
        let expanded = expand_for_collect(config).unwrap();
        let group = &expanded.local[0];
        let mut state = CollectionState {
            version: 1,
            last_collection: None,
            files: {
                let mut map = HashMap::new();
                map.insert(
                    "file.txt".to_string(),
                    FileState {
                        checksum: "old_checksum".to_string(),
                        last_seen: chrono::Utc::now().to_rfc3339(),
                    },
                );
                map
            },
        };
        
        let changes = detect_group_changes(group, &mut state).unwrap();
        
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, ChangeType::Deleted);
        
        // Should be removed from state
        assert!(!state.files.contains_key("file.txt"));
        
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_detect_group_changes_excluded_file() {
        let temp_dir = std::env::temp_dir().join("dt_test_detect_4");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let base = temp_dir.join("base");
        let target = temp_dir.join("target");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(&target).unwrap();
        
        fs::write(base.join("file.txt"), "content").unwrap();
        fs::write(target.join("file.txt"), "modified content").unwrap();
        
        let config = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "test"
base = "{}"
sources = ["file.txt"]
target = "{}"
collect = true
exclude = ["file.txt"]
"#,
            base.display(),
            target.display(),
        )).unwrap();
        
        let expanded = expand_for_collect(config).unwrap();
        let group = &expanded.local[0];
        let mut state = CollectionState::default();
        
        let changes = detect_group_changes(group, &mut state).unwrap();
        
        // Excluded file should not be detected
        assert_eq!(changes.len(), 0);
        
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_detect_group_changes_orphan_file() {
        let temp_dir = std::env::temp_dir().join("dt_test_detect_5");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let base = temp_dir.join("base");
        let target = temp_dir.join("target");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(&target).unwrap();
        
        // Source does NOT exist, but target does (orphan file)
        fs::write(target.join("orphan.txt"), "orphan content").unwrap();
        
        let config = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "test"
base = "{}"
sources = ["*.txt"]
target = "{}"
collect = true
"#,
            base.display(),
            target.display(),
        )).unwrap();
        
        let expanded = expand_for_collect(config).unwrap();
        let group = &expanded.local[0];
        let mut state = CollectionState::default();
        
        let changes = detect_group_changes(group, &mut state).unwrap();
        
        // Orphan file should be detected as New
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, ChangeType::New);
        assert_eq!(changes[0].relative_path, PathBuf::from("orphan.txt"));
        
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    // =================================================================
    // Integration tests for collect
    // =================================================================

    #[test]
    fn test_collect_dry_run_no_changes() {
        let temp_dir = std::env::temp_dir().join("dt_test_collect_1");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let base = temp_dir.join("base");
        let target = temp_dir.join("target");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(&target).unwrap();
        
        fs::write(base.join("file.txt"), "source").unwrap();
        fs::write(target.join("file.txt"), "modified").unwrap();
        
        let config = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "test"
base = "{}"
sources = ["file.txt"]
target = "{}"
collect = true
"#,
            base.display(),
            target.display(),
        )).unwrap();
        
        let expanded = expand_for_collect(config).unwrap();
        let state_path = temp_dir.join(".dt-state.json");
        
        // Dry run
        let result = collect(&expanded, &state_path, true, false).unwrap();
        
        assert!(result.has_changes());
        assert!(result.is_success());
        
        // Source should NOT be modified
        let source_content = fs::read_to_string(base.join("file.txt")).unwrap();
        assert_eq!(source_content, "source");
        
        // State should NOT be saved
        assert!(!state_path.exists());
        
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_collect_actual_changes() {
        let temp_dir = std::env::temp_dir().join("dt_test_collect_2");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let base = temp_dir.join("base");
        let target = temp_dir.join("target");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(&target).unwrap();
        
        fs::write(base.join("file.txt"), "source").unwrap();
        fs::write(target.join("file.txt"), "modified").unwrap();
        
        let config = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "test"
base = "{}"
sources = ["file.txt"]
target = "{}"
collect = true
"#,
            base.display(),
            target.display(),
        )).unwrap();
        
        let expanded = expand_for_collect(config).unwrap();
        let state_path = temp_dir.join(".dt-state.json");
        
        // Actual collect
        let result = collect(&expanded, &state_path, false, false).unwrap();
        
        assert!(result.has_changes());
        assert!(result.is_success());
        
        // Source should be updated
        let source_content = fs::read_to_string(base.join("file.txt")).unwrap();
        assert_eq!(source_content, "modified");
        
        // State should be saved
        assert!(state_path.exists());
        
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_collect_conflict_aborts() {
        let temp_dir = std::env::temp_dir().join("dt_test_collect_3");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let base = temp_dir.join("base");
        let target1 = temp_dir.join("target1");
        let target2 = temp_dir.join("target2");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(&target1).unwrap();
        fs::create_dir_all(&target2).unwrap();
        
        fs::write(base.join("file.txt"), "content").unwrap();
        fs::write(target1.join("file.txt"), "target1").unwrap();
        fs::write(target2.join("file.txt"), "target2").unwrap();
        
        let config = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "group1"
base = "{}"
sources = ["file.txt"]
target = "{}"
collect = true

[[local]]
name = "group2"
base = "{}"
sources = ["file.txt"]
target = "{}"
collect = true
"#,
            base.display(), target1.display(),
            base.display(), target2.display(),
        )).unwrap();
        
        let expanded = expand_for_collect(config).unwrap();
        let state_path = temp_dir.join(".dt-state.json");
        
        // Should fail due to conflict
        let result = collect(&expanded, &state_path, false, false);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Source-Target conflict"));
        
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_collect_no_changes_when_clean() {
        let temp_dir = std::env::temp_dir().join("dt_test_collect_4");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let base = temp_dir.join("base");
        let target = temp_dir.join("target");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(&target).unwrap();
        
        // Same content in both
        fs::write(base.join("file.txt"), "same content").unwrap();
        fs::write(target.join("file.txt"), "same content").unwrap();
        
        let config = DTConfig::from_str(&format!(
            r#"
[[local]]
name = "test"
base = "{}"
sources = ["file.txt"]
target = "{}"
collect = true
"#,
            base.display(),
            target.display(),
        )).unwrap();
        
        let expanded = expand_for_collect(config).unwrap();
        let state_path = temp_dir.join(".dt-state.json");
        
        // Pre-populate state with correct checksum
        let mut state = CollectionState::default();
        state.files.insert(
            "file.txt".to_string(),
            FileState {
                checksum: calculate_checksum(&target.join("file.txt")).unwrap(),
                last_seen: chrono::Utc::now().to_rfc3339(),
            },
        );
        save_state(&state_path, &state).unwrap();
        
        // Collect should find no changes
        let result = collect(&expanded, &state_path, false, false).unwrap();
        
        assert!(!result.has_changes());
        assert!(result.is_success());
        
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
