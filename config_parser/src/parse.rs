use std::{
    fmt::Display,
    num::{ParseFloatError, ParseIntError},
};

use miette::{Diagnostic, SourceSpan};
use parsey::{Parsey, Spanned, parse_any};
use thiserror::Error;

use crate::{ConfigError, ParseConfigNode, ParseConfigValue};

#[derive(Error, Debug, Diagnostic, PartialEq, Eq)]
pub enum SyntaxError {
    #[error("Expected {expected}")]
    Expected {
        #[label("Expected {expected}")]
        span: SourceSpan,

        expected: &'static str,
    },

    #[error("Expected {expected} but got {got}")]
    ExpectedButGot {
        #[label("Expected {expected} but got {got}")]
        span: SourceSpan,

        expected: &'static str,
        got: &'static str,

        #[help]
        help: Option<&'static str>,
    },

    #[error("Invalid float value: {parse_error}")]
    InvalidFloat {
        #[label("Here")]
        span: SourceSpan,
        parse_error: ParseFloatError,
    },

    #[error("Invalid integer value: {parse_error}")]
    InvalidInt {
        #[label("Here")]
        span: SourceSpan,
        parse_error: ParseIntError,
    },

    #[error("Ident strings can only contain the characters: a-z_-.")]
    IdentStringFailed {
        #[label("Invalid character {char} in ident string")]
        invalid_char: SourceSpan,
        char: char,
    },

    #[error("No closing bracket found")]
    CurliesWrong {
        #[label("Opening bracket")]
        opening: SourceSpan,
    },
}
impl SyntaxError {
    pub fn expected<'c>(span: &mut Parsey<'c>, expected: &'static str) -> Self {
        Self::Expected {
            span: span.take_until_or_end(|c| is_whitespace(c)).span().into(),
            expected,
        }
    }
    pub fn expected_but_got<'c>(
        span: &mut Parsey<'c>,
        expected: &'static str,
        got: &'static str,
        help: Option<&'static str>,
    ) -> Self {
        Self::ExpectedButGot {
            span: span.take_until_or_end(|c| is_whitespace(c)).span().into(),
            expected,
            got,
            help,
        }
    }
}

pub type Result<T, E = SyntaxError> = std::result::Result<T, E>;

fn is_whitespace(char: char) -> bool {
    char.is_ascii_whitespace()
}

const PUNCT: [char; 7] = ['=', ',', '{', '}', '[', ']', '"'];

fn skip_space_and_comments(parser: &mut Parsey, skip_newlines: bool) -> Result<()> {
    if skip_newlines {
        parser.take_until_or_end(|c| !is_whitespace(c));
    } else {
        parser.take_until_or_end(|c| !is_whitespace(c) || c == '\n');
    }

    if let Some(_) = parser.take("/*") {
        parser.take_until_inclusive("*/");
        skip_space_and_comments(parser, skip_newlines)?;
    } else if let Some(_) = parser.take("//") {
        parser.take_until_or_end(|c| c == '\n');

        // put the cursor on the end of the line comment if we don't want to skip newlines
        if !skip_newlines {
            return Ok(());
        }
        skip_space_and_comments(parser, skip_newlines)?;
    }
    Ok(())
}

fn ident<'c>(parser: &mut Parsey<'c>) -> Result<Option<Parsey<'c>>> {
    let ident = parser.take_until_or_end(|c| is_whitespace(c) || PUNCT.contains(&c));
    if ident.str().len() == 0 {
        return Ok(None);
    }
    Ok(Some(ident))
}

#[derive(Debug, PartialEq)]
pub struct Document<'c> {
    pub nodes: Vec<Option<ConfigNode<'c>>>,
}
impl<'c> Document<'c> {
    pub fn new(nodes: impl IntoIterator<Item = ConfigNode<'c>>) -> Self {
        Self {
            nodes: nodes.into_iter().map(|n| Some(n)).collect(),
        }
    }

