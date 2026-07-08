use std::path::PathBuf;
use std::{rc::Rc, sync::Arc};

mod config_node;
mod config_value;
mod document;
mod error;
pub mod parse;

#[cfg(feature = "derive")]
pub use config_parser_derive::{ConfigNode, ConfigValue};
pub use {config_node::*, config_value::*, document::*, error::*, parsey::Spanned};

/// Parses a config string into a T
///
/// Note: You need to manually attach the miette source code to the error returned by this function.
///```rust
/// let parsed = config_parser::from_str(source_code).unwrap_or_else(|e| {
///     panic!(
///         "{:?}",
///         Report::from(e).with_source_code(source_code.to_string())
///     )
/// });
///```
pub fn from_str<'c, T: ParseConfigNode<'c>>(str: &'c str) -> Result<T, ConfigError> {
    let doc = Document::from_str(str)?;
    doc.parse_into::<T>()
}

pub trait ParseConfigValue<'c>: Sized {
    fn consume_value(value: Spanned<ConfigValue<'c>>) -> Result<Self>;
}

pub trait ParseConfigNode<'c>: Sized {
    /// Returns true if this node should be parsed as the provided node.
    /// Most implementations just check the name of the node and let the system throw an error
    /// during parsing for missing properties, arguments and child nodes.
    fn match_node(node: &ConfigNode<'c>) -> bool;

    /// Consumes the node into this type.
    /// the parameter `terminate` indicates if the node should be terminated by the function.
    /// After a node is terminated it can't be consumed further anymore and an error is thrown if it
    /// was not fully consumed.
    fn consume_node(node: &mut ConfigNode<'c>, terminate: bool) -> Result<Self>;
}

// impls for common types

impl<'c, T: ParseConfigValue<'c>> ParseConfigValue<'c> for Spanned<T> {
    fn consume_value(value: Spanned<ConfigValue<'c>>) -> Result<Self> {
        let span = value.span.clone();
        Ok(Spanned::new(T::consume_value(value)?, span))
    }
}
impl<'c, T: ParseConfigNode<'c>> ParseConfigNode<'c> for Spanned<T> {
    fn match_node(node: &ConfigNode<'c>) -> bool {
        T::match_node(node)
    }
    fn consume_node(node: &mut ConfigNode<'c>, terminate: bool) -> Result<Self> {
        let span = node.name.span.clone();
        Ok(Spanned::new(T::consume_node(node, terminate)?, span))
    }
}

impl<'c> ParseConfigValue<'c> for ConfigValue<'c> {
    fn consume_value(value: Spanned<ConfigValue<'c>>) -> Result<Self> {
        Ok(value.inner)
    }
}

impl<'c, T: ParseConfigValue<'c>> ParseConfigValue<'c> for Option<T> {
    fn consume_value(value: Spanned<ConfigValue<'c>>) -> Result<Self> {
        Ok(Some(T::consume_value(value)?))
    }
}
impl<'c, T: ParseConfigNode<'c>> ParseConfigNode<'c> for Option<T> {
    fn match_node(node: &ConfigNode<'c>) -> bool {
        T::match_node(node)
    }
    fn consume_node(node: &mut ConfigNode<'c>, terminate: bool) -> Result<Self> {
        Ok(Some(T::consume_node(node, terminate)?))
    }
}

macro_rules! impl_parse_config_value {
    ($($type:ty:$config_type:ident),*) => {
        $(
        impl<'c> ParseConfigValue<'c> for $type {
            fn consume_value(value: Spanned<ConfigValue<'c>>) -> Result<Self> {
                match value.inner {
                    ConfigValue::$config_type(v) => Ok(v.try_into().map_err(|e|ConfigError::message(value.span, format!("Failed to convert number to {}: {}", stringify!($type), e)))?),
                    _=> Err(ConfigError::type_error(&value, ConfigValueType::$config_type))
                }
            }
        }
        )*
    };
}

impl_parse_config_value! {
    f64:Float,

    usize:Int,
    isize:Int,
    i64:Int,
    i32:Int,
    i16:Int,
    i8:Int,
    u64:Int,
    u32:Int,
    u16:Int,
    u8:Int,

    bool:Bool,

    // strings
    Rc<str>:String,
    Arc<str>:String,
    Box<str>:String,
    String:String,
    PathBuf:String
}
