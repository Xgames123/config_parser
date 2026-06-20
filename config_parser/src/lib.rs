use crate::parse::{ConfigNode, ConfigValue, ConfigValueType, Document, SyntaxError};
use miette::{Diagnostic, SourceSpan};
use parsey::Span;
use std::path::PathBuf;
use std::{borrow::Cow, rc::Rc, sync::Arc};
use thiserror::Error;

pub mod parse;

#[cfg(feature = "derive")]
pub use config_parser_derive::{ConfigNode, ConfigValue};
pub use parsey::Spanned;

pub type Result<T, E = ConfigError> = std::result::Result<T, E>;

#[derive(Debug, Error, Diagnostic)]
pub enum ConfigError {
    #[error("Syntax error")]
    Syntax {
        #[diagnostic_source]
        inner: SyntaxError,
    },

    #[error("Expected type {expected} but found {found}")]
    Type {
        #[label("Expected {expected}")]
        span: SourceSpan,

        expected: ConfigValueType,
        found: ConfigValueType,
    },

    #[error("Missing child node: {node_name}.")]
    ExpectedChild {
        #[label("Parent")]
        parent: SourceSpan,
        node_name: Box<str>,
    },

    #[error("Missing property: {prop_name}.")]
    ExpectedProperty {
        #[label("On this node.")]
        node: SourceSpan,
        prop_name: Box<str>,
    },

    #[error("Expected at least {expected} argument(s), found {found}.")]
    ExpectedArgument {
        #[label("On this node.")]
        node: SourceSpan,

        expected: usize,
        found: usize,
    },

    #[error("Expected {expected} argument(s), found {found}.")]
    TooManyArguments {
        #[label("Superfluous argument.")]
        arg: SourceSpan,

        expected: usize,
        found: usize,
    },

    #[error("Unexpected node type. Available node types are: {expected:?}")]
    UnexpectedNodeExpect {
        #[label("Unexpected node")]
        span: SourceSpan,
        expected: &'static [&'static str],
    },

    #[error("Expected no more nodes but found one.")]
    UnexpectedNode {
        #[label("Unexpected node")]
        span: SourceSpan,
    },

    #[error("{message}")]
    Message {
        #[label("{message}")]
        span: SourceSpan,
        message: Cow<'static, str>,
    },
}
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

impl ConfigError {
    pub fn type_error(value: &Spanned<ConfigValue>, expected: ConfigValueType) -> Self {
        Self::Type {
            span: value.span.clone().into(),
            expected: expected,
            found: value.ty(),
        }
    }
    pub fn expected_child(parent: &ConfigNode, child: impl Into<Box<str>>) -> Self {
        Self::ExpectedChild {
            parent: parent.name.span.clone().into(),
            node_name: child.into(),
        }
    }
    pub fn expected_property(node: &ConfigNode, property: impl Into<Box<str>>) -> Self {
        Self::ExpectedProperty {
            node: node.name.span.clone().into(),
            prop_name: property.into(),
        }
    }
    pub fn unexpected_node(node: &ConfigNode, expected: &'static [&'static str]) -> Self {
        if expected.len() == 0 {
            Self::UnexpectedNode {
                span: node.name.span.clone().into(),
            }
        } else {
            Self::UnexpectedNodeExpect {
                span: node.name.span.clone().into(),
                expected,
            }
        }
    }
    pub fn message(span: impl Into<Span>, message: impl Into<Cow<'static, str>>) -> Self {
        Self::Message {
            span: span.into().into(),
            message: message.into(),
        }
    }

    pub fn is_expect_item_error(&self) -> bool {
        match self {
            Self::ExpectedChild { .. } => true,
            Self::ExpectedProperty { .. } => true,
            Self::ExpectedArgument { .. } => true,
            _ => false,
        }
    }
}

pub trait ParseConfigValue<'c>: Sized {
    fn consume_value(value: Spanned<ConfigValue<'c>>) -> Result<Self>;
}

pub trait ParseConfigNode<'c>: Sized {
    fn match_node(node: &ConfigNode<'c>) -> bool;
    fn consume_node(node: &mut ConfigNode<'c>, terminate: bool) -> Result<Self>;
}

// impls

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