    /// Note: no miette source code is attached to the error
    pub fn from_str(str: &'c str) -> Result<Self, ConfigError> {
        let mut parser = Parsey::new(str);
        Document::parse(&mut parser).map_err(|e| ConfigError::Syntax { inner: e })
    }

    pub fn parse_into<T: ParseConfigNode<'c>>(self) -> Result<T, ConfigError> {
        T::consume_node(&mut self.into_node(), true)
    }
    pub fn into_node(self) -> ConfigNode<'c> {
        let mut node = ConfigNode::new("document");
        node.children = self.nodes;
        node
    }

    pub fn parse(parser: &mut Parsey<'c>) -> Result<Self> {
        let mut nodes = Vec::new();
        skip_space_and_comments(parser, true)?;

        while !parser.end() {
            let Some(node) = ConfigNode::parse(parser)? else {
                return Err(SyntaxError::expected(parser, "a node"));
            };
            nodes.push(Some(node));
            skip_space_and_comments(parser, true)?;
        }
        Ok(Document { nodes })
    }
}

#[derive(Debug, PartialEq)]
pub struct ConfigNode<'c> {
    pub name: Spanned<&'c str>,
    arguments: Vec<Spanned<ConfigValue<'c>>>,
    argument_count: usize,
    properties: Vec<Option<(&'c str, Spanned<ConfigValue<'c>>)>>,
    children: Vec<Option<ConfigNode<'c>>>,
}
impl<'c> ConfigNode<'c> {
    pub fn new(name: &'c str) -> Self {
        Self {
            argument_count: 0,
            name: Spanned::null_span(name),
            arguments: vec![],
            properties: vec![],
            children: vec![],
        }
    }
    pub fn with_child(mut self, child: ConfigNode<'c>) -> Self {
        self.children.push(Some(child));
        self
    }
    pub fn with_prop(mut self, name: &'c str, value: ConfigValue<'c>) -> Self {
        self.properties
            .push(Some((name, Spanned::null_span(value))));
        self
    }
    pub fn with_arg(mut self, value: ConfigValue<'c>) -> Self {
        self.arguments.push(Spanned::null_span(value));
        self.argument_count += 1;
        self
    }

    pub fn eq_no_span(&self, other: &ConfigNode) -> bool {
        if self.name.inner != other.name.inner {
            return false;
        }
        for (prop, value) in self.properties() {
            if other.get_property(prop).map(|v| v.inner).as_ref() != Some(&value.inner) {
                return false;
            }
        }

        for (a1, a2) in self.arguments.iter().zip(other.arguments.iter()) {
            if a1.inner != a2.inner {
                return false;
            }
        }

        for (c1, c2) in self.children().zip(other.children()) {
            if !c1.eq_no_span(c2) {
                return false;
            }
        }
        true
    }

