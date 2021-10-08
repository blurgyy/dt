---
title: Overview
---

# Overview

## Synopsis

`dt-cli` is a highly customizable dotfile manager.

## Usage

`dt-cli` takes a configuration file and issues syncing process defined in the
config file.  For example, a config file lies at `~/.config/dt/cli.toml`, use

```shell
$ dt-cli ~/.config/dt/cli.toml
```

to run with defined behaviours in config.

**Command line options & flags**

- `-d|--dry-run`: Show changes to be made without actually syncing files.

## Configuration

See the [hands-on guide](/config/guide/) for creating a configuration file.
