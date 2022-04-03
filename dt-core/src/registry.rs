use std::{collections::HashMap, rc::Rc};

use content_inspector::inspect;
use handlebars::Handlebars;
use serde::Serialize;

use crate::{
    config::DTConfig,
    error::{Error as AppError, Result},
};

#[allow(unused_variables)]
/// A registry should hold an environment of templates, and a cached storing
/// the rendered contents.
pub trait Register
where
    Self: Sized,
{
    /// Registers DT's [built-in helpers].
    ///
    /// [built-in helpers]: helpers
    fn register_helpers(self) -> Result<Self> {
        unimplemented!()
    }
    /// Load templates and render them into cached storage, items that are not
    /// templated (see [`is_templated`]) will not be registered into templates
    /// but directly stored into the rendered cache.
    ///
    /// [`is_templated`]: crate::config::Group::is_templated
    fn load(self, config: &DTConfig) -> Result<Self> {
        unimplemented!()
    }
    /// Returns the content of an item rendered with given rendering context.
    /// This does not modify the stored content.
    ///
    /// Rendering only happens if this item is considered as a plain text
    /// file.  If this item is considered as a binary file, it's original
    /// content is returned.  The content type is inspected via the
    /// [`content_inspector`] crate.  Although it can correctly determine if
    /// an item is binary or text mostly of the time, it is just a heuristic
    /// check and can fail in some cases, e.g. NUL byte in the first 1024
    /// bytes of a UTF-8-encoded text file, etc..  See [the crate's home page]
    /// for the full caveats.
    ///
    /// [`content_inspector`]: https://crates.io/crates/content_inspector
    /// [the crate's home page]: https://github.com/sharkdp/content_inspector
    fn render<S: Serialize>(
        &self,
        name: &str,
        ctx: &Rc<S>,
    ) -> Result<Vec<u8>> {
        unimplemented!()
    }
    /// Updates the stored content of an item with the new content rendered
    /// with given rendering context.
    fn update<S: Serialize>(
        &mut self,
        name: &str,
        ctx: &Rc<S>,
    ) -> Result<()> {
        unimplemented!()
    }
    /// Looks up the rendered content of an item with given name.
    fn get(&self, name: &str) -> Result<Vec<u8>> {
        unimplemented!()
    }
}

/// Registry with a cache for rendered item contents.
#[derive(Debug, Default)]
pub struct Registry<'reg> {
    /// The templates before rendering.
    pub env: Handlebars<'reg>,
    /// The rendered contents of items.
    pub content: HashMap<String, Vec<u8>>,
}

impl Register for Registry<'_> {
    fn register_helpers(self) -> Result<Self> {
        let mut render_env = self.env;

        render_env.register_helper("get_mine", Box::new(helpers::get_mine));
        render_env.register_helper("if_user", Box::new(helpers::if_user));
        render_env.register_helper("if_uid", Box::new(helpers::if_uid));
        render_env.register_helper("if_host", Box::new(helpers::if_host));

        Ok(Self {
            env: render_env,
            ..self
        })
    }

    fn load(self, config: &DTConfig) -> Result<Self> {
        let mut registry = self;
        for group in &config.local {
            for s in &group.sources {
                let name = s.to_string_lossy();
                if let Ok(content) = std::fs::read(s) {
                    if group.is_templated() {
                        if inspect(&content).is_text() {
                            registry.env.register_template_string(
                                &name,
                                std::str::from_utf8(&content)?,
                            )?;
                            registry.content.insert(
                                name.to_string(),
                                registry
                                    .env
                                    .render(&name, &config.context)?
                                    .into(),
                            );
                        } else {
                            log::trace!(
                                "'{}' will not be rendered because it has binary contents",
                                s.display(),
                            );
                            registry
                                .content
                                .insert(name.to_string(), content);
                        }
                    } else {
                        log::trace!(
                            "'{}' will not be rendered because it is not templated",
                            s.display(),
                        );
                        registry.content.insert(name.to_string(), content);
                    }
                }
            }
        }
        Ok(registry)
    }

    fn render<S: Serialize>(
        &self,
        name: &str,
        ctx: &Rc<S>,
    ) -> Result<Vec<u8>> {
        if self.env.get_template(name).is_some() {
            Ok(self.env.render(name, &**ctx)?.into())
        } else {
            match self.content.get(name) {
                Some(content) => Ok(content.to_owned()),
                None => Err(AppError::RenderingError(format!(
                    "The template specified by '{}' is not known",
                    name,
                ))),
            }
        }
    }

    fn update<S: Serialize>(
        &mut self,
        name: &str,
        ctx: &Rc<S>,
    ) -> Result<()> {
        self.content
            .insert(name.to_owned(), self.render(name, ctx)?);
        Ok(())
    }

    fn get(&self, name: &str) -> Result<Vec<u8>> {
        match self.content.get(name) {
            Some(content) => Ok(content.to_owned()),
            None => Err(AppError::TemplatingError(format!(
                "The template specified by '{}' is not known",
                name,
            ))),
        }
    }
}

