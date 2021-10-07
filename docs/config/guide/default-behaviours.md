# Defining Default Behaviours

Note that when syncing our configuration files for `Neovim` in the [basic
config](/config/guide/), `dt-cli` _aborts_ on existing target files.  When
populating items to another machine, it's better to directly overwrite
(assuming you know what you are doing) the target file, so the [basic
config](/config/guide/) is suboptimal.  What we could do is to additionally **define
the default overwriting behaviours** with a `[global]` section in the
configuration:

```toml{1-4}
[global]
allow_overwrite = true


[[local]]
name = "Neovim"
basedir = "~/dt/nvim"
sources = ["*init.vim"]
target = "~/.config/nvim"
```

This time, with the added `allow_overwrite = true`, existence of target file
no longer aborts the syncing process.
