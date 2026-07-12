use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, Ident, LitStr, Type, WhereClause, WherePredicate, meta::ParseNestedMeta,
    parenthesized, punctuated::Punctuated, token::Comma,
};

use crate::pascal_to_kebab;

pub enum VariantName {
    FromVariant,
    Custom(Box<str>),
    FromType(Type),
    Any,
}

/// The content of impl_where()
pub struct ImplWhere {
    added_predicates: Punctuated<syn::WherePredicate, Comma>,
}
impl ImplWhere {
    pub fn extend_where_clause(self, clause: &mut WhereClause) {
        clause.predicates.extend(self.added_predicates);
    }
}

pub struct VariantAttributes {
    pub name: VariantName,
    pub impl_where: Option<ImplWhere>,
}

impl VariantAttributes {
    /// Generates an expression that matches the node name to the node variable
    pub fn node_names(&self, variant_name: &Ident) -> TokenStream {
        match &self.name {
            VariantName::Any => {
                quote! {starryconfig::AllowedNodeNames::<()>::any()}
            }
            VariantName::FromVariant => {
                let name = pascal_to_kebab(variant_name.to_string());
                quote! {starryconfig::AllowedNodeNames::<()>::from_single(#name)}
            }
            VariantName::Custom(name) => {
                quote! {starryconfig::AllowedNodeNames::<()>::from_single(#name)}
            }
            VariantName::FromType(ty) => {
                quote! {<#ty as starryconfig::ParseConfigNode>::allowed_node_names()}
            }
        }
    }

    pub fn parse<'a>(attrs: impl IntoIterator<Item = &'a Attribute>) -> Result<Self, syn::Error> {
        let mut name = VariantName::FromVariant;
        let mut impl_where = None;
        for attr in attrs.into_iter() {
            if attr.path().is_ident("config") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("impl_where") {
                        let content;
                        parenthesized!(content in meta.input);

                        let added_predicates =
                            Punctuated::<WherePredicate, Comma>::parse_terminated(&content)?;
                        impl_where = Some(ImplWhere { added_predicates });
                        return Ok(());
                    } else if meta.path.is_ident("node_name") {
                        name = Self::parse_node_name_attr(&meta)?;
                        return Ok(());
                    }
                    Err(meta.error(
                        "Unknown variant attribute valid attributes are: impl_where, node_name",
                    ))
                })?;
            }
        }
        Ok(VariantAttributes { name, impl_where })
    }

    fn parse_node_name_attr<'a>(meta: &ParseNestedMeta<'a>) -> Result<VariantName, syn::Error> {
        let content;
        parenthesized!(content in meta.input);

        if let Ok(str) = content.parse::<LitStr>() {
            return Ok(VariantName::Custom(str.value().into_boxed_str()));
        }

        if let Ok(ty) = content.parse::<syn::Type>() {
            if let Type::Path(ref path) = ty {
                let path = &path.path;
                if path.is_ident("auto") {
                    return Ok(VariantName::FromVariant);
                } else if path.is_ident("any") {
                    return Ok(VariantName::Any);
                }
            }
            return Ok(VariantName::FromType(ty));
        }
        Err(content
            .error("Unknown node_name value. valid values are node_name=auto node_name=any node_name=\"new name\" node_name=MyForwardType"))
    }
}
