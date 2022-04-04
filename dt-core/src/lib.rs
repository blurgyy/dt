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

/// Helper utilites used internally (the [`Register`] trait and the register
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
[context]
group = true  # enable templating for the group below

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
            let test_host = "HAL9000";
            std::env::set_var("DT_TEST_HOSTNAME_OVERRIDE", test_host);
            std::fs::write(&template_path, r#"Hi, {{get_mine}}!"#)?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                format!("Hi, {}!", test_host),
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
            let test_host = "HAL9000";
            std::env::set_var("DT_TEST_HOSTNAME_OVERRIDE", test_host);
            std::fs::write(
                &template_path,
                format!(
                    r#"The name {} comes from _{{{{get_mine testing_group.origin "None"}}}}_"#,
                    test_host,
                ),
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                format!(
                    "The name {} comes from _2001: a Space Odyssey_",
                    test_host
                ),
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
[context]
group = true  # enable templating for the group below

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
            let test_host = "HAL9000";
            std::env::set_var("DT_TEST_HOSTNAME_OVERRIDE", test_host);
            std::fs::write(&template_path, r#"Hi, {{get_mine}}!"#)?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                format!("Hi, {}!", test_host),
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
            let test_host = "HAL9000";
            std::env::set_var("DT_TEST_HOSTNAME_OVERRIDE", test_host);
            std::fs::write(
                &template_path,
                format!(
                    r#"The name {} comes from _{{{{get_mine testing_group.origin "None"}}}}_"#,
                    test_host,
                ),
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                format!(
                    "The name {} comes from _2001: a Space Odyssey_",
                    test_host
                ),
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );
            Ok(())
        }
    }

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
[context]
user = true  # enable templating for the group below

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

            let test_username = "john";
            std::env::set_var("DT_TEST_USERNAME_OVERRIDE", test_username);
            std::fs::write(
                &template_path,
                format!(
                    r#"Hi, {{{{#if_user "{}"}}}}{}{{{{else}}}}random person{{{{/if_user}}}}!"#,
                    test_username,
                    test_username.to_uppercase(),
                ),
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                format!("Hi, {}!", test_username.to_uppercase()),
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let test_username = "watson";
            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.user]
name = "{}"

[[local]]
name = "user"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                test_username,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::env::set_var("DT_TEST_USERNAME_OVERRIDE", test_username);
            std::fs::write(
                &template_path,
                format!(
                    r#"Welcome back, {{{{#if_user user.name}}}}{}{{{{else}}}}random person{{{{/if_user}}}}!"#,
                    test_username.to_uppercase(),
                ),
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                format!("Welcome back, {}!", test_username.to_uppercase()),
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
[context]
user = true  # enable templating for the group below

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

            let test_username = "greg";
            std::env::set_var("DT_TEST_USERNAME_OVERRIDE", test_username);
            std::fs::write(
                &template_path,
                r#"Hi, {{#if_user "john,greg"}}you are either John or Greg{{else}}random person{{/if_user}}!"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Hi, you are either John or Greg!",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let test_username = "watson";
            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.user]
allowed_names = [
    "lestrade",
    "watson",
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
            std::env::set_var("DT_TEST_USERNAME_OVERRIDE", test_username);
            std::fs::write(
                &template_path,
                r#"Welcome back, {{#if_user user.allowed_names}}Lestrade (or Watson){{else}}random person{{/if_user}}!"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Welcome back, Lestrade (or Watson)!",
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
[context]
user = true  # enable templating for the group below

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

            let test_username = "john";
            std::env::set_var("DT_TEST_USERNAME_OVERRIDE", test_username);
            std::fs::write(
                &template_path,
                format!(
                    r#"Hi, {{{{#unless_user "{}"}}}}random person{{{{else}}}}{}{{{{/unless_user}}}}!"#,
                    test_username,
                    test_username.to_uppercase(),
                ),
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                format!("Hi, {}!", test_username.to_uppercase()),
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let test_username = "watson";
            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.user]
name = "{}"

[[local]]
name = "user"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                test_username,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::env::set_var("DT_TEST_USERNAME_OVERRIDE", test_username);
            std::fs::write(
                &template_path,
                format!(
                    r#"Welcome back, {{{{#unless_user user.name}}}}random person{{{{else}}}}{}{{{{/unless_user}}}}!"#,
                    test_username.to_uppercase(),
                ),
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                format!("Welcome back, {}!", test_username.to_uppercase()),
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
[context]
user = true  # enable templating for the group below

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

            let test_username = "greg";
            std::env::set_var("DT_TEST_USERNAME_OVERRIDE", test_username);
            std::fs::write(
                &template_path,
                r#"Hi, {{#unless_user "john,greg"}}random person{{else}}you are either John or Greg{{/unless_user}}!"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Hi, you are either John or Greg!",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let test_username = "watson";
            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.user]
allowed_names = [
    "lestrade",
    "watson",
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
            std::env::set_var("DT_TEST_USERNAME_OVERRIDE", test_username);
            std::fs::write(
                &template_path,
                r#"Welcome back, {{#unless_user user.allowed_names}}random person{{else}}Lestrade (or Watson){{/unless_user}}!"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Welcome back, Lestrade (or Watson)!",
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
[context]
uid = true  # enable templating for the group below

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

            let test_uid: u32 = 410;
            std::env::set_var("DT_TEST_UID_OVERRIDE", test_uid.to_string());
            std::fs::write(
                &template_path,
                format!(
                    r#"If your UID is a HTTP status code, {{{{#if_uid {}}}}}it means `Precondition Failed`{{{{else}}}}I have no idea what it means{{{{/if_uid}}}}!"#,
                    test_uid,
                ),
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "If your UID is a HTTP status code, it means `Precondition Failed`!",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let test_uid: u32 = 418;
            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.uid]
number = {}

[[local]]
name = "uid"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                test_uid,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::env::set_var("DT_TEST_UID_OVERRIDE", test_uid.to_string());
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

            let test_uid: u32 = 416;
            std::env::set_var("DT_TEST_UID_OVERRIDE", test_uid.to_string());
            std::fs::write(
                &template_path,
                r#"Hi, {{#if_uid "410,412,418"}}user#410/412/418{{else}}random person{{/if_uid}}!"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "Hi, random person!",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            // Match inverse block
            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.uid]
allowed_numbers = [410, 412, 418]

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
            let test_uid: u32 = 1000;
            std::env::set_var("DT_TEST_UID_OVERRIDE", test_uid.to_string());
            std::fs::write(
                &template_path,
                r#"You are {{#if_uid uid.allowed_numbers}}{{else}}not {{/if_uid}}user#410/412/418"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "You are not user#410/412/418",
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
[context]
uid = true  # enable templating for the group below

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

            let test_uid: u32 = 412;
            std::env::set_var("DT_TEST_UID_OVERRIDE", test_uid.to_string());
            std::fs::write(
                &template_path,
                r#"If your UID is a HTTP status code, {{#if_uid 412}}it means `Precondition Failed`{{else}}I have no idea what it means{{/if_uid}}!"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "If your UID is a HTTP status code, it means `Precondition Failed`!",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let test_uid: u32 = 418;
            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.uid]
number = {}

[[local]]
name = "uid"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                test_uid,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::env::set_var("DT_TEST_UID_OVERRIDE", test_uid.to_string());
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
[context]
uid = true  # enable templating for the group below

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

            let test_uid: u32 = 412;
            std::env::set_var("DT_TEST_UID_OVERRIDE", test_uid.to_string());
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
            let test_uid: u32 = 1000;
            std::env::set_var("DT_TEST_UID_OVERRIDE", test_uid.to_string());
            std::fs::write(
                &template_path,
                r#"You {{#unless_uid uid.allowed_numbers}}can't{{else}}might{{/unless_uid}} be a teapot"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "You can't be a teapot",
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
[context]
host = true  # enable templating for the group below

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

            let test_host = "c-3po";
            std::env::set_var("DT_TEST_HOSTNAME_OVERRIDE", test_host);
            std::fs::write(
                &template_path,
                r#"I have {{#if_host "c-3po"}}a bad{{else}}no{{/if_host}} feeling about this"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "I have a bad feeling about this",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let test_host = "eniac";
            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.host]
name = "{}"

[[local]]
name = "host"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                test_host,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::env::set_var("DT_TEST_HOSTNAME_OVERRIDE", test_host);
            std::fs::write(
                &template_path,
                r#"This is {{#if_host host.name}}an ancient one{{else}}a machine{{/if_host}}"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "This is an ancient one",
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
[context]
host = true  # enable templating for the group below

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

            let test_host = "c-3po";
            std::env::set_var("DT_TEST_HOSTNAME_OVERRIDE", test_host);
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

            let test_host = "r2d2";
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
            std::env::set_var("DT_TEST_HOSTNAME_OVERRIDE", test_host);
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
[context]
host = true  # enable templating for the group below

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

            let test_host = "c-3po";
            std::env::set_var("DT_TEST_HOSTNAME_OVERRIDE", test_host);
            std::fs::write(
                &template_path,
                r#"I have {{#unless_host "c-3po"}}no{{else}}a bad{{/unless_host}} feeling about this"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "I have a bad feeling about this",
                std::str::from_utf8(
                    &reg.get(&template_path.to_string_lossy())?
                )?,
            );

            let test_host = "eniac";
            let config = expand(DTConfig::from_str(&format!(
                r#"
[context.host]
name = "{}"

[[local]]
name = "host"
base = "{}"
target = "{}"
sources = ["{}"]
"#,
                test_host,
                base.display(),
                target.display(),
                src_name,
            ))?)?;
            std::env::set_var("DT_TEST_HOSTNAME_OVERRIDE", test_host);
            std::fs::write(
                &template_path,
                r#"This is {{#unless_host host.name}}a machine{{else}}an ancient one{{/unless_host}}"#,
            )?;
            let reg =
                Registry::default().register_helpers()?.load(&config)?;
            assert_eq!(
                "This is an ancient one",
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
[context]
host = true  # enable templating for the group below

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

            let test_host = "c-3po";
            std::env::set_var("DT_TEST_HOSTNAME_OVERRIDE", test_host);
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

            let test_host = "r2d2";
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
            std::env::set_var("DT_TEST_HOSTNAME_OVERRIDE", test_host);
            std::fs::write(
                &template_path,
                r#"I have {{#unless_host host.allowed_names}}a bad{{else}}beep boop bop{{/unless_host}} feeling about this"#,
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
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 17 2021, 21:32 [CST]
