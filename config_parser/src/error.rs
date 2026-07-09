use crate::{AllowedNodeNames, ConfigNode, ConfigValue, ConfigValueType, parse::SyntaxError};
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

    #[error("Expected one of the following node types are: {node_names}")]
    ExpectedChildren {
        #[label("Parent")]
        parent: SourceSpan,
        node_names: Box<str>,
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

    #[error("Unexpected node type. Available node types are: {node_names}")]
    UnexpectedNodeExpect {
        #[label("Unexpected node")]
        span: SourceSpan,
        node_names: Box<str>,
    },

    #[error("Unexpected node.")]
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
    pub fn expected_children(
        parent: &ConfigNode,
        children: AllowedNodeNames<impl Iterator<Item = &'static str> + Clone>,
    ) -> Self {
        Self::ExpectedChildren {
            parent: parent.name_span().into(),
            node_names: children.to_string().into(),
        }
    }
    pub fn expected_property(node: &ConfigNode, property: impl Into<Box<str>>) -> Self {
        Self::ExpectedProperty {
            node: node.name_span().into(),
            prop_name: property.into(),
        }
    }
    pub fn expected_argument(node: &ConfigNode) -> Self {
        ConfigError::ExpectedArgument {
            node: node.name_span().into(),
            expected: node.argument_count + 1,
            found: node.argument_count,
        }
    }

    pub fn unexpected_node(
        node: &ConfigNode,
        expected: AllowedNodeNames<impl Iterator<Item = &'static str> + Clone>,
    ) -> Self {
        if expected.clone().is_empty() {
            Self::UnexpectedNode {
                span: node.name_span().into(),
            }
        } else {
            Self::UnexpectedNodeExpect {
                span: node.name_span().into(),
                node_names: expected.to_string().into(),
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
            Self::ExpectedChildren { .. } => true,
            Self::ExpectedProperty { .. } => true,
            Self::ExpectedArgument { .. } => true,
            _ => false,
        }
    }
}
