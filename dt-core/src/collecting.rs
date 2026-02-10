//! Reverse collection functionality for DT.
//!
//! This module provides the ability to detect changes in target directories
//! and sync them back to the source repository.

use std::{
    collections::HashMap,
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    config::{DTConfig, LocalGroup},
    error::{Error as AppError, Result},
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

/// A detected change
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
    
    // Get the current timestamp
    let now = chrono::Utc::now().to_rfc3339();
    
    // Expand sources to get all files
    let base = &group.base;
    let target = &group.target;
    
    // Collect all files in the target directory that correspond to sources
    for source in &group.sources {
        let target_path = target.join(source);
        let source_path = base.join(source);
        let relative_path = source.clone();
        
        if !target_path.exists() {
            // File might have been deleted
            if state.files.contains_key(&relative_path.to_string_lossy().to_string()) {
                changes.push(Change {
                    change_type: ChangeType::Deleted,
                    relative_path: relative_path.clone(),
                    target_path: target_path.clone(),
                    source_path: source_path.clone(),
                });
                state.files.remove(&relative_path.to_string_lossy().to_string());
            }
            continue;
        }
        
        // Calculate current checksum
        let current_checksum = calculate_checksum(&target_path)?;
        let path_key = relative_path.to_string_lossy().to_string();
        
        match state.files.get(&path_key) {
            None => {
                // New file
                changes.push(Change {
                    change_type: ChangeType::New,
                    relative_path: relative_path.clone(),
                    target_path: target_path.clone(),
                    source_path: source_path.clone(),
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
                changes.push(Change {
                    change_type: ChangeType::Modified,
                    relative_path: relative_path.clone(),
                    target_path: target_path.clone(),
                    source_path: source_path.clone(),
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
    
    // Update last_collection timestamp
    state.last_collection = Some(now);
    
    Ok(changes)
}

/// Collect changes from all enabled groups
pub fn collect(
    config: &DTConfig,
    state_path: &Path,
    dry_run: bool,
) -> Result<Vec<Change>> {
    let mut state = load_state(state_path)?;
    let mut all_changes = Vec::new();
    
    for group in &config.local {
        // Skip groups without collect enabled
        if !group.is_collect_enabled() {
            continue;
        }
        
        let changes = detect_group_changes(group, &mut state)?;
        
        if !dry_run {
            // Apply changes: copy target files back to source
            for change in &changes {
                if change.change_type == ChangeType::Deleted {
                    // Optionally delete source file
                    if change.source_path.exists() {
                        fs::remove_file(&change.source_path)?;
                    }
                } else {
                    // Copy target to source
                    if let Some(parent) = change.source_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(&change.target_path, &change.source_path)?;
                }
            }
        }
        
        all_changes.extend(changes);
    }
    
    if !dry_run {
        save_state(state_path, &state)?;
    }
    
    Ok(all_changes)
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
