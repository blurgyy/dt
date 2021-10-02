# Roadmap

- [x] Expand tilde and globs in source paths
- [x] Add ignore patterns to LocalSyncConfig
- [x] Add [global] section to config file, which defines global settings like
      staging directory, and default behaviours like whether to
      `copy`/`symlink` when syncing.
- [ ] Implement staging when `global.method` is `symlink`
- [x] Add `basedir` to LocalSyncConfig for easier configuring in sources
- [ ] Manage permission bits on a per-group basis
- [x] Return error when `sources` contains `.*` or similar globs, because this
      glob also matches current directory (`.`) and parent directory (`..`)

> Author: Blurgy <gy@blurgy.xyz>
> Date:   Sep 29 2021, 00:18 [CST]
