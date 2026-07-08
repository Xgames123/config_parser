use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Ident, LitStr, Type, meta::ParseNestedMeta};

use crate::pascal_to_kebab;

pub enum VariantName {
    Auto,
    Custom(Box<str>),
    FromType(Type),
}

pub struct VariantAttributes {
    pub name: VariantName,
}

impl VariantAttributes {
    /// Generates an expression that matches the node name to the node variable
    pub fn node_name_expr(&self, variant_name: &Ident) -> TokenStream {
        match &self.name {
            VariantName::Auto => {
                let name = pascal_to_kebab(variant_name.to_string());
                quote! {&node.name.inner == #name}
            }
            VariantName::Custom(name) => quote! {&node.name.inner == #name},
            VariantName::FromType(ty) => {
                quote! {<#ty as config_parser::ParseConfigNode::match_node(node)>}
            }
        }
    }

    pub fn parse<'a>(attrs: impl IntoIterator<Item = &'a Attribute>) -> Result<Self, syn::Error> {
        let mut name = VariantName::Auto;
        for attr in attrs.into_iter() {
            if attr.path().is_ident("config") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("node_name") {
                        name = Self::parse_node_name_attr(&meta)?;
                    } else if meta.path.is_ident("rename") {
                        let value = meta.value()?.parse::<LitStr>()?.value().into_boxed_str();
                        name = VariantName::Custom(value);
                    }
                    Err(meta.error(
                        "Unknown variant attribute valid attributes are: node_name and rename",
                    ))
                })?;
            }
        }
        Ok(VariantAttributes { name })
    }

    fn parse_node_name_attr<'a>(meta: &ParseNestedMeta<'a>) -> Result<VariantName, syn::Error> {
        let value = meta.value()?;
        if let Ok(str) = value.parse::<LitStr>() {
            return Ok(VariantName::Custom(str.value().into_boxed_str()));
        } else if let Ok(ident) = value.parse::<syn::Ident>() {
            if ident == "auto" {
                return Ok(VariantName::Auto);
            }
        }

        return Err(value.error(
            "Unknown node_name value. valid values are auto or a string with the new name",
        ));
    }
}
