# Basics

Configurations are composed with **groups**.  A **group** represents a
collection of files that share some properties.  A `local` group is added to
the configuration file by adding a `[[local]]` section.

Assuming your configuration files for `Neovim` reside in `~/dt/nvim`, and all
match the globbing pattern `*init.vim`, a minimal working example can then be
configured as:

```toml
[[local]]
name = "Neovim"
basedir = "~/dt/nvim"
sources = ["*init.vim"]
target = "~/.config/nvim"
```

This content causes `dt-cli` to perform the following steps:

1. Create a "staging" directory at `$XDG_DATA_HOME/dt/staging` if the
   `XDG_DATA_HOME` environment variable is set, otherwise at
   `~/.local/share/dt/staging`;
2. Create the group's staging directory under the staging directory:
  `[...]/dt/staging/Neovim`;
3. Find all items (recursively if an item is a directory) that matches glob
   `~/dt/nvim/*init.vim` and store them back in the `sources` array;
4. For each item in the updated `sources` array, first copy it to the group's
   staging directory (`~/.local/share/dt/staging/Neovim`), then symlink it to
   the target directory (`~/.config/nvim`), abort if a target file already
   exists.

:::tip
Details of above steps are explained in the [Syncing
Methods](03-syncing-methods) section.
:::

:::warning
Aborting on existing target files is probably not what you want.  Read on for
a better solution!
:::