    pub fn children(&self) -> impl Iterator<Item = &ConfigNode<'c>> {
        self.children.iter().filter_map(|c| c.as_ref())
    }
    pub fn properties(&self) -> impl Iterator<Item = (&'c str, &Spanned<ConfigValue<'c>>)> {
        self.properties
            .iter()
            .filter_map(|p| p.as_ref().map(|(n, p)| (*n, p)))
    }

    pub fn get_property(&self, name: &str) -> Option<Spanned<ConfigValue<'c>>> {
        for (prop, value) in self.properties.iter().filter_map(|c| c.as_ref()) {
            if *prop == name {
                return Some(value.clone());
            }
        }
        None
    }
    pub fn consume_children_matching(
        &mut self,
        mut f: impl FnMut(&ConfigNode<'c>) -> bool,
    ) -> impl Iterator<Item = Self> {
        self.children.iter_mut().filter_map(move |child| {
            if let Some(child_node) = child {
                if f(&child_node) {
                    return child.take();
                }
            }
            None
        })
    }
    pub fn consume_children_into<I: ParseConfigNode<'c>, O: FromIterator<I>>(
        &mut self,
        name: &str,
    ) -> Result<O, ConfigError> {
        self.consume_children_matching(|c| I::match_node(&c))
            .map(|mut n| ParseConfigNode::consume_node(&mut n, true))
            .collect::<Result<O, ConfigError>>()
    }

    pub fn consume_child_optional(&mut self, name: &str) -> Option<Self> {
        let Some(index) = self
            .children
            .iter()
            .position(|c| c.as_ref().map(|c| c.name.inner) == Some(name))
        else {
            return None;
        };
        Some(self.children[index].take().unwrap())
    }
    pub fn consume_child(&mut self, name: &str) -> Result<Self, ConfigError> {
        self.consume_child_optional(name)
            .ok_or(ConfigError::expected_child(&self, name))
    }

    pub fn consume_property_optional(&mut self, name: &str) -> Option<Spanned<ConfigValue<'c>>> {
        let Some(index) = self
            .properties
            .iter()
            .position(|prop| prop.as_ref().map(|(n, _)| *n) == Some(name))
        else {
            return None;
        };
        Some(self.properties[index].take().unwrap().1)
    }

    pub fn consume_property(
        &mut self,
        name: &str,
    ) -> Result<Spanned<ConfigValue<'c>>, ConfigError> {
        self.consume_property_optional(name)
            .ok_or(ConfigError::expected_property(self, name))
    }

    pub fn consume_argument_optional(&mut self) -> Option<Spanned<ConfigValue<'c>>> {
        self.arguments.pop()
    }

    pub fn consume_argument(&mut self) -> Result<Spanned<ConfigValue<'c>>, ConfigError> {
        self.consume_argument_optional()
            .ok_or(ConfigError::ExpectedArgument {
                node: self.name.span.clone().into(),
                expected: self.argument_count + 1,
                found: self.argument_count,
            })
    }
    pub fn consume_arguments_into<I: ParseConfigValue<'c>, O: FromIterator<I>>(
        &mut self,
    ) -> Result<O, ConfigError> {
        self.arguments
            .drain(..)
            .map(|arg| ParseConfigValue::consume_value(arg))
            .collect::<Result<O, ConfigError>>()
    }

    pub fn terminate(&mut self) -> Result<(), ConfigError> {
        if let Some(c) = self.children().next() {
            return Err(ConfigError::unexpected_node(c, &[]));
        }
        if let Some(arg) = self.arguments.iter().next() {
            return Err(ConfigError::TooManyArguments {
                arg: arg.span.clone().into(),
                expected: self.argument_count - self.arguments.len(),
                found: self.argument_count,
            });
        }
        Ok(())
    }

    pub fn parse(parser: &mut Parsey<'c>) -> Result<Option<Self>> {
        skip_space_and_comments(parser, true)?;

        let Some(name) = ident(parser)? else {
            return Ok(None);
        };

        let mut arguments = Vec::new();
        let mut properties = Vec::new();
        let mut argument_mode = true;
        loop {
            skip_space_and_comments(parser, false)?;

            // end the parsing of properties and arguments when any of the following characters
            // are encountered or end of parser.
            //
            // `node { child }` child ends with }
            if parser
                .peek_char()
                .map(|c| ['{', '}', '\n'].contains(&c))
                .unwrap_or(true)
            {
                break;
            }

            match parser.sandbox_result(Self::parse_property)? {
                Some(prop) => {
                    argument_mode = false;
                    properties.push(Some(prop));
                }
                None => {
                    match ConfigValue::parse(parser)? {
                        Some(v) => {
                            if argument_mode {
                                arguments.push(v)
                            } else {
                                return Err(SyntaxError::ExpectedButGot {
                                    span: v.span.into(),
                                    expected: "a property",
                                    got: "an argument",
                                    help: Some("Arguments need to be defined before properties"),
                                });
                            }
                        }
                        None => {
                            if argument_mode {
                                return Err(SyntaxError::expected(
                                    parser,
                                    "an argument or property",
                                ));
                            } else {
                                return Err(SyntaxError::expected(parser, "a property"));
                            }
                        }
                    }
                    if argument_mode {
                    } else {
                        return Err(SyntaxError::expected(parser, "a property"));
                    }
                }
            }
        }
        skip_space_and_comments(parser, false)?;

        let mut children = Vec::new();
        if let Some(opening_curl) = parser.take("{") {
            loop {
                skip_space_and_comments(parser, true)?;
                if parser.end() {
                    return Err(SyntaxError::CurliesWrong {
                        opening: opening_curl.span().into(),
                    });
                }
                if let Some(_) = parser.take("}") {
                    break;
                }

                let Some(node) = ConfigNode::parse(parser)? else {
                    return Err(SyntaxError::expected(parser, "a node"));
                };
                children.push(Some(node));
            }
        }

        Ok(Some(Self {
            name: Spanned::new(name.str(), name.span()),
            argument_count: arguments.len(),
            arguments,
            properties,
            children,
        }))
    }

    fn parse_property(
        parser: &mut Parsey<'c>,
    ) -> Result<Option<(&'c str, Spanned<ConfigValue<'c>>)>> {
        skip_space_and_comments(parser, false)?;
        let Some(name) = ident(parser)? else {
            return Ok(None);
        };
        skip_space_and_comments(parser, false)?;
        if let None = parser.take("=") {
            return Ok(None);
        }

        Ok(Some((
            name.str(),
            ConfigValue::parse(parser)?.ok_or(SyntaxError::expected(parser, "a property value"))?,
        )))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigValueType {
    String,
    Bool,
    Float,
    Int,
}
impl Display for ConfigValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::String => "string",
            Self::Bool => "bool",
            Self::Float => "float",
            Self::Int => "int",
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue<'c> {
    String(&'c str),
    Bool(bool),
    Float(f64),
    Int(i64),
}
impl<'c> ConfigValue<'c> {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Int(i) => Some(*i as f64),
            Self::Float(f) => Some(*f),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }
    pub fn ty(&self) -> ConfigValueType {
        match self {
            Self::String(_) => ConfigValueType::String,
            Self::Bool(_) => ConfigValueType::Bool,
            Self::Float(_) => ConfigValueType::Float,
            Self::Int(_) => ConfigValueType::Int,
        }
    }
}
impl<'c> ConfigValue<'c> {
    pub fn parse(parser: &mut Parsey<'c>) -> Result<Option<Spanned<Self>>> {
        skip_space_and_comments(parser, false)?;

        parse_any!(
            parser,
            Self::parse_number,
            Self::parse_bool,
            Self::parse_string,
            Self::parse_ident_string
        )
    }

