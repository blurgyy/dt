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
        render_env.register_helper("for_user", Box::new(helpers::for_user));
        render_env.register_helper("for_uid", Box::new(helpers::for_uid));
        render_env.register_helper("for_host", Box::new(helpers::for_host));

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
            (Renders current machine's hostname)

        2. {{{{ {0} <map> <default-value> }}}}
            (Gets value of <map>.$CURRENT_HOSTNAME, falls back to <default-value>)"#,
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
    /// given string.
    ///
    /// Usage:
    ///
    /// 1. `{{#for_user "foo"}}..content..{{/for_user}}`
    ///
    ///     Renders content only if current user's username is "foo".
    /// 2. `{{#for_user "foo"}}{{else}}..content..{{/for_user}}`
    ///
    ///     Renders content only if current user's username is NOT "foo".
    pub fn for_user<'reg, 'rc>(
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
        1. {{{{#{0} "foo"}}}}..content..{{{{/{0}}}}}
                (Renders `..content..` only if current user's username is "foo")

        2. {{{{#{0} "foo"}}}}{{{{else}}}}..content..{{{{/{0}}}}}
                (Renders `..content..` only if current user's username is NOT "foo")
                    "#,
                h.name(),
                h.params().len(),
            )));
        }

        let username: String = match h.param(0) {
            Some(v) => v.value().render(),
            None => {
                return Err(RenderError::new(&format!(
                    r#"
Block helper `#{0}`:
    expected exactly 1 argument, 0 found

    Usage:
        1. {{{{#{0} "foo"}}}}..content..{{{{/{0}}}}}
                (Renders `..content..` only if current user's username is "foo")

        2. {{{{#{0} "foo"}}}}{{{{else}}}}..content..{{{{/{0}}}}}
                (Renders `..content..` only if current user's username is NOT "foo")
                    "#,
                    h.name(),
                )));
            }
        };

        let current_username = get_current_username()
            .unwrap()
            .to_string_lossy()
            .to_string();
        if current_username == username {
            log::debug!(
                "Current username {} matches {}",
                current_username,
                username,
            );
            h.template().map(|t| t.render(r, ctx, rc, out));
        } else {
            log::debug!(
                "Current username {} is not {}",
                current_username,
                username,
            );
            h.inverse().map(|t| t.render(r, ctx, rc, out));
        }
        Ok(())
    }

    /// A templating helper that tests if current user's effective uid matches
    /// a given integer.
    ///
    /// Usage:
    ///
    /// 1. `{{#for_uid 0}}..content..{{/for_uid}}`
    ///
    ///     Renders `..content..` only if current user's effective uid is 0.
    /// 2. `{{#for_uid 0}}{{else}}..content..{{/for_uid}}`
    ///
    ///     Renders `..content..` only if current user's effective uid is NOT 0.
    pub fn for_uid<'reg, 'rc>(
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
        1. {{{{#{0} 0}}}}..content..{{{{/{0}}}}}
            (Renders `..content..` only if current user's effective uid is 0)

        2. {{{{#{0} 0}}}}{{{{else}}}}..content..{{{{/{0}}}}}
            (Renders `..content..` only if current user's effective uid is NOT 0)
                    "#,
                h.name(),
                h.params().len(),
            )));
        }

        let uid: u32 = match h.param(0) {
            Some(v) => v.value().render().parse()?,
            None => {
                return Err(RenderError::new(&format!(
                    r#"
Block helper `#{0}`:
    expected exactly 1 argument, 0 found

    Usage:
        1. {{{{#{0} 0}}}}..content..{{{{/{0}}}}}
            (Renders `..content..` only if current user's effective uid is 0)

        2. {{{{#{0} 0}}}}{{{{else}}}}..content..{{{{/{0}}}}}
            (Renders `..content..` only if current user's effective uid is NOT 0)
                    "#,
                    h.name(),
                )));
            }
        };

        let current_uid = get_current_uid();
        if current_uid == uid {
            log::debug!("Current uid '{}' matches '{}'", current_uid, uid);
            h.template().map(|t| t.render(r, ctx, rc, out));
        } else {
            log::debug!("Current uid '{}' is not '{}'", current_uid, uid);
            h.inverse().map(|t| t.render(r, ctx, rc, out));
        }
        Ok(())
    }

    /// A templating helper that tests if current machine's hostname matches a
    /// given string.
    ///
    /// Usage:
    ///
    /// 1. `{{#for_host "bar"}}..content..{{/for_host}}`
    ///
    ///     Renders content only if current machine's hostname is "bar".
    /// 2. `{{#for_host "bar"}}{{else}}..content..{{/for_host}}`
    ///
    ///     Renders content only if current machine's hostname is NOT "bar".
    pub fn for_host<'reg, 'rc>(
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
        1. {{{{#{0} "bar"}}}}..content..{{{{/{0}}}}}
                (Renders `..content..` only if current machine's hostname is "bar")

        2. {{{{#{0} "bar"}}}}{{{{else}}}}..content..{{{{/{0}}}}}
                (Renders `..content..` only if current machine's hostname is NOT "bar")
                    "#,
                h.name(),
                h.params().len(),
            )));
        }

        let expected_hostname: String = match h.param(0) {
            Some(v) => v.value().render(),
            None => {
                return Err(RenderError::new(&format!(
                    r#"
Block helper `#{0}`:
    expected exactly 1 argument, 0 found

    Usage:
        1. {{{{#{0} "bar"}}}}..content..{{{{/{0}}}}}
                (Renders `..content..` only if current machine's hostname is "bar")

        2. {{{{#{0} "bar"}}}}{{{{else}}}}..content..{{{{/{0}}}}}
                (Renders `..content..` only if current machine's hostname is NOT "bar")
                    "#,
                    h.name(),
                )));
            }
        };

        let current_hostname = gethostname();
        let current_hostname = current_hostname.to_string_lossy();
        if current_hostname == expected_hostname {
            log::debug!(
                "Current hostname {} matches {}",
                current_hostname,
                expected_hostname,
            );
            h.template().map(|t| t.render(r, ctx, rc, out));
        } else {
            log::debug!(
                "Current hostname {} is not {}",
                current_hostname,
                expected_hostname,
            );
            h.inverse().map(|t| t.render(r, ctx, rc, out));
        }
        Ok(())
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Jan 29 2022, 14:42 [CST]
