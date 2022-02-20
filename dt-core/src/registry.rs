use content_inspector::inspect;
use handlebars::Handlebars;

use crate::{config::DTConfig, error::Result};

#[allow(unused_variables)]
/// Helper trait for manipulating registries within DT.
pub trait DTRegistry
where
    Self: Sized,
{
    /// Reads source files from templated groups and register them as
    /// templates into a global registry.
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
    fn register_templates(self, config: &DTConfig) -> Result<Self> {
        unimplemented!()
    }
    /// Registers DT's [built-in helpers].
    ///
    /// [built-in helpers]: helpers
    fn register_helpers(self) -> Result<Self> {
        unimplemented!()
    }
}

impl DTRegistry for Handlebars<'_> {
    fn register_templates(self, config: &DTConfig) -> Result<Self> {
        let mut registry = self;
        for group in &config.local {
            if group.is_templated() {
                for s in &group.sources {
                    if let Ok(content) = std::fs::read(s) {
                        if inspect(&content).is_text() {
                            registry.register_template_string(
                                s.to_str().unwrap(),
                                std::str::from_utf8(&content)?,
                            )?;
                        } else {
                            log::trace!(
                            "'{}' seems to have binary contents, it will not be rendered",
                            s.display(),
                        );
                        }
                    }
                }
            }
        }
        Ok(registry)
    }

    fn register_helpers(self) -> Result<Self> {
        let mut registry: Handlebars = self.into();

        registry.register_helper("get-mine", Box::new(helpers::get_mine));
        registry.register_helper("get_mine", Box::new(helpers::get_mine));
        registry.register_helper("getmine", Box::new(helpers::get_mine));

        Ok(registry.into())
    }
}

// ===========================================================================

/// Additional built-in helpers
pub mod helpers {
    use gethostname::gethostname;
    use handlebars::{
        Context, Handlebars, Helper, HelperResult, JsonRender, Output,
        RenderContext, RenderError,
    };

    /// A templating helper that retrieves the value for current host from a
    /// map, returns a default value when current host is not recorded in the
    /// map.
    ///
    /// Usage:
    ///
    /// 1. `{{ get_mine }}`
    ///
    ///     Render current machin's hostname.
    /// 2. `{{ get_mine <map> <default-value> }}`
    ///
    ///     Render `<map>.$CURRENT_HOSTNAME`, falls back to `<default-value>`.
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
                out.write(
                    gethostname().to_str().expect("Failed getting hostname"),
                )?;
                return Ok(());
            }
        };
        let default_content = match h.param(1) {
            Some(content) => content.value(),
            None => {
                return Err(RenderError::new(&format!(
                    r#"
Helper `{0}`:
    expected 0 or 2 arguments, 1 found

    Usage:
        1. {{{{ {0} }}}}
            (Render current machine's hostname)

        2. {{{{ {0} <map> <default-value> }}}}
            (Gets value of <map>.$CURRENT_HOSTNAME, falls back to <default-value>)"#,
                    h.name(),
                )))
            }
        };

        let content = match map
            .get(gethostname().to_str().expect("Failed getting hostname"))
        {
            Some(content) => content.render(),
            None => default_content.render(),
        };

        out.write(&content)?;

        Ok(())
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Jan 29 2022, 14:42 [CST]
