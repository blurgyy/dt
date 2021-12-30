# Filename Manipulating <sub>[[**Examples**]]</sub>

Being able to have different names for the source items and their destination
items arises as a vital need in some use cases.  `dt-cli` offers this utility
via a convenient and versatile way, with the power of regular expressions.

## Background

This is not new.  [GNU Stow] has an option `--dotfiles`, which explains this
in [man:stow(8)]:

> `--dotfiles`:
>
> Enable special handling for "dotfiles" (files or folders whose name begins
> with a period) in the package directory. If this option is enabled, Stow
> will add a preprocessing step for each file or folder whose name begins
> with "dot-", and replace the "dot-" prefix in the name by a period (.).
> This is useful when Stow is used to manage collections of dotfiles, to
> avoid having a package directory full of hidden files.

`dt-cli`'s filename manipulating capability functions in a similar way, but in
a much more powerful way: you can specify arbitrary renaming rules, thanks to
the flexibility of regular expressions; you can also specify arbitrary number
of renaming rules, multiple defined rules will apply to your items
one-after-another.

## `rename`

Renaming rules are defined in the [`rename`] array.  A renaming rule is
represented by a 2-tuple in the config file, where the first element is a
regular expression _pattern_ and the second element is a _substitution rule_.
To achieve an identical behaviour as the `--dotfiles` option with [GNU Stow],
a renaming rule can be set in `global` section of the config file as:

```toml
[global]
rename = [
  [
    "^dot-",  # "pattern"
    ".",      # "substitution rule"
  ],
  # Multiple renaming rules are applied sequentially, with the previous rule's
  # output being the input of the current rule.
]
```

Since `dt-cli`'s renaming capability is powered by regular expressions, it also
supports regular expression features like [capturing groups], which allows
infinitely flexible filename manipulation.  More examples can be found in the
[hands-on guide].

[**Examples**]: /config/guide/06-filename-manipulating

[GNU Stow]: https://www.gnu.org/software/stow/
[man:stow(8)]: https://man.archlinux.org/man/community/stow/stow.8.en
[`rename`]: /config/key-references#rename

[capturing groups]: https://www.regular-expressions.info/refcapture.html
[hands-on guide]: /config/guide/06-filename-manipulating
