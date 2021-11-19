# Groups

Syncing only one group is boring.  We can also add more groups.

Assuming your configuration files for `VSCode` lies at
`~/dt/VSCode/User/settings.json`, the group for syncing this file can be:

```toml
[[local]]
name = "VSCode"
basedir = "~/dt/VSCode"
sources = ["User/settings.json"]
target = "~/.config/Code - OSS/"
```

Appending this `VSCode` group to our previous config file, we obtain:

```toml{10-14}
[global]
allow_overwrite = true


[[local]]
name = "Neovim"
basedir = "~/dt/nvim"
sources = ["*init.vim"]
target = "~/.config/nvim"
[[local]]
name = "VSCode"
basedir = "~/dt/VSCode"
sources = ["User/settings.json"]
target = "~/.config/Code - OSS/"
```

After syncing with this config, the **target item** `~/.config/Code -
OSS/User/settins.json` will be a symlink of the **staged item**
`~/.cache/dt/staging/VSCode/User/settings.json`, where the **staged item**
mirrors the content of the **source item** `~/dt/VSCode/User/settings.json`.

## Overriding Default Behaviours

But what if we exceptionally want the `VSCode` group to **not** overwrite the
target file if it already exists?  No worries, here is the recipe of
overriding a default behaviour for the `VSCode` group:

```toml{6-7}
[[local]]
name = "VSCode"
basedir = "~/dt/VSCode"
sources = ["User/settings.json"]
target = "~/.config/Code - OSS/"

allow_overwrite = false
```

A group's behaviour will precedent the default behaviour if explicitly
specified.  Listing all overridable configs and their default values here:

- `allow_overwrite`: `false`
- `hostname_sep`: `"@@"`
- `method`: `"Symlink"`

References to those keys can be found at [Key
References](/config/key-references).

:::tip
So far we have not explained why does `dt-cli` sync files in such a (somewhat
complex) manner.  You might ask:

> So what is _staging_, in particular?

Read on for a comprehensive explanation for this decision!
:::
