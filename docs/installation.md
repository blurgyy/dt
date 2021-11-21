# Install

## AUR

`dt-cli` is in the [AUR](https://aur.archlinux.org/packages/dt-cli/), you can
install it with your favorite package manager:

```shell
$ paru -S dt-cli
```

## Alternative ways

Alternatively, you can:

- Download latest [release](https://github.com/blurgyy/dt/releases/latest)
  from GitHub
- Install from [crates.io](https://crates.io/crates/dt-cli/):
  
  ```shell
  $ cargo install dt-cli
  ```
  
- Build from source:
  
  ```shell
  $ git clone git@github.com:blurgyy/dt.git
  $ cd dt
  $ cargo test --release
  $ cargo install --path=dt-cli
  ```
