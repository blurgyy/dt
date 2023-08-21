# Filename Manipulating

To this point, our source items must be named as their destination requires,
most "dotfiles" begin their names with a literal dot (`.`), which is sometimes
annoying when managing them (like with [`git`]).  In the meantime, for items
that have names not feasible to be altered --- like items from your system
directory (e.g. a wallpaper image which is provided as part of a system-wide
installed package) --- according to the previous sections, there seem to be no
good way to have them tracked by `dt-cli`.

## Basics

To manipulate filename of items, `dt-cli` provides a configurable `rename`
option in the config file.  It is an array of renaming rules, each of them
constitutes of a _pattern_ and a _substitution rule_.  A simple renaming rule
to rename all items with a "dot-" prefix (like `dot-config`) to a "dotfile"
(like `.config`, in this case) can be specified in the `[global]` section as:

```toml
[global]
rename = [
  [
    "^dot-",  # "pattern", must be a valid regular expression
    ".",      # "substitution rule"
  ],
  # Multiple renaming rules are applied sequentially, with the previous rule's
  # output being the input of the current rule.
]
```

:::warning
Note that only the path components that appear after a group's `target` will
be altered by `dt-cli`.  For example, with the above renaming rule added to
your `[global]` section, a group with `target` set to `/some/path/dot-target`
will have all its items populated to the exact path `/some/path/dot-target`,
instead of `/some/path/.target`.
:::

## Per-group Rules

You might have guessed it: `rename` rules can also be specified on a per-group
basis.  The way this works is that `dt-cli` processes renaming rules in the
`rename` array one by one, first [`global.rename`], then [the group's `rename`]
if any.

For example, to revert the above renaming operation for a single group, you
can add a rule to this group:

```toml
[[local]]
name = "Group from which items must have a 'dot-' prefix after syncing"
# [...omitted...]
rename = [
  [
    "^.",     # "pattern", matches prefixing dot
    "dot-",   # "substitution rule", replace the matched string into "dot-"
  ],
]
```

Apparently, this rule "undo"s the renaming rule in the global section which we
previously defined.  Items that have names prefixed with `dot-` in this group
will first be renamed to have a `.` prefix, then the `.` prefix is renamed
back to `dot-`.

## Capturing Groups <sub>(as in regular expressions)</sub>

Since this functionality is powered by [the Rust crate `regex`], substitution
rules are supported to the extent which this crate allows.  A powerful
capability it provides is defining [capturing groups].  Capturing groups can
either be [named] or [numbered], which allows arbitrary manipulation to be
applied to any synced items.

### Example: <sub>Repeating the Extension of an Item</sub>

To illustrate how capturing groups work, we try to have the destination items
to repeat the extension name of their corresponding source items, via
capturing groups.  With numbered capturing group, this rule can be written as:

```toml
[global]
rename = [
  [
    "(.*)\\.(\\w+)",
    "${1}.${2}.${2}",
  ]
]
```

Or, with named capturing group:

```toml
[global]
rename = [
  [
    "(?P<actual_name>.*)\\.(?P<extension>\\w+)",
    "${actual_name}.${extension}.${extension}",
  ]
]
```

The outcomes of above two approaches are identical.

[`git`]: https://git-scm.com/doc
[`global.rename`]: /config/key-references#rename
[the group's `rename`]: /config/key-references#rename-1
[the Rust crate `regex`]: https://docs.rs/regex/latest/regex/

[capturing groups]: https://www.regular-expressions.info/refcapture.html
[named]: https://www.regular-expressions.info/named.html
[numbered]: https://www.regular-expressions.info/brackets.html
