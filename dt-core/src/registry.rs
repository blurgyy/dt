use std::{
    collections::HashMap,
    io::{Read, Seek},
    rc::Rc,
};

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
    /// templated (see [`renderable`]) will not be registered into templates
    /// but directly stored into the rendered cache.
    ///
    /// [`renderable`]: crate::config::Group::renderable
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
        render_env
            .register_helper("unless_user", Box::new(helpers::unless_user));
        render_env
            .register_helper("unless_uid", Box::new(helpers::unless_uid));
        render_env
            .register_helper("unless_host", Box::new(helpers::unless_host));

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

                let mut f = std::fs::File::open(s)?;
                f.seek(std::io::SeekFrom::Start(0))?;
                let mut indicator =
                    vec![
                        0;
                        std::cmp::min(1024, f.metadata()?.len() as usize)
                    ];
                f.read_exact(&mut indicator)?;

                if group.is_renderable() {
                    if inspect(&indicator).is_text() {
                        let content = std::fs::read(s)?;
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
                            "'{}' has binary contents, skipping rendering",
                            s.display(),
                        );
                    }
                } else {
                    log::trace!(
                        "'{}' is from an unrenderable group '{}'",
                        s.display(),
                        group.name,
                    );
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
                None => Ok(std::fs::read(name)?),
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
    #[cfg(not(test))]
    use {
        gethostname::gethostname,
        users::{get_current_uid, get_current_username},
    };

    #[cfg(test)]
    use crate::utils::testing::{
        get_current_uid, get_current_username, gethostname,
    };

    use handlebars::{
        Context, Handlebars, Helper, HelperResult, JsonRender, Output,
        RenderContext, RenderError, Renderable,
    };

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
    /// 1. {{#if_user "foo,bar"}}..baz..{{/if_user}}
    ///
    ///    Renders `..baz..` only if current user's username is either "foo"
    ///    or "bar".
    /// 2. {{#if_user "foo"}}..baz..{{else}}..qux..{{/if_user}}
    ///
    ///    Renders `..baz..` only if current user's username is "foo", renders
    ///    `..qux..` only if current user's username is NOT "foo".
    ///
    /// 3. {{#if_user some.array}}..foo..{{/if_user}}
    ///
    ///    Renders `..foo..` only if current user's username is exactly one of
    ///    the values from the templating variable `some.array` (defined in
    ///    the config file's [`[context]`] section).
    ///
    /// [`[context]`]: dt_core::config::ContextConfig
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
        1. {{{{#{0} "foo,bar"}}}}..baz..{{{{/{0}}}}}
           Renders `..baz..` only if current user's username is either "foo"
           or "bar"

        2. {{{{#{0} "foo"}}}}..baz..{{{{else}}}}..qux..{{{{/{0}}}}}
           Renders `..baz..` only if current user's username is "foo", renders
           `..qux..` only if current user's username is NOT "foo"

        3. {{{{#{0} some.array}}}}..foo..{{{{/{0}}}}}
           Renders `..foo..` only if current user's username is exactly one of
           the values from the templating variable `some.array` (defined in
           the config file's `[context]` section)"#,
                h.name(),
                h.params().len(),
            )));
        }

        let allowed_usernames: Vec<String> = match h.param(0) {
            Some(v) => {
                if v.value().is_array() {
                    v.value()
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|elem| elem.render())
                        .collect()
                } else {
                    v.value()
                        .render()
                        .split(',')
                        .map(|u| u.trim().to_owned())
                        .collect()
                }
            }
            None => {
                return Err(RenderError::new(&format!(
                    r#"
Block helper `#{0}`:
    expected exactly 1 argument, 0 found

    Usage:
        1. {{{{#{0} "foo,bar"}}}}..baz..{{{{/{0}}}}}
           Renders `..baz..` only if current user's username is either "foo"
           or "bar"

        2. {{{{#{0} "foo"}}}}..baz..{{{{else}}}}..qux..{{{{/{0}}}}}
           Renders `..baz..` only if current user's username is "foo", renders
           `..qux..` only if current user's username is NOT "foo"

        3. {{{{#{0} some.array}}}}..foo..{{{{/{0}}}}}
           Renders `..foo..` only if current user's username is exactly one of
           the values from the templating variable `some.array` (defined in
           the config file's `[context]` section)"#,
                    h.name(),
                )));
            }
        };

        let current_username = get_current_username()
            .unwrap()
            .to_string_lossy()
            .to_string();
        if !allowed_usernames.is_empty() {
            if allowed_usernames.contains(&current_username) {
                log::debug!(
                    "Current username '{}' matches allowed usernames '{:?}'",
                    current_username,
                    allowed_usernames,
                );
                h.template().map(|t| t.render(r, ctx, rc, out));
            } else {
                log::debug!(
                    "Current username '{}' does not match allowed usernames {:?}",
                    current_username,
                    allowed_usernames,
                );
                h.inverse().map(|t| t.render(r, ctx, rc, out));
            }
        } else {
            return Err(RenderError::new(format!(
                "no username(s) supplied for matching in helper {}",
                h.name(),
            )));
        }
        Ok(())
    }

    /// A templating helper that tests if current user's username does not
    /// match a set of given string(s).  It is the negated version of
    /// [`if_user`].
    ///
    /// Usage:
    ///
    /// 1. {{#unless_user "foo,bar"}}..baz..{{/unless_user}}
    ///
    ///    Renders `..baz..` only if current user's username is neither "foo"
    ///    nor "bar".
    /// 2. {{#unless_user "foo"}}..baz..{{else}}..qux..{{/unless_user}}
    ///
    ///    Renders `..baz..` only if current user's username is NOT "foo",
    ///    renders `..qux..` only if current user's username is "foo".
    ///
    /// 3. {{#unless_user some.array}}..foo..{{/unless_user}}
    ///
    ///    Renders `..foo..` only if current user's username is none of the
    ///    values from the templating variable `some.array` (defined in the
    ///    config file's [`[context]`] section).
    ///
    /// [`if_user`]: if_user
    /// [`[context]`]: dt_core::config::ContextConfig
    pub fn unless_user<'reg, 'rc>(
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
        1. {{{{#{0} "foo,bar"}}}}..baz..{{{{/{0}}}}}
           Renders `..baz..` only if current user's username is neither "foo"
           nor "bar"

        2. {{{{#{0} "foo"}}}}..baz..{{{{else}}}}..qux..{{{{/{0}}}}}
           Renders `..baz..` only if current user's username is NOT "foo",
           renders `..qux..` only if current user's username is "foo"

        3. {{{{#{0} some.array}}}}..foo..{{{{/{0}}}}}

           Renders `..foo..` only if current user's username is none of the
           values from the templating variable `some.array` (defined in the
           config file's `[context]` section)"#,
                h.name(),
                h.params().len(),
            )));
        }

        let disallowed_usernames: Vec<String> = match h.param(0) {
            Some(v) => {
                if v.value().is_array() {
                    v.value()
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|elem| elem.render())
                        .collect()
                } else {
                    v.value()
                        .render()
                        .split(',')
                        .map(|u| u.trim().to_owned())
                        .collect()
                }
            }
            None => {
                return Err(RenderError::new(&format!(
                    r#"
Block helper `#{0}`:
    expected exactly 1 argument, 0 found

    Usage:
        1. {{{{#{0} "foo,bar"}}}}..baz..{{{{/{0}}}}}
           Renders `..baz..` only if current user's username is neither "foo"
           nor "bar"

        2. {{{{#{0} "foo"}}}}..baz..{{{{else}}}}..qux..{{{{/{0}}}}}
           Renders `..baz..` only if current user's username is NOT "foo",
           renders `..qux..` only if current user's username is "foo"

        3. {{{{#{0} some.array}}}}..foo..{{{{/{0}}}}}

           Renders `..foo..` only if current user's username is none of the
           values from the templating variable `some.array` (defined in the
           config file's `[context]` section)"#,
                    h.name(),
                )));
            }
        };

        let current_username: String = get_current_username()
            .unwrap()
            .to_string_lossy()
            .to_string();
        if !disallowed_usernames.is_empty() {
            if disallowed_usernames.contains(&current_username) {
                log::debug!(
                    "Current username '{}' matches disallowed usernames '{:?}'",
                    current_username,
                    disallowed_usernames,
                );
                h.inverse().map(|t| t.render(r, ctx, rc, out));
            } else {
                log::debug!(
                    "Current username '{}' does not match disallowed usernames {:?}",
                    current_username,
                    disallowed_usernames,
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
    /// 1. `{{#if_uid "1000,1001"}}..foo..{{/if_uid}}`
    ///
    ///    Renders `..foo..` only if current user's effective uid is either
    ///    `1000` or `1001`.
    /// 2. `{{#if_uid 0}}..foo..{{else}}..bar..{{/if_uid}}`
    ///
    ///    Renders `..foo..` only if current user's effective uid is `0`,
    ///    renders `..bar..` only if current user's effective uid is not `0`.
    /// 3. {{#if_uid some.array}}..foo..{{/if_uid}}
    ///
    ///    Renders `..foo..` only if current user's effective uid is exactly
    ///    one of the values from the templating variable `some.array`
    ///    (defined in the config file's [`[context]`] section).
    ///
    /// [`[context]`]: dt_core::config::ContextConfig
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
        1. {{{{#{0} "1000,1001"}}}}..foo..{{{{/{0}}}}}
           Renders `..foo..` only if current user's effective uid is either
           `1000` or `1001`"

        2. {{{{#{0} 0}}}}..foo..{{{{else}}}}..bar..{{{{/{0}}}}}
           Renders `..foo..` only if current user's effective uid is `0`,
           renders `..bar..` if current user's effective uid is not `0`

        3. {{{{#{0} some.array}}}}..foo..{{{{/{0}}}}}

           Renders `..foo..` only if current user's effective uid is exactly
           one of the values from the templating variable `some.array`
           (defined in the config file's `[context]` section)"#,
                h.name(),
                h.params().len(),
            )));
        }

        let allowed_uids: Vec<u32> = match h.param(0) {
            Some(v) => {
                if v.value().is_array() {
                    v.value()
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|elem| elem.render().parse())
                        .collect::<Result<Vec<_>, _>>()?
                } else {
                    v.value()
                        .render()
                        .split(',')
                        .map(|uid| uid.parse())
                        .collect::<Result<Vec<_>, _>>()?
                }
            }
            None => {
                return Err(RenderError::new(&format!(
                    r#"
Block helper `#{0}`:
    expected exactly 1 argument, 0 found

    Usage:
        1. {{{{#{0} "1000,1001"}}}}..foo..{{{{/{0}}}}}
           Renders `..foo..` only if current user's effective uid is either
           `1000` or `1001`"

        2. {{{{#{0} 0}}}}..foo..{{{{else}}}}..bar..{{{{/{0}}}}}
           Renders `..foo..` only if current user's effective uid is `0`,
           renders `..bar..` if current user's effective uid is not `0`

        3. {{{{#{0} some.array}}}}..foo..{{{{/{0}}}}}

           Renders `..foo..` only if current user's effective uid is exactly
           one of the values from the templating variable `some.array`
           (defined in the config file's `[context]` section)"#,
                    h.name(),
                )));
            }
        };

        let current_uid = get_current_uid();
        if !allowed_uids.is_empty() {
            if allowed_uids.contains(&current_uid) {
                log::debug!(
                    "Current uid '{}' matches allowed uids '{:?}'",
                    current_uid,
                    allowed_uids,
                );
                h.template().map(|t| t.render(r, ctx, rc, out));
            } else {
                log::debug!(
                    "Current uid '{}' does not match allowed uids {:?}",
                    current_uid,
                    allowed_uids,
                );
                h.inverse().map(|t| t.render(r, ctx, rc, out));
            }
        } else {
            return Err(RenderError::new(format!(
                "no uid(s) supplied for matching in helper {}",
                h.name(),
            )));
        }
        Ok(())
    }

    /// A templating helper that tests if current user's effective uid matches
    /// a set of given integer(s).  It is the negated version of [`if_uid`].
    ///
    /// Usage:
    ///
    /// 1. `{{#unless_uid "1000,1001"}}..foo..{{/unless_uid}}`
    ///
    ///    Renders `..foo..` only if current user's effective uid is neither
    ///    `1000` nor `1001`.
    /// 2. `{{#unless_uid 0}}..foo..{{else}}..bar..{{/unless_uid}}`
    ///
    ///    Renders `..foo..` only if current user's effective uid is NOT `0`,
    ///    renders `..bar..` only if current user's effective uid is `0`.
    /// 3. `{{#unless_uid some.array}}..foo..{{/unless_uid}}`
    ///
    ///    Renders `..foo..` only if current user's effective uid is none of
    ///    the values from the templating variable `some.array` (defined in
    ///    the config file's [`[context]`] section).
    ///
    /// [`if_uid`]: if_uid
    /// [`[context]`]: dt_core::config::ContextConfig
    pub fn unless_uid<'reg, 'rc>(
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
        1. {{{{#{0} "1000,1001"}}}}..foo..{{{{/{0}}}}}
           Renders `..foo..` only if current user's effective uid is neither
           `1000` nor `1001`"

        2. {{{{#{0} 0}}}}..foo..{{{{else}}}}..bar..{{{{/{0}}}}}
           Renders `..foo..` only if current user's effective uid is NOT `0`,
           renders `..bar..` if current user's effective uid is `0`

        3. {{{{#{0} some.array}}}}..foo..{{{{/{0}}}}}`

           Renders `..foo..` only if current user's effective uid is none of
           the values from the templating variable `some.array` (defined in
           the config file's `[context]` section)"#,
                h.name(),
                h.params().len(),
            )));
        }

        let disallowed_uids: Vec<u32> = match h.param(0) {
            Some(v) => {
                if v.value().is_array() {
                    v.value()
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|elem| elem.render().parse())
                        .collect::<Result<Vec<_>, _>>()?
                } else {
                    v.value()
                        .render()
                        .split(',')
                        .map(|uid| uid.parse())
                        .collect::<Result<Vec<_>, _>>()?
                }
            }
            None => {
                return Err(RenderError::new(&format!(
                    r#"
Block helper `#{0}`:
    expected exactly 1 argument, 0 found

    Usage:
        1. {{{{#{0} "1000,1001"}}}}..foo..{{{{/{0}}}}}
           Renders `..foo..` only if current user's effective uid is neither
           `1000` nor `1001`"

        2. {{{{#{0} 0}}}}..foo..{{{{else}}}}..bar..{{{{/{0}}}}}
           Renders `..foo..` only if current user's effective uid is NOT `0`,
           renders `..bar..` if current user's effective uid is `0`

        3. `{{{{#{0} some.array}}}}..foo..{{{{/{0}}}}}`

           Renders `..foo..` only if current user's effective uid is none of
           the values from the templating variable `some.array` (defined in
           the config file's `[context]` section)"#,
                    h.name(),
                )));
            }
        };

        let current_uid = get_current_uid();
        if !disallowed_uids.is_empty() {
            if disallowed_uids.contains(&current_uid) {
                log::debug!(
                    "Current uid '{}' matches disallowed uids '{:?}'",
                    current_uid,
                    disallowed_uids,
                );
                h.inverse().map(|t| t.render(r, ctx, rc, out));
            } else {
                log::debug!(
                    "Current uid '{}' does not match disallowed uids '{:?}'",
                    current_uid,
                    disallowed_uids,
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
    /// 1. {{#if_host "foo,bar"}}..baz..{{/if_host}}
    ///
    ///    Renders `..baz..` only if current machine's hostname is either
    ///    "foo" or "bar".
    /// 2. {{#if_host "foo"}}..baz..{{else}}..qux..{{/if_host}}
    ///
    ///    Renders `..baz..` only if current machine's hostname is "foo",
    ///    renders `..qux..` only if current user's username is NOT "foo".
    /// 3. `{{#if_host some.array}}..foo..{{/if_host}}`
    ///
    ///    Renders `..foo..` only if current machine's hostname is exactly one
    ///    of the values from the templating variable `some.array` (defined in
    ///    the config file's [`[context]`] section).
    ///
    /// [`[context]`]: dt_core::config::ContextConfig
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
        1. {{{{#{0} "foo,bar"}}}}..bar..{{{{/{0}}}}}
           Renders `..bar..` only if current machine's hostname is either
           "foo" or "bar"

        2. {{{{#{0} "foo"}}}}..baz..{{{{else}}}}..qux..{{{{/{0}}}}}
           Renders `..baz..` only if current machine's hostname is "foo",
           renders `..qux..` only if current user's username is NOT "foo"

        3. `{{{{#{0} some.array}}}}..foo..{{{{/{0}}}}}`

           Renders `..foo..` only if current machine's hostname is exactly one
           of the values from the templating variable `some.array` (defined in
           the config file's `[context]` section)"#,
                h.name(),
                h.params().len(),
            )));
        }

        let allowed_hostnames: Vec<String> = match h.param(0) {
            Some(v) => {
                if v.value().is_array() {
                    v.value()
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|elem| elem.render())
                        .collect::<Vec<_>>()
                } else {
                    v.value()
                        .render()
                        .split(',')
                        .map(|h| h.trim().to_owned())
                        .collect()
                }
            }
            None => {
                return Err(RenderError::new(&format!(
                    r#"
Block helper `#{0}`:
    expected exactly 1 argument, 0 found

    Usage:
        1. {{{{#{0} "foo,bar"}}}}..bar..{{{{/{0}}}}}
           Renders `..bar..` only if current machine's hostname is either
           "foo" or "bar"

        2. {{{{#{0} "foo"}}}}..baz..{{{{else}}}}..qux..{{{{/{0}}}}}
           Renders `..baz..` only if current machine's hostname is "foo",
           renders `..qux..` only if current user's username is NOT "foo"

        3. `{{{{#{0} some.array}}}}..foo..{{{{/{0}}}}}`

           Renders `..foo..` only if current machine's hostname is exactly one
           of the values from the templating variable `some.array` (defined in
           the config file's `[context]` section)"#,
                    h.name(),
                )));
            }
        };

        let current_hostname = gethostname();
        let current_hostname: String =
            current_hostname.to_string_lossy().to_string();
        if !allowed_hostnames.is_empty() {
            if allowed_hostnames.contains(&current_hostname) {
                log::debug!(
                    "Current hostname '{}' matches allowed hostnames '{:?}'",
                    current_hostname,
                    allowed_hostnames,
                );
                h.template().map(|t| t.render(r, ctx, rc, out));
            } else {
                log::debug!(
                    "Current hostname '{}' does not match allowed hostnames '{:?}'",
                    current_hostname,
                    allowed_hostnames,
                );
                h.inverse().map(|t| t.render(r, ctx, rc, out));
            }
        } else {
            return Err(RenderError::new(format!(
                "no hostname(s) supplied for matching in helper {}",
                h.name(),
            )));
        }
        Ok(())
    }

    /// A templating helper that tests if current machine's hostname matches a
    /// set of given string(s).  It it the negated version of [`if_host`]
    ///
    /// Usage:
    ///
    /// 1. {{#unless_host "foo,bar"}}..baz..{{/unless_host}}
    ///
    ///    Renders `..baz..` only if current machine's hostname is neither
    ///    "foo" nor "bar".
    /// 2. {{#unless_host "foo"}}..baz..{{else}}..qux..{{/unless_host}}
    ///
    ///    Renders `..baz..` only if current machine's hostname is NOT "foo",
    ///    renders `..qux..` only if current user's username is "foo".
    /// 3. `{{#unless_host some.array}}..foo..{{/unless_host}}`
    ///
    ///    Renders `..foo..` only if current machine's hostname is none of the
    ///    values from the templating variable `some.array` (defined in the
    ///    config file's [`[context]`] section).
    ///
    /// [`if_host`]: if_host
    /// [`[context]`]: dt_core::config::ContextConfig
    pub fn unless_host<'reg, 'rc>(
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
        1. {{{{#{0} "foo,bar"}}}}..bar..{{{{/{0}}}}}
           Renders `..bar..` only if current machine's hostname is neither
           "foo" nor "bar"

        2. {{{{#{0} "foo"}}}}..baz..{{{{else}}}}..qux..{{{{/{0}}}}}
           Renders `..baz..` only if current machine's hostname is NOT "foo",
           renders `..qux..` only if current user's username is "foo"

        3. `{{{{#{0} some.array}}}}..foo..{{{{/{0}}}}}`

           Renders `..foo..` only if current machine's hostname is exactly one
           of the values from the templating variable `some.array` (defined in
           the config file's `[context]` section)"#,
                h.name(),
                h.params().len(),
            )));
        }

        let disallowed_hostnames: Vec<String> = match h.param(0) {
            Some(v) => {
                if v.value().is_array() {
                    v.value()
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|elem| elem.render())
                        .collect::<Vec<_>>()
                } else {
                    v.value()
                        .render()
                        .split(',')
                        .map(|h| h.trim().to_owned())
                        .collect()
                }
            }
            None => {
                return Err(RenderError::new(&format!(
                    r#"
Block helper `#{0}`:
    expected exactly 1 argument, 0 found

    Usage:
        1. {{{{#{0} "foo,bar"}}}}..bar..{{{{/{0}}}}}
           Renders `..bar..` only if current machine's hostname is neither
           "foo" nor "bar"

        2. {{{{#{0} "foo"}}}}..baz..{{{{else}}}}..qux..{{{{/{0}}}}}
           Renders `..baz..` only if current machine's hostname is NOT "foo",
           renders `..qux..` only if current user's username is "foo"

        3. `{{{{#{0} some.array}}}}..foo..{{{{/{0}}}}}`

           Renders `..foo..` only if current machine's hostname is exactly one
           of the values from the templating variable `some.array` (defined in
           the config file's `[context]` section)"#,
                    h.name(),
                )));
            }
        };

        let current_hostname = gethostname();
        let current_hostname: String =
            current_hostname.to_string_lossy().to_string();
        if !disallowed_hostnames.is_empty() {
            if disallowed_hostnames.contains(&current_hostname) {
                log::debug!(
                    "Current hostname '{}' matches disallowed hostnames '{:?}'",
                    current_hostname,
                    disallowed_hostnames,
                );
                h.inverse().map(|t| t.render(r, ctx, rc, out));
            } else {
                log::debug!(
                    "Current hostname '{}' does not match disallowed hostnames '{:?}'",
                    current_hostname,
                    disallowed_hostnames,
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
