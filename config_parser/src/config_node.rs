use crate::{ConfigError, ConfigValue, Document, ParseConfigNode, ParseConfigValue, Spanned};

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
    pub fn consume_children_into<T: ParseConfigNode<'c>, O: FromIterator<T>>(
        &mut self,
    ) -> Result<O, ConfigError> {
        self.consume_children_matching(|c| T::match_node(&c))
            .map(|mut n| ParseConfigNode::consume_node(&mut n, true))
            .collect::<Result<O, ConfigError>>()
    }

    pub fn consume_optional_child_matching(
        &mut self,
        f: impl FnMut(&ConfigNode<'c>) -> bool,
    ) -> Option<ConfigNode<'_>> {
        self.consume_children_matching(f).next()
    }

    pub fn consume_optional_child_into<T: ParseConfigNode<'c>>(
        &'c mut self,
        terminate: bool,
    ) -> Result<Option<T>, ConfigError> {
        let Some(mut child) = self.consume_optional_child_matching(|c| T::match_node(c)) else {
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
            .ok_or(ConfigError::expected_child(&self, name))?)
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

    /// Terminates this node.
    /// After a node is terminated it can't be consumed further. If the node was not empty an error
    /// will be thrown.
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
}
