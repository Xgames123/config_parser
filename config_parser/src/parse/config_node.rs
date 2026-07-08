use parsey::{Parsey, Spanned};

use crate::{
    ConfigNode, ConfigValue,
    parse::{
        Result, SyntaxError,
        parse_utils::{ident, skip_space_and_comments},
    },
};

impl<'c> ConfigNode<'c> {
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

#[cfg(test)]
mod test {
    use parsey::{Parsey, Spanned};

    use crate::{ConfigNode, ConfigValue};

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
