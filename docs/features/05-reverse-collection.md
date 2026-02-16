# Reverse Collection

:::info
This chapter explains how to use the `dt collect` command to sync changes from
your local configuration files back to your dotfiles repository.

For configuration examples, please refer to the
<sub>[[**Examples**](#)]</sub> link below.
:::

## Background

Traditionally, `dt-cli` works in one direction: from your dotfiles repository
(source) to your local machine (target). However, you often make quick edits
to configuration files directly on your local machine. The reverse collection
feature allows you to detect these local changes and sync them back to your
repository.

## Use Cases

- **Quick fixes**: You edited a config file to fix an urgent issue and want to
  persist the change.
- **Multiple machines**: You configured something on one machine and want to
  propagate it to your dotfiles repo.
- **Backup**: Ensure your repository always reflects your latest local configs.

## How It Works

`dt collect` compares the SHA-256 checksum of files in your target directories
against a stored state. When it detects:

- **New files**: Files exist in target but not in state
- **Modified files**: Checksum differs from state
- **Deleted files**: Files in state but not on disk

It copies the changed files back to your source repository.

## Basic Usage

```bash
# Detect and collect all changes
dt collect

# Preview what would be collected (dry-run)
dt collect --dry-run

# Verbose output
dt collect -v
```

## State File

The collection state is stored in `.dt-state.json` alongside your config file.
This file tracks file checksums and should be committed to your repository.

```json
{
  "version": 1,
  "last_collection": "2026-02-16T15:00:00Z",
  "files": {
    "config/nvim/init.vim": {
      "checksum": "a1b2c3d4...",
      "last_seen": "2026-02-16T15:00:00Z"
    }
  }
}
```

## Limitations

The following features are planned but not yet implemented:

- `--force` flag to ignore state and collect all files
- `collect_options.exclude_patterns` for filtering files
- `collect_options.include_hidden` for hidden file handling

See the [design document](/design.md) for technical details.
