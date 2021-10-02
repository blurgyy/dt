# Roadmap

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

> Author: Blurgy <gy@blurgy.xyz>
> Date:   Sep 29 2021, 00:18 [CST]
