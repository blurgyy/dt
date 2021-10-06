# DT

`DT` allows you to sync/populate configuration files at will.  It currently
provides a CLI tool [`dt-cli`](./dt-cli), built with backend [`dt-core`](./dt-core).

## Usage

The command line interface `dt-cli` takes a path to the configuration file as
argument and issues the syncing process defined in the configuration file.
Passing `-d|--dry-run` to `dt-cli` will show changes to be made without
actually making those changes, for example:

```shell
$ dt-cli path/to/config --dry-run
```

### Example

A minimal working configuration file to sync all files that matches
`*init.vim` from `~/myDotfiles/nvim` to `~/.config/nvim` can be written as:

```toml
[[local]]
name = "Neovim Configs"
basedir = "~/myDotfiles/nvim"
sources = ["*init.vim"]
target = "~/.config/nvim"
```

**Stop here if you don't know what you are doing, or have not backed up
existing files under `~/.config/nvim`.**

Call `dt-cli` with path to above configuration file as argument to sync it.

For more detailed usage, see <https://dt.blurgy.xyz/>, for details about
`dt-core`, see <https://docs.rs/dt-core/latest/dt_core/>.

## Install

To install `dt-cli`, you can:

- Download pre-built executables from GitHub: <https://github.com/blurgyy/dt/releases/latest>
- Or install from <https://crates.io>:
  
  ```shell
  $ cargo install dt-cli
  ```
  
- Or build from source:
  
  ```shell
  $ git clone github.com:blurgyy/dt.git
  $ cd dt
  $ cargo test --release
  $ cargo install --path=dt-cli
  ```