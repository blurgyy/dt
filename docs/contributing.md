# Contributing

Thank you for your interest in contributing! There are many ways to contribute
to [`dt`](https://github.com/blurgyy/dt).  You can start with examining
unchecked items in the
[roadmap](https://github.com/blurgyy/dt/blob/main/roadmap.md), or discuss some
features to be added to the roadmap.

When contributing to this repository, please first discuss the change you wish
to make via [issue](https://github.com/blurgyy/dt/issues),
[email](mailto:gy@blurgy.xyz), or any other method with the owners of this
repository before making a change.

Please note we have a [code of
conduct](https://github.com/blurgyy/dt/blob/main/CODE_OF_CONDUCT.md), please
follow it in all your interactions with the project.

## Making your changes

1. Fork your own copy of `dt` into your account and clone your forked
   repository to your development machine:
   ```shell
   $ git clone https://github.com/${your_account}/dt.git && cd dt
   ```
2. Install latest stable version of [Rust](https://github.com/rust-lang/rust),
   then make your commits.  Please follow the [conventional commit
   specification](https://www.conventionalcommits.org/) when writing your
   commit messages.
3. If features are added/updated, also add/update tests and documents and
   check if the tests are passed:
   ```shell
   $ cargo test
   ```
4. Resolve errors and warnings given by `cargo-clippy`:
   ```shell
   $ cargo clippy
   ```
5. Format your changes with `cargo-fmt`:
   ```shell
   $ cargo fmt -- --verbose
   ```
6. Submit a [pull request](https://github.com/blurgyy/dt/pulls) and discuss
   the possible changes with the owner/maintainer, update your pull request if
   necessary.

## License

By contributing, you agree that your contributions will be licenced under [MIT
OR Apache 2.0](https://github.com/blurgyy/dt/blob/main/LICENSE) license.
