# Introduction

> With more _servers_ there must also come more _configuration files_.

When you own more than one machine, you will eventually face the problem that
one configuration file that works perfectly on one machine does not work well
on another, be it due to their monitor sizes, network conditions,
architectures, etc..

---

What you want is to populate different configuration files for different
machines.  To allow multiple items with the same name name, `dt-cli` checks
for an additional **hostname suffix** for every source item, and ignores
those items which are meant for other hosts.  `dt-cli` works with them quite
intuitively.  In short, it ignores items for other machines, and syncs items
for current machine whenever possible.

:::info
Specifically, with **hostname suffix** defined, source items can be (virtually)
categorized into 3 types:

- `Current`: Items that are host-specific, and are for current machine only;
- `General`: Items that are for all machines;
- `Other`: Items that are host-specific, but are for some other machine.

`dt-cli` will sync items that are of type `Current` if they exist;
if no `Current` item exists, `dt-cli` finds `General` items and sync them.
Items of type `Other` are ignored for current machine.
:::
