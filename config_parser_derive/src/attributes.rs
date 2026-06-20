use proc_macro2::TokenStream;
use syn::{Attribute, LitStr};
use quote::quote;


pub struct FieldAttributes {
    pub handeling: FieldHandeling,
    pub rename: Option<String>,
    pub default: DefaultHandeling,
}
impl FieldAttributes {
    pub fn parse<'a>(attrs: impl IntoIterator<Item = &'a Attribute>) -> Result<Self, syn::Error> {
        let mut handeling = None;
        let mut rename = None;
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
                        Some(FieldHandeling::Property)
                    }else if meta.path.is_ident("argument") {
                        Some(FieldHandeling::Argument)
                    }else if meta.path.is_ident("arguments") {
                        Some(FieldHandeling::Arguments)
                    }else if meta.path.is_ident("skip") {
                        Some(FieldHandeling::Skip)
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

                    if meta.path.is_ident("rename") {
                        rename = Some(meta.value()?.parse::<LitStr>()?.value());
                        return Ok(());
                    }
                    if meta.path.is_ident("default") {
                        default = DefaultHandeling::DefaultTrait;
                        return Ok(());
                    }
                    Err(meta.error("Unknown attribute valid attributes are: child, children, property, argument, arguments, rename, default and flatten"))
                })?;
            }
        }
        Ok(FieldAttributes { handeling: handeling.unwrap_or(FieldHandeling::Property), rename, default })
    }
}

#[derive(Debug)]
pub enum FieldHandeling {
    Child,
    Children,
    Property,
    Argument,
    Arguments,
    Flatten,
    Skip,
}

pub enum DefaultHandeling {
    Error,
    DefaultTrait,
}
impl DefaultHandeling {
    pub fn gen_code(self) -> TokenStream {
        match self {
            Self::Error => quote! {?},
            Self::DefaultTrait => quote! {.unwrap_or_else(|_|Ok(std::default::Default::default()))}
        }
    }
}
