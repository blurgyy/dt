# Priority Resolving

## Background

:::tip
Here explains why such a feature is desirable, feel free to skip this
subsection.
:::

Since `dt-cli` syncs your dotfiles on a per-group basis, you don't want to run
through all of the groups when only a single file is modified in your dotfile
library.  For example, when you updated your shell init script, you might run
the following command:

```shell
$ dt-cli shell
 INFO  dt_core::syncing > Local group: [shell]
```

Nevertheless, sometimes you have to run a full sync, which involves all of
your defined groups in your config file.  It may look like this:

```shell
$ dt-cli
 INFO  dt_core::syncing > Local group: [gdb]
 INFO  dt_core::syncing > Local group: [ssh]
 INFO  dt_core::syncing > Local group: [gpg]
 INFO  dt_core::syncing > Local group: [systemd]
 INFO  dt_core::syncing > Local group: [dt]
 INFO  dt_core::syncing > Local group: [nvim]
 INFO  dt_core::syncing > Local group: [fontconfig]
 INFO  dt_core::syncing > Local group: [shell]
 INFO  dt_core::syncing > Local group: [gui]
 INFO  dt_core::syncing > Local group: [xdg_config_home]
 INFO  dt_core::syncing > Local group: [misc]
```

Some groups may contain overlapping source items, in the above example, group
`xdg_config_home` contains `fontconfig` and `dt`'s base directories.  It's
neither friendly nor efficient for `dt-cli` to sync the same item twice or
even more times: it's slow, and the final result depends on the order of the
definitions of the groups.

## Scope

`dt-cli` solves this problem by defining an extra attribute `scope` for each
group.

A group's [`scope`](/config/key-references#scope) decides the priority of it
being synced.  There are 3 predefined scopes, namely `Dropin`, `App` and
`General`.  The names are pretty much self-explanatory:

- `General` groups have the lowest priority.  They are typically meant for the
  parent directories of your dotfile library.
- `Dropin` groups have the highest priority.  They are typically meant for
  those items that come from external sources as drop-in replacements, such as
  files from a system directory that is managed by your system's package
  manager.
- `App` groups have medium priority.  As the name implies, it is meant for
  some specific application, for example, a group containing your config files
  for GUI applications, or a group containing your shell/editor
  preferences/init scripts, etc..

:::info NOTE
A `scope` key in a group's definition is optional.  When omitted, the default
value of `scope` is `General`.
:::

:::tip
Generally, a larger scope has a lower priority.
:::

:::warning
If a file is included in multiple groups that have the same `scope`, it will
only be synced by the first group, later defined groups (with the same `scope`)
won't repeatedly sync the file.
:::

## Examples

### `Dropin`

On [Arch Linux](https://archlinux.org), package
[`fontconfig`](https://archlinux.org/packages/extra/x86_64/fontconfig/)
provides a file `/usr/share/fontconfig/conf.avail/10-sub-pixel-rgb.conf`,
which [works for most monitors](http://www.lagom.nl/lcd-test/subpixel.php).  A
drop-in group can be defined as:

```toml
[[local]]
scope = "Dropin"
name = "fontconfig-system"
basedir = "/usr/share/fontconfig/conf.avail/"
sources = [
  # Pixel Alignment.  Test monitor's subpixel layout at
  # <http://www.lagom.nl/lcd-test/subpixel.php>, reference:
  # <https://wiki.archlinux.org/title/Font_configuration#Pixel_alignment>
  "10-sub-pixel-rgb.conf",
  # Enable lcdfilter.  Reference:
  # <https://forum.endeavouros.com/t/faq-bad-font-rendering-in-firefox-and-other-programs/13430/3>
  "11-lcdfilter-default.conf",
]
target = "~/.config/fontconfig/conf.d"
```

### `App`

For example, a group of GUI applications under the [wayland
protocol](https://wayland.freedesktop.org) could be defined as:

```toml
[[local]]
scope = "General"
name = "gui"
basedir = "/path/to/your/dotfiles/library/root"
sources = [
  ".gtkrc-2.0",
  ".local/share/icons",
  ".local/share/fcitx5",
  ".config/sway",
  ".config/swaylock",
  ".config/waybar",
  ".config/dunst",
  ".config/gtk-*.0",
]
target = "~"
```

### `General`

This scope is mostly used in the fallback groups, for example:

```toml
[[local]]
scope = "General"
name = "xdg_config_home"
basedir = "/path/to/your/dotfiles/library/root/.config"
sources = [
  "*",
]
target = "~/.config"
[[local]]
scope = "General"
name = "misc"
basedir = "/path/to/your/dotfiles/library/root"
sources = [
  ".[!.]*",
  "..?*",
]
target = "~"
```
