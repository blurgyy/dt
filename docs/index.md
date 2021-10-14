---
title: Overview
---

# Overview

## Synopsis

`dt-cli` is a highly customizable dotfile manager.

## Usage

`dt-cli` takes a configuration file and issues syncing process defined in the
config file.  Put your config file at `~/.config/dt/cli.toml` and run

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

## Configuration

See the [hands-on guide](/config/guide/) for creating a configuration file.
