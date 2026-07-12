#[derive(Clone)]
pub enum AllowedNodeNames<I> {
    Any,
    Iter(I),
}
impl<I: Iterator<Item = &'static str> + Clone> AllowedNodeNames<I> {
    pub fn is_allowed(self, name: &str) -> bool {
        match self {
            Self::Any => true,
            Self::Iter(mut iter) => iter.find(|n| *n == name).is_some(),
        }
    }
    pub fn is_empty(self) -> bool {
        match self {
            Self::Any => false,
            Self::Iter(mut i) => i.next().is_none(),
        }
    }

    pub fn combine(
        self,
        other: AllowedNodeNames<impl Iterator<Item = &'static str> + Clone>,
    ) -> AllowedNodeNames<impl Iterator<Item = &'static str> + Clone> {
        match (self, other) {
            (AllowedNodeNames::Any, _) => AllowedNodeNames::Any,
            (_, AllowedNodeNames::Any) => AllowedNodeNames::Any,
            (AllowedNodeNames::Iter(iter1), AllowedNodeNames::Iter(iter2)) => {
                AllowedNodeNames::Iter(iter1.chain(iter2))
            }
        }
    }
}
impl<I: Iterator<Item = &'static str> + Clone> ToString for AllowedNodeNames<I> {
    fn to_string(&self) -> String {
        let mut string = String::new();
        match self {
            Self::Any => string.push_str("any node"),
            Self::Iter(iter) => {
                for node_name in iter.clone() {
                    string.push_str(node_name);
                    string.push(',');
                }
                string.pop();
            }
        }
        string
    }
}
impl<I> AllowedNodeNames<I> {
    pub fn empty() -> AllowedNodeNames<impl Iterator<Item = &'static str> + Clone> {
        AllowedNodeNames::Iter(std::iter::empty::<&'static str>())
    }
    pub fn from_single(
        name: &'static str,
    ) -> AllowedNodeNames<impl Iterator<Item = &'static str> + Clone> {
        AllowedNodeNames::Iter(std::iter::once(name))
    }
    pub fn any() -> AllowedNodeNames<std::iter::Empty<&'static str>> {
        AllowedNodeNames::Any
    }
    pub fn from_slice(
        slice: &[&'static str],
    ) -> AllowedNodeNames<impl Iterator<Item = &'static str> + Clone> {
        AllowedNodeNames::Iter(slice.iter().map(|c| *c))
    }
}
