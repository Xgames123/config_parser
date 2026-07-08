use crate::{ConfigNode, ConfigValue, ConfigValueType, parse::SyntaxError};
use miette::{Diagnostic, SourceSpan};
use parsey::{Span, Spanned};
use std::borrow::Cow;
use thiserror::Error;

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
