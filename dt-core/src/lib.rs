//! This is a helper library, containing shared utilities used by [`DT`].
//!
//! [`DT`]: https://github.com/blurgyy/dt

/// Definitions for configuration structures and rules.
#[deny(missing_docs)]
pub mod config;

/// Definitions for errors
#[deny(missing_docs)]
pub mod error;

/// Operations and abstractions for items.
#[deny(missing_docs)]
pub mod item;

/// Helper utilities used internally (the [`Register`] trait and the register
/// type [`Registry`] with cache for templates and rendered contents) and
/// exposed for templating uses (additional [built-in helpers]).
///
/// [`Register`]: registry::Register
/// [`Registry`]: registry::Registry
/// [built-in helpers]: registry::helpers
#[deny(missing_docs)]
pub mod registry;

/// Definitions for syncing behaviours.
#[deny(missing_docs)]
pub mod syncing;

/// Miscellaneous utilities.
#[deny(missing_docs)]
pub mod utils;

#[cfg(test)]
mod inline_helpers {
    mod get_mine {
        use std::str::FromStr;

        use crate::{
            config::DTConfig,
            registry::{Register, Registry},
            syncing::expand,
            utils::testing::{get_testroot, prepare_directory, prepare_file},
        };

        use color_eyre::Report;
        use pretty_assertions::assert_eq;

