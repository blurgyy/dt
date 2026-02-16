# DT Reverse Collection - Design Document

## Overview
This document defines the design for the "reverse collection" feature that detects local configuration changes and syncs them back to the source repository.

## 1. Change Detection Strategy: Checksum vs mtime

### Decision: Use **Checksum (SHA-256)** for change detection

#### Rationale

| Criteria | Checksum | mtime |
|----------|----------|-------|
| **Accuracy** | ✅ Detects content changes only | ❌ False positives from touch, git operations |
| **Reliability** | ✅ Content-hash is deterministic | ❌ Filesystem-dependent, can drift |
| **Conflict Detection** | ✅ Can detect if source changed too | ❌ Hard to compare |
| **Performance** | ❌ Requires reading file content | ✅ Just stat() call |
| **Storage** | ❌ Needs state file to store hashes | ✅ No extra storage |

#### Why Checksum Wins
1. **Accuracy is critical** for config sync - we only want to sync when content actually changes
2. **mtime is unreliable** - git operations, editor saves, and filesystem operations can update mtime without content changes
3. **dt already uses content-aware sync** - the forward sync checks content, reverse should too
4. **State file overhead is acceptable** - configs are small files, hash computation is fast

### Implementation
- Use SHA-256 for file content hashing
- Store state in `.dt-state.json` alongside `dt.toml`
- State format: `{ "path": "sha256_hash", ... }`
- Hash computation: `sha256(file_content)`

## 2. State File Schema

### File: `.dt-state.json`

```json
{
  "version": 1,
  "last_collection": "2026-02-10T14:30:00Z",
  "files": {
    "config/dotfiles.json": {
      "checksum": "a1b2c3d4...",
      "last_seen": "2026-02-10T14:30:00Z"
    },
    "credentials/auth-profiles.json": {
      "checksum": "e5f6g7h8...",
      "last_seen": "2026-02-10T14:30:00Z"
    }
  }
}
```

### Schema Fields

| Field | Type | Description |
|-------|------|-------------|
| `version` | integer | State file format version |
| `last_collection` | ISO-8601 timestamp | Last time collection was run |
| `files` | object | Map of relative path → file state |
| `files.{path}.checksum` | string (hex) | SHA-256 hash of file content |
| `files.{path}.last_seen` | ISO-8601 timestamp | When this checksum was recorded |

### State File Location
- Default: Same directory as `dt.toml`
- Configurable via `dt.toml`: `state_file = ".dt-state.json"`

## 3. Change Detection Algorithm

### Algorithm: `detect_changes()`

```rust
fn detect_changes(config: &Config, state: &State) -> Vec<Change> {
    let mut changes = Vec::new();
    
    for group in &config.groups {
        if !group.collect_enabled {
            continue; // Skip groups without collect flag
        }
        
        for source in &group.sources {
            let current_hash = sha256_file(&source.target_path);
            let recorded_hash = state.files.get(&source.relative_path);
            
            match recorded_hash {
                None => {
                    // New file not in state
                    changes.push(Change::New(source.clone()));
                }
                Some(recorded) if recorded.checksum != current_hash => {
                    // File changed
                    changes.push(Change::Modified(source.clone()));
                }
                _ => {
                    // No change
                }
            }
            
            // Update state with current hash
            state.files.insert(
                source.relative_path.clone(),
                FileState {
                    checksum: current_hash,
                    last_seen: now(),
                }
            );
        }
        
        // Detect deletions (files in state but not on disk)
        for (path, _) in &state.files {
            if !path_exists(&group.target.join(path)) {
                changes.push(Change::Deleted(path.clone()));
            }
        }
    }
    
    changes
}
```

### Change Types

```rust
enum Change {
    New(Source),        // File exists in target but not in state
    Modified(Source),   // Checksum differs from state
    Deleted(PathBuf),   // File in state but not on disk
}
```

## 4. CLI Interface

### New Commands

```bash
# Collect changes from targets back to source
dt collect

# Dry-run: show what would be collected
dt collect --dry-run

# Force collection (ignore state)
dt collect --force

# Collection with verbose output
dt collect -v
```

### Configuration Schema (dt.toml)

```toml
[[groups]]
name = "config"
sources = [
    { path = "config/dotfiles.json", target = "~/.dotfiles/dotfiles.json" }
]

# New: Enable reverse collection for this group
collect = true

# New: Collection-specific options (optional)
[groups.collect_options]
exclude_patterns = ["*.tmp", "*.bak"]  # Skip these
include_hidden = false                  # Don't collect dotfiles unless listed
```

## 5. Collection Workflow

```
┌─────────────────┐
│   dt collect    │
└────────┬────────┘
         │
         ▼
┌──────────────────────────┐
│ 1. Load config (dt.toml) │
└────────┬─────────────────┘
         │
         ▼
┌──────────────────────────┐
│ 2. Load state (.dt-state)│
└────────┬─────────────────┘
         │
         ▼
┌──────────────────────────┐
│ 3. Detect changes        │
│    (checksum comparison) │
└────────┬─────────────────┘
         │
         ▼
┌──────────────────────────┐
│ 4. Copy changed files    │
│    target → source       │
└────────┬─────────────────┘
         │
         ▼
┌──────────────────────────┐
│ 5. Update state file     │
└────────┬─────────────────┘
         │
         ▼
┌──────────────────────────┐
│ 6. Report changes        │
└──────────────────────────┘
```

## 6. Edge Cases

| Case | Handling |
|------|----------|
| **Conflict** | Source file changed too → Error/warning, manual resolution |
| **Permission denied** | Skip file, log warning, continue |
| **File deleted** | Remove from state, optionally delete from source |
| **New file in target** | Copy to source, add to state |
| **Binary files** | Hash works fine, treat same as text |
| **Symlinks** | Follow symlinks, hash the target content |
| **Large files** | Config files are small, no special handling needed |

## 7. Integration with HEARTBEAT

Once implemented, HEARTBEAT.md will be updated to use:

```bash
# Instead of manual diff + cp + git
dt collect && cd ~/.local/share/dotfiles-configs && git commit -a && git push rad master
```

---

**Status**: Design Complete ✓  
**Next**: Phase 3 - Implementation
