use content_inspector::inspect;
use handlebars::Handlebars;

use crate::{config::DTConfig, error::Result};

/// Helper trait for manipulating registries within DT.
pub trait DTRegistry<'reg>
where
    Self: From<Handlebars<'reg>> + Into<Handlebars<'reg>>,
{
    /// Reads source files from templated groups and register them as
    /// templates into a global registry.
    fn register_templates(self, config: &DTConfig) -> Result<Self> {
        let mut registry = self.into();

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

        Ok(registry.into())
    }

    /// Registers DT's built-in helpers.
    fn register_helpers(self) -> Result<Self> {
        let mut registry: Handlebars = self.into();

        registry.register_helper("get-mine", Box::new(helpers::get_mine));
        registry.register_helper("get_mine", Box::new(helpers::get_mine));
        registry.register_helper("getmine", Box::new(helpers::get_mine));

        Ok(registry.into())
    }
}

impl<'reg> DTRegistry<'reg> for Handlebars<'reg> {}

// ===========================================================================

/// Additional built-in helpers
pub mod helpers {
    use gethostname::gethostname;
    use handlebars::{
        Context, Handlebars, Helper, HelperResult, JsonRender, Output,
        RenderContext, RenderError,
    };

    /// A templating helper that retrieves the value for current host from a
    /// map, returns a fallback value when current host is not recorded in the
    /// map.
    ///
    /// Usage:
    ///     get_mine <map> <deafult-value>
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
                return Err(RenderError::new(&format!(
                    r#"
Helper `{0}`:
    expected 2 arguments, 0 found

    Usage:
        {0} <map> <default-value>"#,
                    h.name(),
                )))
            }
        };
        let default_content = match h.param(1) {
            Some(content) => content.value(),
            None => {
                return Err(RenderError::new(&format!(
                    r#"
Helper `{0}`:
    expected 2 arguments, 1 found

    Usage:
        {0} <map> <default-value>"#,
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
