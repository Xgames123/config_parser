use crate::{
    ConfigNode, Document,
    parse::{Result, SyntaxError, parse_utils::skip_space_and_comments},
};
use parsey::Parsey;

impl<'c> Document<'c> {
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
