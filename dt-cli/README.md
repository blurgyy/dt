# DT

[![release](https://github.com/blurgyy/dt/actions/workflows/release.yml/badge.svg)](https://github.com/blurgyy/dt/actions/workflows/release.yml)
[![tests](https://github.com/blurgyy/dt/actions/workflows/tests.yml/badge.svg)](https://github.com/blurgyy/dt/actions/workflows/tests.yml)
[![docs](https://github.com/blurgyy/dt/actions/workflows/docs.yml/badge.svg)](https://dt.cli.rs/)
[![crates.io](https://img.shields.io/crates/v/dt-cli?style=flat&labelColor=1C2C2E&color=C96329&logo=Rust&logoColor=white)](https://crates.io/crates/dt-cli)

`DT` allows you to sync/populate configuration files at will.  It currently
provides a CLI tool `dt-cli`.

## Usage

The command line interface `dt-cli` accepts a path to the configuration file
as an argument and performs the syncing process specified in the file.

Configuration guides and detailed usages can be found in the
[documentations](https://dt.cli.rs/).

### Example

A simple working configuration file to sync all files from `~/dt/nvim` to
`~/.config/nvim` that matches `*init.vim` can be written as:

```toml
[[local]]
name = "Neovim Configs"
base = "~/dt/nvim"
sources = ["*init.vim"]
target = "~/.config/nvim"
```

:warning: **STOP HERE if you don't know what you are doing, or have not backed
up existing files under `~/.config/nvim`.**

Save above config to `~/.config/dt/cli.toml` and run

```shell
$ dt-cli -c ~/.config/dt/cli.toml
```

to start syncing.  Note the path in this example (`~/.config/dt/cli.toml`) is
also the default path, so the below command (calling `dt-cli` with no argument)
does the same thing as above:

```shell
$ dt-cli
```

**Other command line flags & options**

| Flags | Description |
|---:|:---|
| `-d\|--dry-run` | Shows changes to be made without actually syncing files. |
| `-h\|--help` | Prints help information. |
| `-q\|--quiet` | Decreases logging verbosity. |
| `-v\|--verbose` | Increases logging verbosity. |
| `-V\|--version` | Prints version information. |

| Options | Description |
|---:|:---|
| `-c\|--config-path` `<path>` | Specifies path to config file. |

| Args | Description |
|---:|:---|
| `<group-name>...` | Specifies name(s) of the group(s) to be processed |

## Install

### AUR

`dt-cli` is in the [AUR](https://aur.archlinux.org/packages/dt-cli/), you can
install it with your favorite package manager:

```shell
$ paru -S dt-cli
```

### Alternative ways

Alternatively, you can:

- Download latest [release](https://github.com/blurgyy/dt/releases/latest)
  from GitHub
- Install from [crates.io](https://crates.io/crates/dt-cli/):

  ```shell
  $ cargo install dt-cli
  ```

- Build from source:

  ```shell
  $ git clone git@github.com:blurgyy/dt.git
  $ cd dt
  $ cargo test --release
  $ cargo install --path=dt-cli
  ```

## Contributing

There are numerous ways to help with this project.  Let's [get
started](https://github.com/blurgyy/dt/blob/main/CONTRIBUTING.md)!

## License

Licensed under the the MIT license <http://opensource.org/licenses/MIT> or
Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0>, at
your option.  This file may not be copied, modified, or distributed except
according to those terms.
