# Host-specific Configuration

When you want to maintain multiple configurations for different machines, you
will have to deal with [host-specific syncing](/features/01-host-specific).
This section describes how to use this feature of `dt-cli` properly.

## Separator

First, you have to define a `hostname_sep` in your config file (or not, the
default value `@@` has a good chance at fitting your need),
[globally](/config/key-references#hostname-sep) or
[per-group](/config/key-references#hostname-sep-1), for example, you want your
hostname separator to be `QwQ` by default:

```toml
[global]
hostname_sep = "QwQ"
```

Or for a group only:

```toml
[[local]]
hostname_sep = "QwQ"
```

## Source Items

Knowing what your `hostname_sep` is, you can now specify your source items.

`dt-cli` automatically deals with the logic for host-specific syncing,
thus you **should not** contain a [hostname
suffix](/features/01-host-specific#hostname-suffix) when specifying your
sources.

### `base`

For example, you want to sync some user-scope systemd services on your
machines:

```toml
[[local]]
name = "SystemD-services"
base = "~/dt/systemd/user"
sources = ["*.service"]
target = "~/.config/systemd/user"

hostname_sep = "@@"
```

Then, on one of your machines, whose hostname is `elbert`, for example, the
above `base` will be automatically expanded to
`~/dt/systemd/user@@elbert` first, if the expanded `base` exists, `dt-cli`
will uses the expanded version; If the expanded `base` does not exist,
`dt-cli` will sync the original `base` when it exists.

### `sources`

Another real-world example is when you are using the same terminal emulator on
multiple machines, your workstation has a 8K ultra monitor, while your laptop
at home only has a monitor sized 14 inches.  You will not want to have the
same font sizes on the two machines.

What you could do is to separately maintain two versions of config files for
the terminal emulator.  When your configs are maintained under the `~/dt`
directory, and you are using Alacritty (for example):

```plain
~/dt/
├── alacritty/
│   ├── alacritty.yml@@laptop
│   └── alacritty.yml@@workstation
├── nvim/
│   ├── init.vim
│   └── ...
└── ...
```

You want to sync all stuff under the directory `~/dt` to `~/.config`, you can
populate your config files safely with:

```toml
[[local]]
name = "All-my-configs-including-for-terminal-emulator"
base = "~/dt"
sources = [
  "*",
  ".[!.]*",
  "..?*",
]
target = "~/.config"
```

:::warning
`dt-cli` will panic (**not a bug**) if you use globbing patterns like `.*` or
`/path/to/something/.*`, because `.*` also expands to the parent directory,
which is almost never what you want.

The globbing patterns in the above `sources` array is the recommended way to
glob all items under a given `base`.
:::

Note that we did not specifically reference the `alacritty` directory anywhere
in the above config, because `dt-cli` will recursively expand directories
**and automatically handle host-specific items in the expanded paths**.  You
can also specify a source only, like below:

```toml
[[local]]
name = "Alacritty"
base = "~/.dt/alacritty"
sources = ["alacritty.yml"]
target = "~/.config/alacritty"
```

:::warning
Do **NOT** include the host-specific part in the `sources` array (like
`alacritty.yml@@laptop` or `alacritty.yml@@workstation`), see the [Error
Handling](/config/guide/99-error-handling#config-validating) section for more
details on this.
:::
