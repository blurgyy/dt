# Reverse Collection

This guide shows how to configure reverse collection to sync local changes
back to your dotfiles repository.

## Enabling Collection

Add `collect = true` to any group you want to enable reverse collection for:

```toml
[[local]]
name = "Neovim"
base = "~/dotfiles/nvim"
sources = ["*.vim", "*.lua"]
target = "~/.config/nvim"
collect = true  # Enable reverse collection for this group
```

## Workflow Example

### Initial Setup

1. Configure your group with `collect = true`
2. Run initial sync:
   ```bash
   dt-cli sync
   ```
3. Run first collection to establish state:
   ```bash
   dt collect
   ```
4. Commit the generated `.dt-state.json`:
   ```bash
   git add .dt-state.json
   git commit -m "Add collection state"
   ```

### Daily Usage

1. Edit your local config files as needed
2. Preview what changed:
   ```bash
   dt collect --dry-run
   ```
3. Collect the changes:
   ```bash
   dt collect
   ```
4. Review and commit:
   ```bash
   git status
   git diff
   git add -A
   git commit -m "Sync local changes"
   ```

## Multiple Groups

You can selectively enable collection for specific groups:

```toml
[[local]]
name = "Neovim"
base = "~/dotfiles/nvim"
sources = ["*.vim"]
target = "~/.config/nvim"
collect = true

[[local]]
name = "SSH"
base = "~/dotfiles/ssh"
sources = ["config"]
target = "~/.ssh"
# collect not set - this group won't be collected
```

## Best Practices

### Commit State File

Always commit `.dt-state.json` to your repository. This ensures consistent
change detection across machines.

### Review Before Committing

Use `--dry-run` to review changes before collecting:

```bash
dt collect --dry-run
dt collect
```

### Exclude Sensitive Files

Be careful with groups that might contain sensitive data. Consider using
`exclude` patterns in your regular sync config:

```toml
[[local]]
name = "SSH"
base = "~/dotfiles/ssh"
sources = ["config", "authorized_keys"]
target = "~/.ssh"
exclude = ["id_*", "*.pem"]  # Never sync private keys
collect = true
```

## Troubleshooting

### Everything shows as "New"

This happens when `.dt-state.json` is missing or corrupted. Run:

```bash
dt collect
```
to regenerate the state file, then commit it.

### Permission Denied

Ensure you have write access to both source and target directories. The
collect command needs to read target files and write to source files.
