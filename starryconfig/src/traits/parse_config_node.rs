use crate::{AllowedNodeNames, ConfigNode, Result, Spanned};

pub trait ParseConfigNode<'c>: Sized {
    /// Node names which this node can be parsed from.
    fn allowed_node_names() -> AllowedNodeNames<impl Iterator<Item = &'static str> + Clone>;

    /// Consumes the node into this type.
    ///
    /// The parameter `terminate` indicates if the node should be terminated by the function.
    /// After a node is terminated it can't be consumed further anymore and an error is thrown if it
    /// was not fully consumed.
    fn consume_node(node: &mut ConfigNode<'c>, terminate: bool) -> Result<Self>;
}

// impls for common types

impl<'c, T: ParseConfigNode<'c>> ParseConfigNode<'c> for Spanned<T> {
    fn allowed_node_names() -> AllowedNodeNames<impl Iterator<Item = &'static str> + Clone> {
        T::allowed_node_names()
    }
    fn consume_node(node: &mut ConfigNode<'c>, terminate: bool) -> Result<Self> {
        let span = node.name_spanned().span.clone();
        Ok(Spanned::new(T::consume_node(node, terminate)?, span))
    }
}
impl<'c, T: ParseConfigNode<'c>> ParseConfigNode<'c> for Option<T> {
    fn allowed_node_names() -> AllowedNodeNames<impl Iterator<Item = &'static str> + Clone> {
        T::allowed_node_names()
    }
    fn consume_node(node: &mut ConfigNode<'c>, terminate: bool) -> Result<Self> {
        Ok(Some(T::consume_node(node, terminate)?))
    }
}
impl<'c> ParseConfigNode<'c> for ConfigNode<'c> {
    fn allowed_node_names() -> AllowedNodeNames<impl Iterator<Item = &'static str> + Clone> {
        AllowedNodeNames::<()>::any()
    }
    fn consume_node(node: &mut ConfigNode<'c>, _: bool) -> Result<Self> {
        Ok(node.clone())
    }
}
