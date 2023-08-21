# Templating

`dt-cli` allows its source files to be templates, the templates are rendered
with values defined in `dt-cli`'s config file.  Here is a simple example that
parameterizes several GUI-related properties, and render a template to its
destination with `dt-cli`.

:::info NOTE
Only templating a single file shows little benefit.  This is just a toy
example that demonstrates the basic usage of templating.  In real-world uses,
the `sources` array can include more template files, so that templating can
actually ease config file management.
:::

## Setting Values

In `dt-cli`'s config file, add another section with name `[context]`.  Here is
where the values are set.  We will define the following values:

```toml{5-7,10}
# Contents of ~/.config/dt/cli.toml
[global]
...

[context]
gui.font-size = 15
gui.cursor-size = 24

[[local]]
name = "gui"
base = "~/dt/gui"
sources = [
  "gtk-3.0/settings.ini",
]
target = "~/.config"
```

In this config example, we have two values under the `context.gui` section.

:::warning INFO
These two values will **only** be rendered for templates in a group named
`gui`.
:::

## Writing templates

Templates are understood by `dt-cli` with the [Handlebars] syntax.  We can
template the gtk settings file in the `gui` group (se above) as:

```ini
# Contents of ~/dt/gui/gtk-3.0/settings.ini
[Settings]
gtk-cursor-theme-size={{{ gui.cursor-size }}}
gtk-font-name=system-ui {{{ gui.font-size }}}
```

After this, running `dt-cli` and `~/.config/gtk-3.0/settings.ini` will have
our templated values:

```ini
# Contents of ~/.config/gtk-3.0/settings.ini
[Settings]
gtk-cursor-theme-size=24
gtk-font-name=system-ui 15
```

<!-- Writing Templates -->
[Handlebars]: https://handlebarsjs.com/guide/
