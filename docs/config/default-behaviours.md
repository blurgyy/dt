# Defining default behaviours

Note that when syncing our configuration files for `Neovim` in the [basic
config](/config/), `dt-cli` _aborts_ on existing target files.  When
populating dotfiles to another machine, it's better to directly overwrite
(assuming you know what you are doing) the target file, so the basic config is
suboptimal.  What we could do is to additionally define the default
overwriting behaviours with a `[global]` section in the configuration:

```toml{1-4}
[global]
allow_overwrite = true


[[local]]
name = "Neovim"
basedir = "~/dotfiles/nvim"
sources = ["*init.vim"]
target = "~/.config/nvim"
```

This time, with the added `allow_overwrite = true`, existence of target file
no longer aborts the syncing process.
