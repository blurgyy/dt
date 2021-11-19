# Overview

## Name

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

**Other command line flags & options**

| Flags | Description |
|---:|:---|
| `-d\|--dry-run` | Shows changes to be made without actually syncing files. |
| `-h\|--help` | Prints help information. |
| `-q\|--quiet` | Decreases logging verbosity. |
| `-v\|--verbose` | Increases logging verbosity. |
| `-V` | Prints version information. |

| Options | Description |
|---:|:---|
| `-c\|--config-path` `<path>` | Specifies path to config file. |

| Args | Description |
|---:|:---|
| `<group-name>...` | Specifies name(s) of the group(s) to be processed |

## Configuration

Create a configuration file by following the steps in the [hands-on
guide](/config/guide/).
