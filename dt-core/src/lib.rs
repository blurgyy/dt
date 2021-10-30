//! This is a helper library, containing shared utilities used by [`DT`].
//!
//! [`DT`]: https://github.com/blurgyy/dt
#[deny(missing_docs)]
/// Definitions for configuration structures and rules.
pub mod config;
/// Definitions for errors
pub mod error;
/// Definition for the [`DTItem`] trait.
///
/// [`DTItem`]: item::DTItem
pub mod item;
/// Definitions for syncing behaviours.
pub mod syncing;
/// Miscellaneous utilities.
pub mod utils;

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Sep 17 2021, 21:32 [CST]
