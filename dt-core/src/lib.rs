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

/// Helper utilites used internally (the [`DTRegistry`] trait) and exposed for
/// templating uses (additional [built-in helpers]).
///
/// [`DTRegistry`]: registry::DTRegistry
/// [built-in helpers]: registry::helpers
#[deny(missing_docs)]
pub mod registry;

/// Definitions for syncing behaviours.
#[deny(missing_docs)]
pub mod syncing;

/// Miscellaneous utilities.
#[deny(missing_docs)]
pub mod utils;

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 17 2021, 21:32 [CST]
