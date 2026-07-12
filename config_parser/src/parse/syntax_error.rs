use std::num::{ParseFloatError, ParseIntError};

use miette::{Diagnostic, SourceSpan};
use starryparse::Parser;
use thiserror::Error;

use crate::parse::parse_utils::is_whitespace;

pub type Result<T, E = SyntaxError> = std::result::Result<T, E>;

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
    pub fn expected<'c>(span: &mut Parser<'c>, expected: &'static str) -> Self {
        Self::Expected {
            span: span.take_until_or_end(|c| is_whitespace(c)).span().into(),
            expected,
        }
    }
    pub fn expected_but_got<'c>(
        span: &mut Parser<'c>,
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
