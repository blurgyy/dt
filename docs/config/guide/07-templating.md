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

## Writing Templates

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

## Skipping Rendering

By default, `dt-cli` treats all source files as templates to be rendered.
Sometimes we want to skip rendering, for example when a source file is huge,
or when a source file contains strings that conflicts with the Handlebars
syntax, or whatever.  To skip rendering for a group, use the [`renderable =
false`] option:

```toml{16}
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
renderable = false
```

After another run of `dt-cli`, `~/.config/gtk-3.0/settings.ini` will have
contents identical to the original (unrendered) template:

```ini
# Contents of ~/.config/gtk-3.0/settings.ini
[Settings]
gtk-cursor-theme-size={{{ gui.cursor-size }}}
gtk-font-name=system-ui {{{ gui.font-size }}}
```

Finally, template rendering can be disabled globally by adding the
[`renderable = false`] line to the `[global]` section:

```toml{3}
[global]
...
renderable = false
```

## Advanced Syntaxes

The [built-in helpers] of the [Handlebars crate] are understood in dt-cli's
templates.  Please refer to the [Handlebars crate]'s page for guides on those
basic control flow syntaxes like looping and conditioning.

In addition, `dt-cli` provides some [more helpers] to further boost the power
of templating.  The following table lists their names and descriptions,
**click on their names in the table to see their respective usages in
detail**.

|helper|description|
|:---:|:---|
|[`get_mine`]|Retrieves the value for current host from a map, returns a default value when current host is not recorded in the map|
|[`if_host`]|Tests if current machine’s hostname matches a set of given string(s)|
|[`if_os`]|Conditions on values parsed from target machine’s `/etc/os-release` file|
|[`if_uid`]|Tests if current user’s effective uid matches a set of given integer(s)|
|[`if_user`]|Tests if current user’s username matches a set of given string(s)|
|[`unless_host`], [`unless_os`], [`unless_uid`], [`unless_user`]|Negated versions of [`if_host`], [`if_os`], [`if_uid`], [`if_user`]|

:::info NOTE
Above table might get out-dated, check out
<https://docs.rs/dt-core/latest/dt_core/registry/helpers/index.html> for a
list of supported helpers <sub>(in addition to those already supported by the
[Handlebars crate])</sub> and their usages.
:::

<!-- Writing Templates -->
[Handlebars]: https://handlebarsjs.com/guide/
[`renderable = false`]: /config/key-references#renderable-1
[Handlebars crate]: https://docs.rs/handlebars/latest/handlebars/
[built-in helpers]: https://docs.rs/handlebars/latest/handlebars/#built-in-helpers
[more helpers]: <https://docs.rs/dt-core/latest/dt_core/registry/helpers/index.html>
[`get_mine`]: <https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.get_mine.html>
[`if_host`]: <https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.if_host.html>
[`if_os`]: <https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.if_os.html>
[`if_uid`]: <https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.if_uid.html>
[`if_user`]: <https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.if_user.html>
[`unless_host`]: <https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.unless_host.html>
[`unless_os`]: <https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.unless_os.html>
[`unless_uid`]: <https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.unless_uid.html>
[`unless_user`]: <https://docs.rs/dt-core/latest/dt_core/registry/helpers/fn.unless_user.html>