    fn parse_number(parser: &mut Parsey<'c>) -> Result<Option<Spanned<Self>>> {
        parser.sandbox_result(|parser| {
            let number = parser.take_until_or_end(|c| is_whitespace(c));

            if !number
                .str()
                .starts_with(|c: char| c == '.' || c.is_ascii_digit())
            {
                return Ok(None);
            }

            if number.str().contains('.') {
                let float = number
                    .str()
                    .parse::<f64>()
                    .map_err(|e| SyntaxError::InvalidFloat {
                        span: number.span().into(),
                        parse_error: e,
                    })?;
                Ok(Some(Spanned::new(Self::Float(float), number.span())))
            } else {
                let mut radix = 10;
                let mut num_no_radix = number.fork();
                if let Some(_) = num_no_radix.take("0x") {
                    radix = 16;
                } else if let Some(_) = num_no_radix.take("0b") {
                    radix = 2;
                }

                let int = i64::from_str_radix(num_no_radix.str(), radix).map_err(|e| {
                    SyntaxError::InvalidInt {
                        span: number.span().into(),
                        parse_error: e,
                    }
                })?;

                Ok(Some(Spanned::new(Self::Int(int), number.span())))
            }
        })
    }

    fn parse_bool(parser: &mut Parsey<'c>) -> Result<Option<Spanned<Self>>> {
        parser.sandbox_result(|parser| {
            let ident = parser.take_until_or_end(|c| is_whitespace(c));
            match ident.str() {
                "true" | "#true" => Ok(Some(Self::Bool(true))),
                "false" | "#false" => Ok(Some(Self::Bool(false))),
                _ => Ok(None),
            }
            .map(|v| v.map(|v| Spanned::new(v, ident.span())))
        })
    }

