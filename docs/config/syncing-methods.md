# Syncing methods

Until the last section, no comments has been given on the **stage** ->
**symlink** steps.  This section explains all the details a user wants to
know about this process.

:::tip
If you are interested in all the details of the process, I refer you to the
implementation of `dt_core::syncing::sync_core`
[here](https://github.com/blurgyy/dt/blob/main/dt-core/src/syncing.rs).
:::

## Overview

There are 2 available syncing methods: `Copy` and `Symlink`, where `Symlink`
is the chosen default.

### `Copy`

Directly copies source items defined in `sources` arrays to target.

### `Symlink`

First copies source items defined in `sources` arrays (this is called
_staging_) to **current group's** staging directory (see
[`global.staging`](/config/key-references.html#staging) and
[`name`](/config/key-references.html#name)), then symlinks the staged items to
target.

## Default Method

I have chosen `Symlink` to be the default behaviour of `dt-cli`.  The added
_staging_ step:

- Makes it possible to organize sources according to their group
  [`names`](/config/key-references.html#name), which `Copy` does not.
- Makes it possible to control permission of organized items from system-level
  `sources` which you shouldn't directly modify.
- When the target and source are the same (by accident), `Copy` does not
  guarantee integrity of the source item, while `Symlink` preserves the file
  content in the staging directory.

## Overriding

You can always override the default syncing method to `Copy` conveniently by
adding `method = "Copy"` to the `[global]` section:

```toml
[global]
method = "Copy"
```
