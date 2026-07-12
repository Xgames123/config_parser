use std::path::PathBuf;
use std::{rc::Rc, sync::Arc};

use crate::{
    AllowedNodeNames, ConfigError, ConfigNode, ConfigValue, ConfigValueType, Result, Spanned,
};

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