    fn parse_string(parser: &mut Parsey<'c>) -> Result<Option<Spanned<Self>>> {
        parser.sandbox_result(|parser| {
            if let None = parser.take("\"") {
                return Ok(None);
            }
            let str = parser
                .take_until(|c| c == '"')
                .ok_or(SyntaxError::Expected {
                    span: parser.span().into(),
                    expected: "string ending quote (\")",
                })?;
            parser.take_n(1); // ending "

            Ok(Some(Spanned::new(Self::String(str.str()), str.span())))
        })
    }

    fn parse_ident_string(parser: &mut Parsey<'c>) -> Result<Option<Spanned<Self>>> {
        parser.sandbox_result(|parser| {
            let string = parser.take_until_or_end(|c| is_whitespace(c));
            if string.str().len() == 0 {
                return Ok(None);
            }
            for (i, char) in string.str().char_indices() {
                if !char.is_ascii_alphanumeric() && !['_', '-'].contains(&char) {
                    let char_len = char.len_utf8();
                    return Err(SyntaxError::IdentStringFailed {
                        invalid_char: (string.span().start() + i, char_len).into(),
                        char,
                    });
                }
            }
            Ok(Some(Spanned::new(
                Self::String(string.str()),
                string.span(),
            )))
        })
    }
}

#[cfg(test)]
mod test {
    use parsey::Parsey;

    use crate::{ConfigNode, ConfigValue, parse::Spanned};

    #[test]
    fn is_whitespace() {
        assert!(super::is_whitespace('\n'));
    }

    #[test]
    fn skip_space_and_comments_no_newlines() {
        let mut parser = Parsey::new("  //hello\nyow");
        assert_eq!(super::skip_space_and_comments(&mut parser, false), Ok(()));
        assert_eq!(super::skip_space_and_comments(&mut parser, false), Ok(()));
        assert_eq!(parser.str(), "\nyow");
    }

    #[test]
    fn skip_space_and_comments() {
        let mut parser = Parsey::new("  //hello\n\n");
        assert_eq!(super::skip_space_and_comments(&mut parser, true), Ok(()));
        assert_eq!(parser.str(), "");
    }

    #[test]
    fn parse_number() {
        assert_eq!(
            ConfigValue::parse_number(&mut Parsey::new("0xff")),
            Ok(Some(Spanned::new(ConfigValue::Int(0xFF), 0..4)))
        );

        assert_eq!(
            ConfigValue::parse_number(&mut Parsey::new("0b0101")),
            Ok(Some(Spanned::new(ConfigValue::Int(0b0101), 0..6)))
        );

        assert_eq!(
            ConfigValue::parse_number(&mut Parsey::new("67")),
            Ok(Some(Spanned::new(ConfigValue::Int(67), 0..2)))
        );

        assert_eq!(
            ConfigValue::parse_number(&mut Parsey::new("1.3")),
            Ok(Some(Spanned::new(ConfigValue::Float(1.3), 0..3)))
        );
    }

    #[test]
    fn parse_node() {
        let code = "test_node";

        assert_eq!(
            ConfigNode::parse(&mut Parsey::new(code)),
            Ok(Some(ConfigNode {
                name: Spanned::new("test_node", 0..9),
                argument_count: 0,
                arguments: vec![],
                properties: vec![],
                children: vec![],
            }))
        );
    }

    #[test]
    fn parse_node_with_prop() {
        let code = "test_node /* next is an argument */ 1.3";

        assert_eq!(
            ConfigNode::parse(&mut Parsey::new(code)),
            Ok(Some(ConfigNode {
                name: Spanned::new("test_node", 0..9),
                arguments: vec![Spanned::new(ConfigValue::Float(1.3), 36..39)],
                argument_count: 1,
                properties: vec![],
                children: vec![],
            }))
        );
    }
}
