# The Hostname Suffix

A **hostname suffix** comprises of a **hostname separator** and a
**hostname**:

- Hostname separator: Defined in configuration file as `hostname_sep`,
  [globally](/config/key-references#hostname-sep) or
  [per-group](/config/key-references#hostname-sep-1).
- Hostname: Current machine's hostname.

:::warning Multiple Occurances
To eliminate ambiguity, the hostname separator should appear at most once
in any of the source items.  Multiple occurances of the hostname separator
will cause `dt-cli` to panic.
:::

The default value (when not configured) for `hostname_sep` is `@@`.  If a
directory is marked as host-specific, all of its children will only be synced
when the directory is for current machine.
