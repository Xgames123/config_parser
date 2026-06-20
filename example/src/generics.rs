use config_parser::{ConfigNode, ParseConfigNode};

/// This can be used to wrap a type and give it a name property
///
/// This can be parsed as a `Named<MyNode>`
///```kdl
/// my-node name="hello" other-property=true
/// ```
#[derive(Debug, ConfigNode)]
#[config(impl_where=T: ParseConfigNode<'c>)]
struct Named<T> {
    #[config(property)]
    name: String,

    #[config(flatten)]
    t: T,
}

#[derive(Debug, ConfigNode)]
struct MyNode {
    #[config(property)]
    other_property: String,
}
