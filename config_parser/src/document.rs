use parsey::Parsey;

use crate::{ConfigError, ConfigNode, ParseConfigNode, Result};

#[derive(Debug, PartialEq)]
pub struct Document<'c> {
    pub(crate) nodes: Vec<Option<ConfigNode<'c>>>,
}
impl<'c> Document<'c> {
    pub fn new(nodes: impl IntoIterator<Item = ConfigNode<'c>>) -> Self {
        Self {
            nodes: nodes.into_iter().map(|n| Some(n)).collect(),
        }
    }

    /// Note: You need to manually attach the miette source code to the error returned by this function.
    ///```rust
    ///# #[derive(config_parser::ConfigNode)]
    ///# struct MyType { }
    ///# let source_code = "";
    ///
    /// let parsed : MyType = config_parser::from_str(source_code).unwrap_or_else(|e| {
    ///     panic!(
    ///         "{:?}",
    ///         miette::Report::from(e).with_source_code(source_code.to_string())
    ///     )
    /// });
    /// # let parsed : MyType = parsed;
    ///```
    pub fn from_str(str: &'c str) -> Result<Self> {
        let mut parser = Parsey::new(str);
        Document::parse(&mut parser).map_err(|e| ConfigError::Syntax { inner: e })
    }

    pub fn parse_into<T: ParseConfigNode<'c>>(self) -> Result<T> {
        T::consume_node(&mut self.into_node(), true)
    }
    pub fn into_node(self) -> ConfigNode<'c> {
        ConfigNode::from_document(self)
    }
}
