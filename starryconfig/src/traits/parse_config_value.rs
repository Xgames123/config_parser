use crate::{ConfigError, ConfigValue, ConfigValueType, Result, Spanned};
use std::path::PathBuf;
use std::{rc::Rc, sync::Arc};

pub trait ParseConfigValue<'c>: Sized {
    fn consume_value(value: Spanned<ConfigValue<'c>>) -> Result<Self>;
}

impl<'c, T: ParseConfigValue<'c>> ParseConfigValue<'c> for Spanned<T> {
    fn consume_value(value: Spanned<ConfigValue<'c>>) -> Result<Self> {
        let span = value.span.clone();
        Ok(Spanned::new(T::consume_value(value)?, span))
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