// ===========================================================================

/// Additional built-in helpers
pub mod helpers {
    use gethostname::gethostname;
    use handlebars::{
        Context, Handlebars, Helper, HelperResult, JsonRender, Output,
        RenderContext, RenderError, Renderable,
    };
    use users::{get_current_uid, get_current_username};

    /// A templating helper that retrieves the value for current host from a
    /// map, returns a default value when current host is not recorded in the
    /// map.
    ///
    /// Usage:
    ///
    /// 1. `{{ get_mine }}`
    ///
    ///     Renders current machine's hostname.
    /// 2. `{{ get_mine <map> <default-value> }}`
    ///
    ///     Renders `<map>.$CURRENT_HOSTNAME`, falls back to `<default-value>`.
    pub fn get_mine(
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _rc: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let map = match h.param(0) {
            Some(map) => map.value(),
            None => {
                out.write(&gethostname().to_string_lossy())?;
                return Ok(());
            }
        };
        let default_content = match h.param(1) {
            Some(content) => content.value(),
            None => {
                return Err(RenderError::new(&format!(
                    r#"
Inline helper `{0}`:
    expected 0 or 2 arguments, 1 found

    Usage:
        1. {{{{ {0} }}}}
           Renders current machine's hostname

        2. {{{{ {0} <map> <default-value> }}}}
           Gets value of <map>.$CURRENT_HOSTNAME, falls back to <default-value>"#,
                    h.name(),
                )))
            }
        };

        let content =
            match map.get(gethostname().to_string_lossy().to_string()) {
                Some(content) => content.render(),
                None => default_content.render(),
            };

        out.write(&content)?;

        Ok(())
    }

