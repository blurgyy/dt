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
- [x] Return error when `sources` contains `.*` or similar globs, because this
      glob also matches current directory (`.`) and parent directory (`..`)
- [x] Expand environment variables in `local.basedir`, `local.sources`,
      `local.target`
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
- [ ] Add `global.per_host` and `global.hostname_sep` as default value,
      overrided by group configs

## CLI

- [ ] Find config in `$XDG_CONFIG_HOME/dt/config.toml` by default

> Author: Blurgy <gy@blurgy.xyz>
> Date:   Sep 29 2021, 00:18 [CST]
