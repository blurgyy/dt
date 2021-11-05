# Error Handling

For an application that works with your daily configuration files, it's
crucial that it handles error in an expectable manner.

`dt-cli` looks for possible errors in 3 stages throughout its running course,
this section describes the checking in details.

## Config Validating

Firstly, after a config file has been successfully loaded into memory,
`dt-cli` validates each field of the config object.  Specifically, the
following cases are considered invalid:

- Any group that has empty `name`/`basedir`/`target`
- Any group name that
  - is also another group's name (duplicated group name)
  - contains `/` (used for creating subdirectory under `staging` directory)
- Any group that has the same `basedir` and `target`
- Any group whose `basedir` contains any occurrences of `hostname_sep`
- Any group whose `sources` contains any item that contains any occurrences of
  `hostname_sep`
- Any source item that:
  - begins with `../` (references the parent of base directory)
  - begins with `~` or `/` (path is absolute)
  - is `.*` (bad globbing pattern, it will expand to parent directory)
  - ends with `/.*` (bad globbing pattern)

:::tip
Checking operations in this step does not touch the filesystem, only matchs
string patterns.  This is for spotting obvious errors as fast as possible.
:::

## Sources Expanding

If the above validating step passed successfully, `dt-cli` begins to iterate
through every group, recursively expand all sources according to their file
hierarchy, the `basedir`s are also expanded to [host-specific](/host-specific)
ones wherever possible.  The following cases are considered invalid while
expanding `sources` and `basedir`:

- The group's `basedir` is non-existent
- The group's `basedir` exists but is not a directory
- The group's `target` exists and is not a directory
- The group's `target` is non-existent but cannot be created
- When any group uses the `Symlink` [syncing
  method](/config/guide/syncing-methods):
  - `staging` exists but iis not a directory
  - `staging` is non-existent but cannot be created

:::info
Broken symlinks and item types other than `file` or `directory` are ignored
and warned during expanding.  These items will not cause errors.
:::

:::tip
Checking operations does not create or modify anything, only query the
filesystem to check existences and permissions.
:::

## Syncing

Finally, if no error can be found, `dt-cli` carefully (and efficiently) syncs
the **expanded** source items to the target directory.  During this process,
according to the values of
[`allow_overwrite`](/config/key-references#allow-overwrite-1), different
logging levels will be set when encountered with existing target items.  Any
other cases (e.g. a directory changes its permission to readonly) unhandled by
the above 2 steps will cause `dt-cli` to panic.

:::tip
If you think there's anything missing here, your contribution is welcome!
Start by following the [contributing guide](/contributing).
:::