    /// A templating helper that tests if current user's username matches a
    /// set of given string(s).
    ///
    /// Usage:
    ///
    /// 1. {{#if_user "!foo,!bar"}}..baz..{{/if_user}}
    ///
    ///    Renders `..baz..` only if current user's username is neither "foo"
    ///    nor "bar".
    /// 2. {{#if_user "foo"}}..baz..{{else}}..qux..{{/if_user}}
    ///
    ///    Renders `..baz..` only if current user's username is "foo", renders
    ///    `..qux..` only if current user's username is NOT "foo".
    pub fn if_user<'reg, 'rc>(
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        if h.params().len() > 1 {
            return Err(RenderError::new(&format!(
                r#"
Block helper `#{0}`:
    expected exactly 1 argument, {1} found

    Usage:
        1. {{{{#{0} "!foo,!bar"}}}}..baz..{{{{/{0}}}}}
           Renders `..baz..` only if current user's username is neither "foo"
           nor "bar"

        2. {{{{#{0} "foo"}}}}..baz..{{{{else}}}}..qux..{{{{/{0}}}}}
           Renders `..baz..` only if current user's username is "foo", renders
           `..qux..` only if current user's username is NOT "foo""#,
                h.name(),
                h.params().len(),
            )));
        }

        let username: String = match h.param(0) {
            Some(v) => {
                if v.value().is_array() {
                    v.value()
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|elem| elem.render())
                        .collect::<Vec<_>>()
                        .join(",")
                } else {
                    v.value().render()
                }
            }
            None => {
                return Err(RenderError::new(&format!(
                    r#"
Block helper `#{0}`:
    expected exactly 1 argument, 0 found

    Usage:
        1. {{{{#{0} "!foo,!bar"}}}}..baz..{{{{/{0}}}}}
           Renders `..baz..` only if current user's username is neither "foo"
           nor "bar"

        2. {{{{#{0} "foo"}}}}..baz..{{{{else}}}}..qux..{{{{/{0}}}}}
           Renders `..baz..` only if current user's username is "foo", renders
           `..qux..` only if current user's username is NOT "foo""#,
                    h.name(),
                )));
            }
        };

        let current_username: &str = &get_current_username()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let allowed_usernames: Vec<&str> = username
            .split(',')
            .filter(|u| !u.starts_with("!"))
            .collect();
        let disallowed_usernames: Vec<&str> = username
            .split(',')
            .filter_map(|u| {
                if u.starts_with("!") {
                    u.strip_prefix("!")
                } else {
                    None
                }
            })
            .collect();
        if allowed_usernames.len() > 0 && disallowed_usernames.len() > 0 {
            return Err(RenderError::new(format!(
                "you can only supply one of positve OR negated type of arguments in a single {}",
                h.name(),
            )));
        }
        if allowed_usernames.len() > 0 {
            if allowed_usernames.contains(&current_username) {
                log::debug!(
                    "Current username {} matches {}",
                    current_username,
                    username,
                );
                h.template().map(|t| t.render(r, ctx, rc, out));
            } else {
                log::debug!(
                    "Current username {} does not match {}",
                    current_username,
                    username,
                );
                h.inverse().map(|t| t.render(r, ctx, rc, out));
            }
        } else if disallowed_usernames.len() > 0 {
            if disallowed_usernames.contains(&current_username) {
                log::debug!(
                    "Current username {} does not match {}",
                    current_username,
                    username,
                );
                h.inverse().map(|t| t.render(r, ctx, rc, out));
            } else {
                log::debug!(
                    "Current username {} matches {}",
                    current_username,
                    username,
                );
                h.template().map(|t| t.render(r, ctx, rc, out));
            }
        } else {
            return Err(RenderError::new(format!(
                "no username(s) supplied for matching in helper {}",
                h.name(),
            )));
        }
        Ok(())
    }

    /// A templating helper that tests if current user's effective uid matches
    /// a set of given integer(s).
    ///
    /// Usage:
    ///
    /// 1. `{{#if_uid "!0"}}..foo..{{/if_uid}}`
    ///
    ///    Renders `..foo..` only if current user's effective uid is not `0`.
    /// 2. `{{#if_uid 0}}..foo..{{else}}..bar..{{/if_uid}}`
    ///
    ///    Renders `..foo..` only if current user's effective uid is `0`,
    ///    renders `..bar..` only if current user's effective uid is not `0`.
    /// 3. `{{#if_uid "1000,1001"}}..foo..{{/if_uid}}`
    ///
    ///    Renders `..foo..` only if current user's effective uid is either
    ///    `1000` or `1001`.
    pub fn if_uid<'reg, 'rc>(
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        if h.params().len() > 1 {
            return Err(RenderError::new(&format!(
                r#"
Block helper `#{0}`:
    expected exactly 1 argument, {1} found

    Usage:
        1. {{{{#{0} "!0"}}}}..foo..{{{{/{0}}}}}
           Renders `..foo..` only if current user's effective uid is not 0

        2. {{{{#{0} 0}}}}..foo..{{{{else}}}}..bar..{{{{/{0}}}}}
           Renders `..foo..` only if current user's effective uid is 0,
           renders `..bar..` if current user's effective uid is not 0

        3. {{{{#{0} "1000,1001"}}}}..foo..{{{{/{0}}}}}
           Renders `..foo..` only if current user's effective uid is either
           1000 or 1001"#,
                h.name(),
                h.params().len(),
            )));
        }

        let uid: String = match h.param(0) {
            Some(v) => {
                if v.value().is_array() {
                    v.value()
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|elem| elem.render())
                        .collect::<Vec<_>>()
                        .join(",")
                } else {
                    v.value().render()
                }
            }
            None => {
                return Err(RenderError::new(&format!(
                    r#"
Block helper `#{0}`:
    expected exactly 1 argument, 0 found

    Usage:
        1. {{{{#{0} "!0"}}}}..foo..{{{{/{0}}}}}
           Renders `..foo..` only if current user's effective uid is not 0

        2. {{{{#{0} 0}}}}..foo..{{{{else}}}}..bar..{{{{/{0}}}}}
           Renders `..foo..` only if current user's effective uid is 0,
           renders `..bar..` if current user's effective uid is not 0

        3. {{{{#{0} "1000,1001"}}}}..foo..{{{{/{0}}}}}
           Renders `..foo..` only if current user's effective uid is either
           1000 or 1001"#,
                    h.name(),
                )));
            }
        };

        let current_uid = get_current_uid();
        let allowed_uids: Vec<u32> = uid
            .split(',')
            .filter_map(|id| {
                if !id.starts_with("!") {
                    id.parse().ok()
                } else {
                    None
                }
            })
            .collect();
        let disallowed_uids: Vec<u32> = uid
            .split(',')
            .filter_map(|id| {
                if id.starts_with("!") {
                    id.strip_prefix("!").unwrap().parse().ok()
                } else {
                    None
                }
            })
            .collect();
        if allowed_uids.len() > 0 && disallowed_uids.len() > 0 {
            return Err(RenderError::new(format!(
                "you can only supply one of positve OR negated type of arguments in a single {}",
                h.name(),
            )));
        }
        if allowed_uids.len() > 0 {
            if allowed_uids.contains(&current_uid) {
                log::debug!(
                    "Current uid '{}' matches '{}'",
                    current_uid,
                    uid,
                );
                h.template().map(|t| t.render(r, ctx, rc, out));
            } else {
                log::debug!(
                    "Current uid '{}' does not match '{}'",
                    current_uid,
                    uid,
                );
                h.inverse().map(|t| t.render(r, ctx, rc, out));
            }
        } else if disallowed_uids.len() > 0 {
            if disallowed_uids.contains(&current_uid) {
                log::debug!(
                    "Current uid '{}' does not match '{}'",
                    current_uid,
                    uid,
                );
                h.inverse().map(|t| t.render(r, ctx, rc, out));
            } else {
                log::debug!(
                    "Current uid '{}' matches '{}'",
                    current_uid,
                    uid,
                );
                h.template().map(|t| t.render(r, ctx, rc, out));
            }
        } else {
            return Err(RenderError::new(format!(
                "no uid(s) supplied for matching in helper {}",
                h.name(),
            )));
        }
        Ok(())
    }

    /// A templating helper that tests if current machine's hostname matches a
    /// set of given string(s).
    ///
    /// Usage:
    ///
    /// 1. {{#if_host "!foo,!bar"}}..baz..{{/if_host}}
    ///
    ///    Renders `..baz..` only if current machine's hostname is neither
    ///    "foo" nor "bar".
    /// 2. {{#if_host "foo"}}..baz..{{else}}..qux..{{/if_host}}
    ///
    ///    Renders `..baz..` only if current machine's hostname is "foo",
    ///    renders `..qux..` only if current user's username is NOT "foo".
    pub fn if_host<'reg, 'rc>(
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        if h.params().len() > 1 {
            return Err(RenderError::new(&format!(
                r#"
Block helper `#{0}`:
    expected exactly 1 argument, {1} found

    Usage:
        1. {{{{#{0} "!foo,!bar"}}}}..bar..{{{{/{0}}}}}
           Renders `..bar..` only if current machine's hostname is neither
           "foo" nor "bar"

        2. {{{{#{0} "foo"}}}}..baz..{{{{else}}}}..qux..{{{{/{0}}}}}
           Renders `..baz..` only if current machine's hostname is "foo",
           renders `..qux..` only if current user's username is NOT "foo""#,
                h.name(),
                h.params().len(),
            )));
        }

        let expected_hostname: String = match h.param(0) {
            Some(v) => {
                if v.value().is_array() {
                    v.value()
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|elem| elem.render())
                        .collect::<Vec<_>>()
                        .join(",")
                } else {
                    v.value().render()
                }
            }
            None => {
                return Err(RenderError::new(&format!(
                    r#"
Block helper `#{0}`:
    expected exactly 1 argument, 0 found

    Usage:
        1. {{{{#{0} "!foo,!bar"}}}}..bar..{{{{/{0}}}}}
           Renders `..bar..` only if current machine's hostname is neither
           "foo" nor "bar"

        2. {{{{#{0} "foo"}}}}..baz..{{{{else}}}}..qux..{{{{/{0}}}}}
           Renders `..baz..` only if current machine's hostname is "foo",
           renders `..qux..` only if current user's username is NOT "foo""#,
                    h.name(),
                )));
            }
        };

        let current_hostname = gethostname();
        let current_hostname: &str =
            &current_hostname.to_string_lossy().to_string();
        let allowed_hostnames: Vec<&str> = expected_hostname
            .split(',')
            .filter(|h| !h.starts_with("!"))
            .collect();
        let disallowed_hostnames: Vec<&str> = expected_hostname
            .split(',')
            .filter_map(|h| {
                if h.starts_with("!") {
                    h.strip_prefix("!")
                } else {
                    None
                }
            })
            .collect();
        if allowed_hostnames.len() > 0 && disallowed_hostnames.len() > 0 {
            return Err(RenderError::new(format!(
                "you can only supply one of positve OR negated type of arguments in a single {}",
                h.name(),
            )));
        }
        if allowed_hostnames.len() > 0 {
            if allowed_hostnames.contains(&current_hostname) {
                log::debug!(
                    "Current hostname {} matches {}",
                    current_hostname,
                    expected_hostname,
                );
                h.template().map(|t| t.render(r, ctx, rc, out));
            } else {
                log::debug!(
                    "Current hostname {} does not match {}",
                    current_hostname,
                    expected_hostname,
                );
                h.inverse().map(|t| t.render(r, ctx, rc, out));
            }
        } else if disallowed_hostnames.len() > 0 {
            if disallowed_hostnames.contains(&current_hostname) {
                log::debug!(
                    "Current hostname {} does not match {}",
                    current_hostname,
                    expected_hostname,
                );
                h.inverse().map(|t| t.render(r, ctx, rc, out));
            } else {
                log::debug!(
                    "Current hostname {} matches {}",
                    current_hostname,
                    expected_hostname,
                );
                h.template().map(|t| t.render(r, ctx, rc, out));
            }
        } else {
            return Err(RenderError::new(format!(
                "no hostname(s) supplied for matching in helper {}",
                h.name(),
            )));
        }
        Ok(())
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Jan 29 2022, 14:42 [CST]
