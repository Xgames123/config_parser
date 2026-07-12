use crate::{
    AllowedNodeNames, ConfigError, ConfigValue, Document, ParseConfigNode, ParseConfigValue,
    Spanned,
};
use starryparse::Span;

#[derive(Debug, PartialEq)]
pub struct ConfigNode<'c> {
    pub(crate) arguments: Vec<Spanned<ConfigValue<'c>>>,
    pub(crate) argument_count: usize,
    pub(crate) properties: Vec<Option<(&'c str, Spanned<ConfigValue<'c>>)>>,
    pub(crate) children: Vec<Option<ConfigNode<'c>>>,
    pub(crate) name: Spanned<&'c str>,
}
impl<'c> ConfigNode<'c> {
    pub fn new(name: &'c str) -> Self {
        Self {
            name: Spanned::null_span(name),
            argument_count: 0,
            arguments: vec![],
            properties: vec![],
            children: vec![],
        }
    }

    pub fn from_document(document: Document<'c>) -> Self {
        let mut node = ConfigNode::new("document");
        node.children = document.nodes;
        node
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

    pub fn name(&self) -> &'c str {
        &self.name
    }
    pub fn name_spanned(&self) -> Spanned<&'c str> {
        self.name.clone()
    }
    pub fn name_span(&self) -> Span {
        self.name.span.clone()
    }

    pub fn children(&self) -> impl Iterator<Item = &ConfigNode<'c>> {
        self.children.iter().filter_map(|c| c.as_ref())
    }
    pub fn properties(&self) -> impl Iterator<Item = (&'c str, &Spanned<ConfigValue<'c>>)> {
        self.properties
            .iter()
            .filter_map(|p| p.as_ref().map(|(n, p)| (*n, p)))
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
        node_names: AllowedNodeNames<impl Iterator<Item = &'static str> + Clone>,
    ) -> impl Iterator<Item = Self> {
        self.children.iter_mut().filter_map(move |child| {
            if let Some(child_node) = child {
                if node_names.clone().is_allowed(child_node.name()) {
                    return child.take();
                }
            }
            None
        })
    }
    pub fn consume_children_into<T: ParseConfigNode<'c>, O: FromIterator<T>>(
        &mut self,
    ) -> Result<O, ConfigError> {
        self.consume_children_matching(T::allowed_node_names())
            .map(|mut n| ParseConfigNode::consume_node(&mut n, true))
            .collect::<Result<O, ConfigError>>()
    }

    pub fn consume_optional_child_matching(
        &mut self,
        node_names: AllowedNodeNames<impl Iterator<Item = &'static str> + Clone>,
    ) -> Option<ConfigNode<'c>> {
        self.consume_children_matching(node_names).next()
    }

    pub fn consume_optional_child_into<T: ParseConfigNode<'c>>(
        &mut self,
        terminate: bool,
    ) -> Result<Option<T>, ConfigError> {
        let Some(mut child) = self.consume_optional_child_matching(T::allowed_node_names()) else {
            return Ok(None);
        };

        Ok(Some(ParseConfigNode::consume_node(&mut child, terminate)?))
    }
    pub fn consume_child_into<T: ParseConfigNode<'c>>(
        &mut self,
        terminate: bool,
    ) -> Result<T, ConfigError> {
        Ok(self
            .consume_optional_child_into(terminate)?
            .ok_or(ConfigError::expected_children(
                &self,
                T::allowed_node_names(),
            ))?)
    }

    pub fn consume_optional_property(&mut self, name: &str) -> Option<Spanned<ConfigValue<'c>>> {
        let Some(index) = self.properties.iter().position(|entry| {
            let prop_name = entry.as_ref().map(|(name, _)| *name);
            prop_name == Some(name)
        }) else {
            return None;
        };
        Some(self.properties[index].take().unwrap().1)
    }
    pub fn consume_optional_property_into<T: ParseConfigValue<'c>>(
        &mut self,
        name: &str,
    ) -> Result<Option<T>, ConfigError> {
        let Some(prop) = self.consume_optional_property(name) else {
            return Ok(None);
        };
        Ok(Some(T::consume_value(prop)?))
    }

    pub fn consume_property(
        &mut self,
        name: &str,
    ) -> Result<Spanned<ConfigValue<'c>>, ConfigError> {
        self.consume_optional_property(name)
            .ok_or(ConfigError::expected_property(self, name))
    }

    pub fn consume_optional_argument(&mut self) -> Option<Spanned<ConfigValue<'c>>> {
        self.arguments.pop()
    }
    pub fn consume_optional_argument_into<T: ParseConfigValue<'c>>(
        &mut self,
    ) -> Result<Option<T>, ConfigError> {
        let Some(arg) = self.consume_optional_argument() else {
            return Ok(None);
        };

        Ok(Some(T::consume_value(arg)?))
    }

    pub fn consume_argument(&mut self) -> Result<Spanned<ConfigValue<'c>>, ConfigError> {
        self.consume_optional_argument()
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

    /// Terminates this node.
    /// After a node is terminated it can't be consumed further. If the node was not empty an error
    /// will be thrown.
    pub fn terminate(&mut self) -> Result<(), ConfigError> {
        if let Some(c) = self.children().next() {
            return Err(ConfigError::unexpected_node(
                c,
                AllowedNodeNames::<()>::empty(),
            ));
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
}
