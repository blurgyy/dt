# Basics

Configurations are composed with "groups".  A "local" group is added to the
configuration file by adding a `[[local]]` section.

Assuming your configuration files for `Neovim` reside in `~/dotfiles/nvim`,
and all match the globbing pattern `*init.vim`, a minimal working example can
then be configured as:

```toml
[[local]]
name = "Neovim"
basedir = "~/dotfiles/nvim"
sources = ["*init.vim"]
target = "~/.config/nvim"
```

This content causes `dt-cli` to perform the following steps:

1. Create a "staging" directory at `~/.cache/dt/staging` (which is the default
   staging location);
2. Create the group's staging directory at `~/.cache/dt/staging/Neovim`;
3. Find all items (recursively if an item is a directory) that matches glob
   `~/dotfiles/nvim/*init.vim` and store them in the `sources` array;
4. For each item in the `sources` array, first copy it to the group's staging
   directory (`~/.cache/dt/staging/Neovim`), then symlink it to the target
   directory (`~/.config/nvim`), abort if a target file already exists.

:::tip
Aborting on existing targets is almost **never** what you want.  Read on for a
better solution!
:::