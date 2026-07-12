#![doc=include_str!("../../README.md")]

use std::path::PathBuf;
use std::{rc::Rc, sync::Arc};

mod config_node;
mod config_value;
mod document;
mod error;
pub mod parse;

#[cfg(feature = "derive")]
pub use starryconfig_derive::{ConfigNode, ConfigValue};
pub use {
    config_node::*,
    config_value::*,
    document::*,
    error::*,
    starryparse::{Span, Spanned},
};

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

#[derive(Clone)]
pub enum AllowedNodeNames<I> {
    Any,
    Iter(I),
}
impl<I: Iterator<Item = &'static str> + Clone> AllowedNodeNames<I> {
    pub fn is_allowed(self, name: &str) -> bool {
        match self {
            Self::Any => true,
            Self::Iter(mut iter) => iter.find(|n| *n == name).is_some(),
        }
    }
    pub fn is_empty(self) -> bool {
        match self {
            Self::Any => false,
            Self::Iter(mut i) => i.next().is_none(),
        }
    }

    pub fn combine(
        self,
        other: AllowedNodeNames<impl Iterator<Item = &'static str> + Clone>,
    ) -> AllowedNodeNames<impl Iterator<Item = &'static str> + Clone> {
        match (self, other) {
            (AllowedNodeNames::Any, _) => AllowedNodeNames::Any,
            (_, AllowedNodeNames::Any) => AllowedNodeNames::Any,
            (AllowedNodeNames::Iter(iter1), AllowedNodeNames::Iter(iter2)) => {
                AllowedNodeNames::Iter(iter1.chain(iter2))
            }
        }
    }
}
impl<I: Iterator<Item = &'static str> + Clone> ToString for AllowedNodeNames<I> {
    fn to_string(&self) -> String {
        let mut string = String::new();
        match self {
            Self::Any => string.push_str("any node"),
            Self::Iter(iter) => {
                for node_name in iter.clone() {
                    string.push_str(node_name);
                    string.push(',');
                }
                string.pop();
            }
        }
        string
    }
}
impl<I> AllowedNodeNames<I> {
    pub fn empty() -> AllowedNodeNames<impl Iterator<Item = &'static str> + Clone> {
        AllowedNodeNames::Iter(std::iter::empty::<&'static str>())
    }
    pub fn from_single(
        name: &'static str,
    ) -> AllowedNodeNames<impl Iterator<Item = &'static str> + Clone> {
        AllowedNodeNames::Iter(std::iter::once(name))
    }
    pub fn any() -> AllowedNodeNames<std::iter::Empty<&'static str>> {
        AllowedNodeNames::Any
    }
    pub fn from_slice(
        slice: &[&'static str],
    ) -> AllowedNodeNames<impl Iterator<Item = &'static str> + Clone> {
        AllowedNodeNames::Iter(slice.iter().map(|c| *c))
    }
}

pub trait ParseConfigValue<'c>: Sized {
    fn consume_value(value: Spanned<ConfigValue<'c>>) -> Result<Self>;
}

pub trait ParseConfigNode<'c>: Sized {
    /// Node names which this node can be parsed from.
    fn allowed_node_names() -> AllowedNodeNames<impl Iterator<Item = &'static str> + Clone>;

    /// Consumes the node into this type.
    ///
    /// The parameter `terminate` indicates if the node should be terminated by the function.
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
    fn allowed_node_names() -> AllowedNodeNames<impl Iterator<Item = &'static str> + Clone> {
        T::allowed_node_names()
    }
    fn consume_node(node: &mut ConfigNode<'c>, terminate: bool) -> Result<Self> {
        let span = node.name_spanned().span.clone();
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
    fn allowed_node_names() -> AllowedNodeNames<impl Iterator<Item = &'static str> + Clone> {
        T::allowed_node_names()
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
