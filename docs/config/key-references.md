# Key References

This chapter lists available configuration keys.

## Global

> [optional]

A global object is defined in the `[global]` section.

### `staging`

> [optional] string

Defines the staging root directory, does not matter when [syncing
method](#method) is set to `Copy`.  If omitted, uses
`$XDG_DATA_HOME/dt/staging` if environment variable `XDG_DATA_HOME` is set,
otherwise uses `$HOME/.local/share/dt/staging`.

### `method`

> [optional] `Copy`|`Symlink`

The syncing method.  Available values are:

- `Copy`
- `Symlink`

If omitted, uses `Symlink`.  When `method` is set to `Copy`, the
[`staging`](#staging) setting will be disabled.  For more details, see
[Syncing Methods](/config/guide/03-syncing-methods).

### `allow_overwrite`

> [optional] bool

Whether to allow overwriting existing files.  If omitted, uses `false`.

This alters syncing behaviours when the target file exists.  If set to `true`,
no errors/warnings will be omitted when the target file exists; otherwise
reports error and skips the existing item.  Using dry run to spot the existing
files before syncing is recommended.

### `hostname_sep`

> [optional] string

Defines the default value when a [group's `hostname_sep`](#hostname-sep-1) is
not set.  If omitted, uses `@@`.

### `rename`

> [optional] array of 2-tuples

Global item renaming rules.

Rules defined here will be prepended to renaming rules of each group.  For
full explanation and usage, see [Filename
Manipulation](/features/03-filename-manipulating).

## Local Groups

> [required] array of [`LocalSyncConfig`](https://docs.rs/dt-core/latest/dt_core/config/struct.LocalSyncConfig.html)s

Local groups are defined in `[[local]]` arrays.

### `name`

> [required] string

A _unique_ name given to this group, used for inferring current group's
staging directory.  For example, if `global.staging` is set to `/some/dir`, a
local group with `name` set to `Dotfiles` will have a staging directory as
`/some/dir/Dotfiles`.  Cannot contain slash (`/`).

### `scope`

> [optional] `General`|`App`|`Dropin`

Priority of the current group, useful when selecting groups via command line.
The syncing priority order is `Dropin` > `App` > `General`.  The first group
in the config file has the highest priority when multiple groups with a same
`scope` contain a same item.  If omitted, uses `General`.

### `basedir`

> [required] string

The base directory of all source items.  This simplifies configuration files
with common prefixes in `local.sources` array.

#### Example

For a directory structure like:

```plain
dt/
├── dt-core/
│  └── src/
│     └── config.rs
├── dt-cli/
│  └── src/
│     └── main.rs
└── README.md
```

Consider the following config file:

```toml
[[local]]
basedir = "dt/dt-cli"
sources = ["*"]
target = "."
```

It will only sync `src/main.rs` to the configured target directory (in this
case, the target directory is where `dt` is being executed).

### `sources`

> [required] array of strings

Paths (relative to `basedir`) to the items to be synced.  Allow globbing
patterns

### `target`

> [required] string

The path of the parent dir of the final synced items.

#### Example

```toml
source = ["/source/file"]
target = "/tar/get"
```

will sync "/source/file" to "/tar/get/file" (creating non-existing directories
along the way), while

```toml
source = ["/source/dir"]
target = "/tar/get/dir"
```

will sync "source/dir" to "/tar/get/dir/dir" (creating non-existing
directories along the way).  If [`method`](#method-1) is set to `Symlink`,
non-existing directories in the staging directory are also created along the
way.

### `ignored`

> [todo]

:::danger Panics
Adding this to config file causes current version of `dt-cli` to panic.
:::

### `method`

> [optional] `Copy`|`Symlink`

Syncing method, overrides the [global `method`](#method) key.

### `allow_overwrite`

> [optional] bool

Whether to allow overwriting existing files, overrides the [global
`allow_overwrite`](#allow-overwrite) key.

:::warning Dead symlinks
Dead symlinks are treated as non-existant, and are always overwrited
(regardless of this option).
:::

### `hostname_sep`

> [optional] string

Separator for per-host settings, overries the [global
`hostname_sep`](#hostname-sep) key.

An additional item with `${hostname_sep}$(hostname)` appended to the original
item name will be checked first, before looking for the original item.  If the
appended item is found, use this item instead of the configured one.

Also ignores items that are meant for other hosts by checking if the string
after `hostname_sep` matches current machine's hostname.

#### Example

When the following directory structure exists:

```plain
~/.ssh/
├── authorized_keys
├── authorized_keys@@sherlock
├── authorized_keys@@watson
├── config
├── config@sherlock
└── config@watson
```

On a machine with hostname set to `watson`, the below configuration
(extraneous keys are omitted here)

```toml [[local]]
...
hostname_sep = "@@"

basedir = "~/.ssh"
sources = ["config"]
target = "/tmp/sshconfig"
...
```

will result in the below target (`/tmp/sshconfig`):

```plain
/tmp/sshconfig/
└── config
```

Where `/tmp/sshconfig/config` mirrors the content of `~/.ssh/config@watson`.

### rename

> [optional] array of 2-tuples

Renaming rules, appends to [global.rename](#rename).

Rules defined here will be appended to globally defined renaming.  For full
explanation and usage, see [Filename
Manipulation](/features/03-filename-manipulating).
