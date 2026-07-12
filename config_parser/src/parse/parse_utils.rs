//! Utility functions that parse small parts of the syntax like identifiers or comments.
use crate::parse::Result;
use starryparse::Parser;

/// All characters that are considered punctuation.
pub const PUNCT_CHARS: [char; 7] = ['=', ',', '{', '}', '[', ']', '"'];

pub fn is_whitespace(char: char) -> bool {
    char.is_ascii_whitespace()
}

pub fn skip_space_and_comments(parser: &mut Parser, skip_newlines: bool) -> Result<()> {
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

pub fn ident<'c>(parser: &mut Parser<'c>) -> Result<Option<Parser<'c>>> {
    let ident = parser.take_until_or_end(|c| is_whitespace(c) || PUNCT_CHARS.contains(&c));
    if ident.str().len() == 0 {
        return Ok(None);
    }
    Ok(Some(ident))
}

#[cfg(test)]
mod test {
    use starryparse::Parser;

    #[test]
    fn is_whitespace() {
        assert!(super::is_whitespace('\n'));
    }

    #[test]
    fn skip_space_and_comments_no_newlines() {
        let mut parser = Parser::new("  //hello\nyow");
        assert_eq!(super::skip_space_and_comments(&mut parser, false), Ok(()));
        assert_eq!(super::skip_space_and_comments(&mut parser, false), Ok(()));
        assert_eq!(parser.str(), "\nyow");
    }

    #[test]
    fn skip_space_and_comments() {
        let mut parser = Parser::new("  //hello\n\n");
        assert_eq!(super::skip_space_and_comments(&mut parser, true), Ok(()));
        assert_eq!(parser.str(), "");
    }
}
