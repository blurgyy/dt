# DT

[![license: MIT OR Apache 2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache%202.0-blue.svg)](./LICENSE)
[![release](https://github.com/blurgyy/dt/actions/workflows/release.yml/badge.svg)](https://github.com/blurgyy/dt/actions/workflows/release.yml)
[![tests](https://github.com/blurgyy/dt/actions/workflows/tests.yml/badge.svg)](https://github.com/blurgyy/dt/actions/workflows/tests.yml)
[![docs](https://github.com/blurgyy/dt/actions/workflows/docs.yml/badge.svg)](https://dt-cli-docs.blurgy.xyz/)

`DT` allows you to sync/populate configuration files at will.  It currently
provides a CLI tool [`dt-cli`](./dt-cli), built with backend
[`dt-core`](./dt-core).

## Usage

The command line interface `dt-cli` takes a path to the configuration file as
argument and issues the syncing process defined in the configuration file.

See [documentations](https://dt-cli-docs.blurgy.xyz/) for configuration guides
and detailed usages.

### Example

A minimal working configuration file to sync all files that matches
`*init.vim` from `~/dt/nvim` to `~/.config/nvim` can be written as:

```toml
[[local]]
name = "Neovim Configs"
basedir = "~/dt/nvim"
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

**Other command line options & flags**

| Options/Flags | Description |
|---:|:---|
| `-c\|--config-path <path>` | Specifies path to config file. |
| `-d\|--dry-run` | Shows changes to be made without actually syncing files. |
| `-v\|--verbose` | Increases logging verbosity. |
| `-q\|--quiet` | Decreases logging verbosity. |
| `-h\|--help` | Prints help information. |
| `-V` | Prints version information. |

## Install

### AUR

`dt-cli` is in the [AUR](https://aur.archlinux.org/packages/dt-cli/), you can
install it with your favorite package manager:

```shell
$ paru -S dt-cli
```

### Alternative Ways

Alternatively, you can:

- Download latest [release](https://github.com/blurgyy/dt/releases/latest)
  from GitHub
- Install from [crates.io](https://crates.io/crates/dt-cli/):
  
  ```shell
  $ cargo install dt-cli
  ```
  
- Build from source:
  
  ```shell
  $ git clone github.com:blurgyy/dt.git
  $ cd dt
  $ cargo test --release
  $ cargo install --path=dt-cli
  ```

## Contributing

There are many ways to contribute to this project. Get started
[here](./CONTRIBUTING.md)!
