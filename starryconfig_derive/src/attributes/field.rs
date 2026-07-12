use proc_macro2::TokenStream;
use syn::{Attribute, LitStr, parenthesized, token::Comma};
use quote::quote;


pub struct FieldAttributes {
    pub handeling: FieldHandeling,
    pub default: DefaultHandeling,
}
impl FieldAttributes {
    pub fn parse<'a>(attrs: impl IntoIterator<Item = &'a Attribute>) -> Result<Self, syn::Error> {
        let mut handeling = None;
        let mut default = DefaultHandeling::Error;
        for attr in attrs.into_iter() {
            if attr.path().is_ident("config") {
                attr.parse_nested_meta(|meta| {

                    let new_handeling = 
                    if meta.path.is_ident("child") {
                        Some(FieldHandeling::Child)
                    } else if meta.path.is_ident("children") {
                        Some(FieldHandeling::Children)
                    }else if meta.path.is_ident("flatten") {
                        Some(FieldHandeling::Flatten)
                    }else if meta.path.is_ident("property") {
                        if meta.input.is_empty() || meta.input.lookahead1().peek(Comma) {
                            Some(FieldHandeling::Property(None))
                        }else {
                            let content;
                            parenthesized!(content in meta.input);
                            let str = content.parse::<LitStr>()?;
                            Some(FieldHandeling::Property(Some(str.value().into())))
                        }
                    }else if meta.path.is_ident("argument") {
                        Some(FieldHandeling::Argument)
                    }else if meta.path.is_ident("arguments") {
                        Some(FieldHandeling::Arguments)
                    }else if meta.path.is_ident("skip") {
                        Some(FieldHandeling::Skip)
                    }else if meta.path.is_ident("node_name") {
                        Some(FieldHandeling::NodeName)
                    } else if meta.path.is_ident("node_name_spanned") {
                        Some(FieldHandeling::NodeNameSpanned)
                    } else { None };

                    if let Some(new_handeling) = new_handeling {
                        return match &handeling {
                            Some(handeling) => Err(meta.error(format!("{:?} {:?} can't be used on the same field", new_handeling, handeling))),
                            None => {
                                handeling = Some(new_handeling);
                                Ok(())
                            }
                        };
                    }

                    if meta.path.is_ident("default") {
                        default = DefaultHandeling::DefaultTrait;
                        return Ok(());
                    }
                    Err(meta.error("Unknown attribute valid attributes are: child, children, property, property(\"prop name\"), argument, arguments, default, flatten and node_name"))
                })?;
            }
        }
        Ok(FieldAttributes { handeling: handeling.unwrap_or(FieldHandeling::Property(None)),  default })
    }
}

#[derive(Debug)]
pub enum FieldHandeling {
    Child,
    Children,
    Property(Option<Box<str>>),
    Argument,
    Arguments,
    Flatten,
    Skip,
    NodeName,
    NodeNameSpanned
}

pub enum DefaultHandeling {
    Error,
    DefaultTrait,
}
impl DefaultHandeling {
    /// Generates code to handle an Option<T>
    pub fn gen_code(self) -> TokenStream {
        match self {
            Self::Error => quote! {?},
            Self::DefaultTrait => quote! {.unwrap_or_else(|_|std::default::Default::default())}
        }
    }
}
