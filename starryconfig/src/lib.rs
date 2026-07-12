#![doc=include_str!("../README.md")]

mod allowed_node_names;
mod config_node;
mod config_value;
mod document;
mod error;
pub mod parse;
mod traits;

pub use {
    allowed_node_names::*,
    config_node::*,
    config_value::*,
    document::*,
    error::*,
    starryparse::{Span, Spanned},
    traits::*,
};

/// Only available on feature `derive`.
#[cfg(feature = "derive")]
pub use starryconfig_derive::ConfigNode;

/// Only available on feature `derive`.
#[cfg(feature = "derive")]
pub use starryconfig_derive::ConfigValue;

/// Parses a config string into a T
///
/// Note: You need to manually attach the miette source code to the error returned by this function.
///```rust
///# #[derive(starryconfig::ConfigNode)]
///# struct MyType { }
///# let source_code = "";
///
/// let parsed : MyType = starryconfig::from_str(source_code).unwrap_or_else(|e| {
///     panic!(
///         "{:?}",
///         miette::Report::from(e).with_source_code(source_code.to_string())
///     )
/// });
/// # let parsed : MyType = parsed;
///```
pub fn from_str<'c, T: ParseConfigNode<'c>>(str: &'c str) -> Result<T, ConfigError> {
    let doc = Document::from_str(str)?;
    doc.parse_into::<T>()
}
