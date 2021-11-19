# Syncing Methods

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
[`global.staging`](/config/key-references#staging) and
[`name`](/config/key-references#name)), then symlinks the staged items to
target.

## Default Method

`dt-cli` chooses `Symlink` as the default behaviour.  The added _staging_ step:

- Makes it possible to organize sources according to their group
  [`name`](/config/key-references#name)s, which `Copy` does not.
  :::tip
  This means it allows human-readable directory structures, because groups are
  organized by your given [`name`](/config/key-references#name)s.  You can
  also create a git repository at the staging root directory if you want,
  :::
- Makes it possible to control permission of organized items from system-level
  `sources` which you shouldn't directly modify.
- When the target and source are the same (by accident), `Copy` does not
  guarantee integrity of the source item, while `Symlink` preserves the file
  content in the staging directory.
- Make all further symlinks point at most to the staged items.
  :::tip
  This particularly helpful when you manage user-scope systemd services
  with symlinks.  According to
  [`systemctl(1)`](https://man.archlinux.org/man/systemctl.1):
  
  > Disables one or more units. This removes all symlinks to the unit files
  > backing the specified units from the unit configuration directory, and
  > hence undoes any changes made by enable or link. Note that this removes
  > all symlinks to matching unit files, including manually created symlinks,
  > and not just those actually created by enable or link.
  
  That said, when disabling services (with `systemctl --user disable`),
  `systemctl` removes all symlinks (**including user-created ones!**).
  
  With this added _staging_ process, your source files will be well protected.
  :::
- Protects original items if you want to make experimental changes.

## Overriding

You can always override the default syncing method to `Copy` conveniently by
adding `method = "Copy"` to the `[global]` section:

```toml
[global]
method = "Copy"
```

Or specify the syncing method for a given group similarly:

```toml
[[local]]
method = "Copy"
```