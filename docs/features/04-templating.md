# Templating<sub>[[**Examples**]]</sub>

## Background

As is always the case, there are quite a few applications that share a same
set of properties.  For example, we want to have uniform looks for Qt and
GTK applications.  Templating utility is developed under the **DRY**
(**D**on't **R**epeat **Y**ourself) principle, it allows to manage these
shared properties in one place: change once, apply everywhere.

## Syntax

### Configuring

To manage shared properties, add a section `[context]` to `dt-cli`'s config
file.  For example, to set a property named `cursor-size` for the `gui` group
to value `24`:

```toml
# ~/.config/dt/cli.toml
...
[context]
gui.cursor-size = 24
## Or, as TOML allows it:
#[context.gui]
#cursor-size = 24
...
```

See the [configuration guide] for detailed usages.

### Applying

`dt-cli` uses Rust's [Handlebars crate] to render templates.  Handlebars is
tested and widely used, according to its descriptions:

> Handlebars-rust is the template engine that renders the official Rust
> website rust-lang.org, its book.

For example, to apply a property named `cursor-size` to all source files under
the `gui` group:

```ini
...
gtk-cursor-theme-size={{{ gui.cursor-size }}}
...
```

With `context.gui.cursor-size` being set to `24` (as in [previous section]),
the above template (in a group with name `gui`) will be rendered as:

```ini
# ~/.config/gtk-3.0/settings.ini
...
gtk-cursor-theme-size=24
...
```

The [Handlebars crate] also allows syntaxes like looping and conditioning, the
[built-in helpers] are understood in `dt-cli`'s templates.  Please refer to
the [Handlebars crate]'s page for syntax guides.

<!-- TITLE -->
[**Examples**]: /config/guide/07-templating

<!-- Syntax.Configuring -->
[configuration guide]: /config/guide/07-templating

<!-- Syntax.Applying -->
[Handlebars crate]: https://docs.rs/handlebars/latest/handlebars/
[previous section]: #configuring
[built-in helpers]: https://docs.rs/handlebars/latest/handlebars/#built-in-helpers