        #[test]
        fn no_param() -> Result<(), Report> {
            let base = prepare_directory(
                get_testroot("inline_helpers")
                    .join("get_mine")
                    .join("no_param"),
                0o755,
            )?;
            let src_name = "template";
            let template_path = prepare_file(base.join(src_name), 0o644)?;
            let target = prepare_directory(base.join("target"), 0o755)?;
            let config = expand(DTConfig::from_str(&format!(
                r#"
[[local]]
name = "group"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::fs::write(&template_path, r#"Hi, {{get_mine}}!"#)?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Hi, r2d2!",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );
            Ok(())
        }

        #[test]
        fn lookup() -> Result<(), Report> {
            let base = prepare_directory(
                get_testroot("inline_helpers")
                    .join("get_mine")
                    .join("lookup"),
                0o755,
            )?;
            let src_name = "template";
            let template_path = prepare_file(base.join(src_name), 0o644)?;
            let target = prepare_directory(base.join("target"), 0o755)?;
            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.testing_group]
origin.HAL9000 = "2001: a Space Odyssey"
origin.c-3po = "Star Wars"
origin.r2d2 = "Star Wars"

[[local]]
name = "testing_group"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::fs::write(
                &template_path,
                r#"The name r2d2 comes from _{{get_mine testing_group.origin "None"}}_"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "The name r2d2 comes from _Star Wars_",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );
            Ok(())
        }
    }
}

#[cfg(test)]
mod block_helpers {
    mod user {
        use std::str::FromStr;

        use crate::{
            config::DTConfig,
            registry::{Register, Registry},
            syncing::expand,
            utils::testing::{get_testroot, prepare_directory, prepare_file},
        };

        use color_eyre::Report;
        use pretty_assertions::assert_eq;

        #[test]
        fn if_user_exact() -> Result<(), Report> {
            let base = prepare_directory(
                get_testroot("block_helpers")
                    .join("user")
                    .join("if_user_exact"),
                0o755,
            )?;
            let src_name = "template";
            let template_path = prepare_file(base.join(src_name), 0o644)?;
            let target = prepare_directory(base.join("target"), 0o755)?;
            let config = expand(DTConfig::from_str(&format!(
                r#"
[[local]]
name = "user"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::fs::write(
                &template_path,
                r#"Hi, {{#if_user "luke"}}Luke Skywalker{{else}}random person{{/if_user}}!"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Hi, Luke Skywalker!",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.user]
name = "luke"

[[local]]
name = "user"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::fs::write(
                &template_path,
                r#"Welcome back, {{#if_user user.name}}Luke Skywalker{{else}}random person{{/if_user}}!"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Welcome back, Luke Skywalker!",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );
            Ok(())
        }

        #[test]
        fn if_user_any() -> Result<(), Report> {
            let base = prepare_directory(
                get_testroot("block_helpers")
                    .join("user")
                    .join("if_user_any"),
                0o755,
            )?;
            let src_name = "template";
            let template_path = prepare_file(base.join(src_name), 0o644)?;
            let target = prepare_directory(base.join("target"), 0o755)?;
            let config = expand(DTConfig::from_str(&format!(
                r#"
[[local]]
name = "user"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;

            std::fs::write(
                &template_path,
                r#"Hi, {{#if_user "luke,skywalker"}}Luke Skywalker{{else}}random person{{/if_user}}!"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Hi, Luke Skywalker!",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.user]
allowed_names = [
    "skywalker",
    "luke",
]

[[local]]
name = "user"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::fs::write(
                &template_path,
                "Welcome back, {{#if_user user.allowed_names}}Luke Skywalker{{else}}random person{{/if_user}}!",
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Welcome back, Luke Skywalker!",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );
            Ok(())
        }

        #[test]
        fn unless_user_exact() -> Result<(), Report> {
            let base = prepare_directory(
                get_testroot("block_helpers")
                    .join("user")
                    .join("unless_user_exact"),
                0o755,
            )?;
            let src_name = "template";
            let template_path = prepare_file(base.join(src_name), 0o644)?;
            let target = prepare_directory(base.join("target"), 0o755)?;
            let config = expand(DTConfig::from_str(&format!(
                r#"
[[local]]
name = "user"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::fs::write(
                &template_path,
                r#"Hi, {{#unless_user "luke"}}random person{{else}}Luke Skywalker{{/unless_user}}!"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Hi, Luke Skywalker!",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.user]
name = "luke"

[[local]]
name = "user"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::fs::write(
                &template_path,
                "Welcome back, {{#unless_user user.name}}random person{{else}}Luke Skywalker{{/unless_user}}!",
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Welcome back, Luke Skywalker!",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );
            Ok(())
        }

        #[test]
        fn unless_user_any() -> Result<(), Report> {
            let base = prepare_directory(
                get_testroot("block_helpers")
                    .join("user")
                    .join("unless_user_any"),
                0o755,
            )?;
            let src_name = "template";
            let template_path = prepare_file(base.join(src_name), 0o644)?;
            let target = prepare_directory(base.join("target"), 0o755)?;
            let config = expand(DTConfig::from_str(&format!(
                r#"
[[local]]
name = "user"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;

            std::fs::write(
                &template_path,
                r#"Hi, {{#unless_user "luke,skywalker"}}random person{{else}}Luke Skywalker{{/unless_user}}!"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Hi, Luke Skywalker!",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.user]
allowed_names = [
    "luke",
    "skywalker"
]

[[local]]
name = "user"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::fs::write(
                &template_path,
                r#"Welcome back, {{#unless_user user.allowed_names}}random person{{else}}Luke Skywalker{{/unless_user}}!"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Welcome back, Luke Skywalker!",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );
            Ok(())
        }
    }

    mod uid {
        use std::str::FromStr;

        use crate::{
            config::DTConfig,
            registry::{Register, Registry},
            syncing::expand,
            utils::testing::{get_testroot, prepare_directory, prepare_file},
        };

        use color_eyre::Report;
        use pretty_assertions::assert_eq;

        #[test]
        fn if_uid_exact() -> Result<(), Report> {
            let base = prepare_directory(
                get_testroot("block_helpers")
                    .join("uid")
                    .join("if_uid_exact"),
                0o755,
            )?;
            let src_name = "template";
            let template_path = prepare_file(base.join(src_name), 0o644)?;
            let target = prepare_directory(base.join("target"), 0o755)?;
            let config = expand(DTConfig::from_str(&format!(
                r#"
[[local]]
name = "uid"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;

            std::fs::write(
                &template_path,
                "Hi, {{#if_uid 418}}teapot{{else}}random user{{/if_uid}}",
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Hi, teapot",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.uid]
number = 418

[[local]]
name = "uid"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::fs::write(
                &template_path,
                r#"Hello {{#if_uid uid.number}}teapot{{else}}there{{/if_uid}}"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Hello teapot",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );
            Ok(())
        }

        #[test]
        fn if_uid_any() -> Result<(), Report> {
            let base = prepare_directory(
                get_testroot("block_helpers").join("uid").join("if_uid_any"),
                0o755,
            )?;
            let src_name = "template";
            let template_path = prepare_file(base.join(src_name), 0o644)?;
            let target = prepare_directory(base.join("target"), 0o755)?;
            let config = expand(DTConfig::from_str(&format!(
                r#"
[context]
uid = true

[[local]]
name = "uid"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;

            std::fs::write(
                &template_path,
                r#"Hi, {{#if_uid "410,412,418"}}user#410/412/418{{else}}random person{{/if_uid}}!"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Hi, user#410/412/418!",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            // Match inverse block
            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.uid]
allowed_numbers = [1000, 1001]

[[local]]
name = "uid"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::fs::write(
                &template_path,
                r#"You are {{#if_uid uid.allowed_numbers}}{{else}}not {{/if_uid}}user#1000/1001"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "You are not user#1000/1001",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );
            Ok(())
        }

        #[test]
        fn unless_uid_exact() -> Result<(), Report> {
            let base = prepare_directory(
                get_testroot("block_helpers")
                    .join("uid")
                    .join("unless_uid_exact"),
                0o755,
            )?;
            let src_name = "template";
            let template_path = prepare_file(base.join(src_name), 0o644)?;
            let target = prepare_directory(base.join("target"), 0o755)?;
            let config = expand(DTConfig::from_str(&format!(
                r#"
[[local]]
name = "uid"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;

            std::fs::write(
                &template_path,
                "Hi, {{#unless_uid 418}}random user{{else}}teapot{{/unless_uid}}",
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Hi, teapot",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.uid]
number = 418

[[local]]
name = "uid"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::fs::write(
                &template_path,
                r#"Hello {{#if_uid uid.number}}teapot{{else}}there{{/if_uid}}"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Hello teapot",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );
            Ok(())
        }

        #[test]
        fn unless_uid_any() -> Result<(), Report> {
            let base = prepare_directory(
                get_testroot("block_helpers")
                    .join("uid")
                    .join("unless_uid_any"),
                0o755,
            )?;
            let src_name = "template";
            let template_path = prepare_file(base.join(src_name), 0o644)?;
            let target = prepare_directory(base.join("target"), 0o755)?;
            let config = expand(DTConfig::from_str(&format!(
                r#"
[[local]]
name = "uid"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;

            std::fs::write(
                &template_path,
                r#"You {{#unless_uid "410,412,418"}}can't{{else}}might{{/unless_uid}} be a teapot"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "You might be a teapot",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.uid]
allowed_numbers = [410, 418, 412, 416]

[[local]]
name = "uid"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::fs::write(
                &template_path,
                r#"You {{#unless_uid uid.allowed_numbers}}can't{{else}}might{{/unless_uid}} be a teapot"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "You might be a teapot",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );
            Ok(())
        }
    }

    mod host {
        use std::str::FromStr;

        use crate::{
            config::DTConfig,
            registry::{Register, Registry},
            syncing::expand,
            utils::testing::{get_testroot, prepare_directory, prepare_file},
        };

        use color_eyre::Report;
        use pretty_assertions::assert_eq;

        #[test]
        fn if_host_exact() -> Result<(), Report> {
            let base = prepare_directory(
                get_testroot("block_helpers")
                    .join("host")
                    .join("if_host_exact"),
                0o755,
            )?;
            let src_name = "template";
            let template_path = prepare_file(base.join(src_name), 0o644)?;
            let target = prepare_directory(base.join("target"), 0o755)?;
            let config = expand(DTConfig::from_str(&format!(
                r#"
[[local]]
name = "host"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;

            std::fs::write(
                &template_path,
                r#"I have {{#if_host "c-3po"}}a bad{{else}}beep boop bop{{/if_host}} feeling about this"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "I have beep boop bop feeling about this",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.host]
name = "r2d2"

[[local]]
name = "host"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::fs::write(
                &template_path,
                r#"This is a {{#if_host host.name}}blue-white{{else}}golden{{/if_host}} one"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "This is a blue-white one",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );
            Ok(())
        }

        #[test]
        fn if_host_any() -> Result<(), Report> {
            let base = prepare_directory(
                get_testroot("block_helpers")
                    .join("host")
                    .join("if_host_any"),
                0o755,
            )?;
            let src_name = "template";
            let template_path = prepare_file(base.join(src_name), 0o644)?;
            let target = prepare_directory(base.join("target"), 0o755)?;
            let config = expand(DTConfig::from_str(&format!(
                r#"
[[local]]
name = "host"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;

            std::fs::write(
                &template_path,
                r#"I {{#if_host "c-3po,r2d2"}}{{else}}don't {{/if_host}}know Luke"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "I know Luke",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.host]
allowed_names = [
    "r2d2",
    "bb8",
]

[[local]]
name = "host"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::fs::write(
                &template_path,
                r#"I have {{#if_host host.allowed_names}}beep boop bop{{else}}a bad{{/if_host}} feeling about this"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "I have beep boop bop feeling about this",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );
            Ok(())
        }

        #[test]
        fn unless_host_exact() -> Result<(), Report> {
            let base = prepare_directory(
                get_testroot("block_helpers")
                    .join("host")
                    .join("unless_host_exact"),
                0o755,
            )?;
            let src_name = "template";
            let template_path = prepare_file(base.join(src_name), 0o644)?;
            let target = prepare_directory(base.join("target"), 0o755)?;
            let config = expand(DTConfig::from_str(&format!(
                r#"
[[local]]
name = "host"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;

            std::fs::write(
                &template_path,
                r#"I have {{#unless_host "c-3po"}}beep boop bop{{else}}a bad{{/unless_host}} feeling about this"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "I have beep boop bop feeling about this",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.host]
name = "r2d2"

[[local]]
name = "host"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::fs::write(
                &template_path,
                r#"This is a {{#unless_host host.name}}golden{{else}}blue-white{{/unless_host}} one"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "This is a blue-white one",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );
            Ok(())
        }

        #[test]
        fn unless_host_any() -> Result<(), Report> {
            let base = prepare_directory(
                get_testroot("block_helpers")
                    .join("host")
                    .join("unless_host_any"),
                0o755,
            )?;
            let src_name = "template";
            let template_path = prepare_file(base.join(src_name), 0o644)?;
            let target = prepare_directory(base.join("target"), 0o755)?;
            let config = expand(DTConfig::from_str(&format!(
                r#"
[[local]]
name = "host"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;

            std::fs::write(
                &template_path,
                r#"I {{#unless_host "c-3po,r2d2"}}don't {{/unless_host}}know Luke"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "I know Luke",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.host]
allowed_names = [
    "r2d2",
    "bb8",
]

[[local]]
name = "host"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::fs::write(
                &template_path,
                r#"I have {{#unless_host host.allowed_names}}a bad{{else}}beep boop bop{{/unless_host}} feeling about this"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            println!("{:?}", crate::utils::testing::gethostname());
            assert_eq!(
                "I have beep boop bop feeling about this",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );
            Ok(())
        }
    }

    mod os {
        use std::str::FromStr;

        use crate::{
            config::DTConfig,
            registry::{Register, Registry},
            syncing::expand,
            utils::testing::{get_testroot, prepare_directory, prepare_file},
        };

        use color_eyre::Report;
        use pretty_assertions::assert_eq;

        #[test]
        fn if_os_exact() -> Result<(), Report> {
            let base = prepare_directory(
                get_testroot("block_helpers").join("os").join("if_os_exact"),
                0o755,
            )?;
            let src_name = "template";
            let template_path = prepare_file(base.join(src_name), 0o644)?;
            let target = prepare_directory(base.join("target"), 0o755)?;
            let config = expand(DTConfig::from_str(&format!(
                r#"
[[local]]
name = "os"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;

            std::fs::write(
                &template_path,
                r#"{{#if_os "id" "dt"}}It works{{else}}Nope it's not working{{/if_os}}"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "It works",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.os]
version = "latest"

[[local]]
name = "host"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::fs::write(
                &template_path,
                r#"{{#if_os "version" os.version}}It works{{else}}Not working{{/if_os}}"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "It works",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );
            Ok(())
        }

        #[test]
        fn if_os_any() -> Result<(), Report> {
            let base = prepare_directory(
                get_testroot("block_helpers").join("os").join("if_os_any"),
                0o755,
            )?;
            let src_name = "template";
            let template_path = prepare_file(base.join(src_name), 0o644)?;
            let target = prepare_directory(base.join("target"), 0o755)?;
            let config = expand(DTConfig::from_str(&format!(
                r#"
[[local]]
name = "os"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;

            std::fs::write(
                &template_path,
                r#"{{#if_os "id" "dummy-version,dt"}}It works{{else}}Nope it's not working{{/if_os}}"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "It works",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.os]
version = ["0.99.99", "99.0.0"]

[[local]]
name = "host"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::fs::write(
                &template_path,
                r#"{{#if_os "version_id" os.version}}It works{{else}}Not working{{/if_os}}"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "It works",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );
            Ok(())
        }

        #[test]
        fn unless_os_exact() -> Result<(), Report> {
            let base = prepare_directory(
                get_testroot("block_helpers")
                    .join("os")
                    .join("unless_os_exact"),
                0o755,
            )?;
            let src_name = "template";
            let template_path = prepare_file(base.join(src_name), 0o644)?;
            let target = prepare_directory(base.join("target"), 0o755)?;
            let config = expand(DTConfig::from_str(&format!(
                r#"
[[local]]
name = "os"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;

            std::fs::write(
                &template_path,
                r##"{{#unless_os "build_id" "#somethingsomething"}}Nope it's not working properly{{else}}It's working{{/unless_os}}"##,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "It's working",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.os]
logo = "Buzz Lightyear"

[[local]]
name = "host"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::fs::write(
                &template_path,
                r#"{{#unless_os "logo" os.logo}}Broken{{else}}Up and running{{/unless_os}}"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Up and running",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );
            Ok(())
        }

        #[test]
        fn unless_os_any() -> Result<(), Report> {
            let base = prepare_directory(
                get_testroot("block_helpers")
                    .join("os")
                    .join("unless_os_any"),
                0o755,
            )?;
            let src_name = "template";
            let template_path = prepare_file(base.join(src_name), 0o644)?;
            let target = prepare_directory(base.join("target"), 0o755)?;
            let config = expand(DTConfig::from_str(&format!(
                r#"
[[local]]
name = "os"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;

            std::fs::write(
                &template_path,
                r##"{{#unless_os "home_url" "https://example.com/,https://github.com/blurgyy/dt/"}}Nope it's not working properly{{else}}It's working{{/unless_os}}"##,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "It's working",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.os.documentation]
url = ["https://dt.cli.rs/", "https://github.com/blurgyy/dt/wiki/"]

[[local]]
name = "host"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::fs::write(
                &template_path,
                r#"{{#unless_os "documentation_url" os.documentation.url}}xxxBroken{{else}}Up and running{{/unless_os}}"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Up and running",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );
            Ok(())
        }
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 17 2021, 21:32 [CST]
