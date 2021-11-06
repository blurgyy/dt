# Roadmap

## Core

- [x] Expand tilde and globs in source paths
- [x] Add ignore patterns to LocalSyncConfig
- [x] Add [global] section to config file, which defines global settings like
      staging directory, and default behaviours like whether to
      `copy`/`symlink` when syncing.
- [x] Implement staging when `global.method` is `symlink`
  - [x] Make `local.basedir` mandatory (instead of optional) so as to preserve
        directory structure in the stating directory
    - [x] Do not expand tilde in sources, because sources are relative paths
          after making basedir mandatory
- [x] Add `basedir` to LocalSyncConfig for easier configuring in sources
- [ ] Manage permission bits on a per-group basis
  - [x] Handle permission denied error when target file is created by current
        user but the `write` permission is not set
  - [ ] Set permissions after all syncing process
- [x] Return error when `sources` contains `.*` or similar globs, because this
      glob also matches current directory (`.`) and parent directory (`..`)
- [ ] Expand environment variables in `local.basedir`, `local.sources`,
      `local.target` and `staging`
- [x] Handle non-existing source, give warnings when encountered
- [ ] ~~Add `local.for` to separate groups for different machines w.r.t. their
      hostname (with `gethostname` crate)~~
- [ ] Add `global.ignored` as fallback ignoring list
- [x] Add `local.name` for local groups as namespaces, to properly handle
      files from different groups having the same relative path to their
      basedir.
- [ ] Let `ignored` array(s) match more intelligently
- [x] Add `local.per_host` to check for per-host syncing items for groups that
      has this key set to `true`
- [x] Make `local.per_host` default to `true`, or remove this from config
- [x] Add group name to logging message
- [x] Do not touch filesystem when parsing config for faster execution, only
      query filesystem when syncing
- [ ] Warn about items without a source in the staging directory
- [x] Add `global.hostname_sep` as default value, overrided by group configs
- [x] Deny sources that start with "./" or "../" or similar
- [ ] Define group type (like one of "General", "App", "Dropin"), to define
      priority when syncing (priority order: Dropin > App > General), so that
      user won't have to carefully separate configurations of specific
      applications from its parent directory (like separating `~/.ssh` (which
      will be of type "App") from `~` (which will be of type "General"), or
      separating `~/.config/nvim` from `~/.config`)
- [x] Recursively expand all sources in function `syncing::expand`
- [x] Add README.md
- [x] Do not remove host-specific suffix in staging directory
- [ ] Warn when a group's `target` is inside another group's `basedir`
- [ ] Set maximum recursion depth when expanding sources
- [x] Deny empty `name`/`basedir`/`target`
- [x] Better logging messages:
  - error: shows errors
  - warn: shows warnings
  - info: shows coarse step messages, like "syncing group [xxx]"
  - debug: shows how items are synced
  - trace: shows messages that contains multiple lines
  Paths in log messages should be single quoted.
- [ ] Keep track of items that are synced last time, so that deletion of
      source items can also be propagated to local fs properly.
- [ ] ?Make `target` an array
- [ ] ?Expand globs in `target` to sync to multiple target directories (use
      case: `user.js` under Firefox user profile, which is named
      `xxxxxxxx.$profile_name` and can be globbed)
- [x] Do not _remove_ existing target file when _overwriting_, because it
      causes some X compositor (like picom) to flash or fail to
      automatically load config file after overwriting
- [ ] Add `global.include` array to allow including other config files

## CLI

- [x] Find config in `$XDG_CONFIG_HOME/dt/cli.toml` by default
- [x] Add command line option to specify which group to sync via passing name
      of the group
- [x] Change default config path to `$XDG_CONFIG_HOME/dt/config.toml`

## Server

- [ ] Serve files with an HTTP server, grouped by their group names
- [ ] Use the same config layout as `dt-cli`
- [ ] Add `confidential` flag to `local` group to determin whether this group
      should be served in the HTTP server
- [ ] Make URL prefix (like `/raw/`) configurable
- [ ] Optionally serve static files at a given root
- [ ] Encryption

> Author: Blurgy <gy@blurgy.xyz>
> Date:   Sep 29 2021, 00:18 [CST]
