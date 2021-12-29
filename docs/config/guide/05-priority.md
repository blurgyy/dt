# Scopes

A group's [`scope`](/config/key-references#scope) decides the priority of its
items.  When multiple groups contain a same item, only the group with the
highest priority will do sync that specific item.  This machanism minimizes
total number of filesystem I/O operations, which makes `dt-cli` to appear
faster, and achieves finer control over what to sync with `dt-cli` without
having to picking out each application's config files from your dotfile
library.

:::tip
This feature is meant to be used with `dt-cli`'s [command-line
argument](/#usage), see the [Background](/features/scope) subsection of this
feature's introduction for more details.
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
