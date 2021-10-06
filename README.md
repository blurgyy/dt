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

For more detailed usage, see <https://dt.blurgy.xyz/>, for details about
`dt-core`, see </#>.

## Install

To install `dt-cli`, you can:

- Download pre-built executables from GitHub: <https://github.com/blurgyy/dt/releases/latest>
- Or build from source:
  
  ```shell
  $ git clone github.com:blurgyy/dt.git
  $ cd dt
  $ cargo test --release
  $ cargo install --path=dt-cli
  ```
