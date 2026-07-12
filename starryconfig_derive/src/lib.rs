use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, Ident, parse_macro_input};

use crate::{
    attributes::{field::FieldAttributes, field::FieldHandeling, variant::VariantAttributes},
    case::pascal_to_kebab,
};

mod attributes;
mod case;

#[proc_macro_derive(ConfigNode, attributes(config))]
pub fn derive_config_node(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = match impl_config_node(input) {
        Ok(v) => v,
        Err(e) => e.to_compile_error(),
    };
    //println!("{}", expanded.to_string());

    proc_macro::TokenStream::from(expanded)
}

fn impl_config_node(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = input.ident;
    let mut variant_attributes = VariantAttributes::parse(&input.attrs)?;

    let mut generics = input.generics;
    let where_clause = generics.make_where_clause();
    if let Some(impl_where) = variant_attributes.impl_where.take() {
        impl_where.extend_where_clause(where_clause);
    }
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let impl_generics = &generics.params;

    let (allowed_node_names, self_init) = gen_code(&name, variant_attributes, &input.data)?;

    Ok(quote! {
        impl<'c, #impl_generics> starryconfig::ParseConfigNode<'c> for #name #ty_generics #where_clause {

            fn allowed_node_names() -> starryconfig::AllowedNodeNames<impl Iterator<Item = &'static str>+Clone> {
                #allowed_node_names
            }

            fn consume_node(node: &mut starryconfig::ConfigNode<'c>, terminate: bool) -> starryconfig::Result<Self> {
                let me = #self_init;
                if terminate {
                    node.terminate()?;
                }
                Ok(me)
            }

        }
    })
}

#[proc_macro_derive(ConfigValue, attributes(config))]
pub fn derive_config_value(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = match impl_config_value(input) {
        Ok(v) => v,
        Err(e) => e.to_compile_error(),
    };
    //println!("{}", expanded.to_string());

    proc_macro::TokenStream::from(expanded)
}

fn impl_config_value(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = input.ident;
    let mut variant_attributes = VariantAttributes::parse(&input.attrs)?;

    let mut generics = input.generics;
    let where_clause = generics.make_where_clause();
    if let Some(impl_where) = variant_attributes.impl_where.take() {
        impl_where.extend_where_clause(where_clause);
    }
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let impl_generics = &generics.params;

    let code = gen_value_code(input.data)?;

    Ok(quote! {
        impl<'c, #impl_generics> starryconfig::ParseConfigValue<'c> for #name #ty_generics #where_clause {
            fn consume_value(value: starryconfig::Spanned<starryconfig::ConfigValue<'c>>) -> starryconfig::Result<Self> {
                #code
            }
        }
    })
}

fn gen_value_code(data: Data) -> syn::Result<TokenStream> {
    match data {
        Data::Enum(e) => {
            let mut option_constructors = Vec::with_capacity(e.variants.len());
            let mut option_names = Vec::with_capacity(e.variants.len());

            for var in e.variants {
                match &var.fields {
                    Fields::Unnamed(u) => {
                        if u.unnamed.len() == 0 {
                            let ident = &var.ident;
                            option_constructors.push(quote!(Self::#ident()));
                            option_names.push(pascal_to_kebab(ident.to_string()));
                        } else {
                            return Err(syn::Error::new(u.span(), "Enum can not contain fields"));
                        }
                    }
                    Fields::Unit => {
                        let ident = &var.ident;
                        option_constructors.push(quote!(Self::#ident));
                        option_names.push(pascal_to_kebab(ident.to_string()));
                    }
                    Fields::Named(f) => {
                        return Err(syn::Error::new(f.span(), "Enum can not contain fields"));
                    }
                }
            }

            Ok(quote! {
                match value.inner.as_str().ok_or(starryconfig::ConfigError::type_error(&value, starryconfig::ConfigValueType::String))? {
                    #(#option_names => Ok(#option_constructors),)*
                    val=>Err(starryconfig::ConfigError::message(value.span, format!(concat!("Invalid value '{}'. Valid options are: ", #(#option_names),*), val)))
                }
            })
        }
        Data::Struct(_) => todo!("impl flattening of nested value in tuple structs with 1 item"),
        Data::Union(u) => Err(syn::Error::new(
            u.fields.span(),
            "Only tuple struct or tuple enums are supported",
        )),
    }
}

fn gen_code(
    name: &Ident,
    variant_attributes: VariantAttributes,
    data: &Data,
) -> syn::Result<(TokenStream, TokenStream)> {
    match data {
        Data::Struct(data) => {
            let node_name = variant_attributes.node_names(&name);
            let self_init = gen_constructor(&data.fields)?;
            Ok((quote!(#node_name), quote!(Self #self_init)))
        }
        Data::Enum(e) => {
            let variant_node_names = e
                .variants
                .iter()
                .map(|v| VariantAttributes::parse(&v.attrs).map(|vattr| vattr.node_names(&v.ident)))
                .collect::<syn::Result<Vec<TokenStream>>>()?;

            let variants = e
                .variants
                .iter()
                .zip(variant_node_names.iter())
                .map(|(variant, node_name)| {
                    let variant_ident = &variant.ident;
                    let self_init = gen_constructor(&variant.fields)?;
                    Ok(quote! {
                        else if #node_name.is_allowed(node.name()) {
                            Self::#variant_ident #self_init
                        }
                    })
                })
                .collect::<syn::Result<Vec<TokenStream>>>()?;

            let variant_node_names = quote! {starryconfig::AllowedNodeNames::<()>::empty()#(.combine(#variant_node_names))*};

            Ok((
                quote! {
                    #variant_node_names
                },
                quote! {
                    if false { unreachable!() }
                    #(#variants)*
                    else {
                        return Err(starryconfig::ConfigError::unexpected_node(node, Self::allowed_node_names()))
                    }
                },
            ))
        }
        Data::Union(_) => unimplemented!("No one uses unions anyway"),
    }
}

fn gen_constructor(fields: &Fields) -> syn::Result<TokenStream> {
    match fields {
        Fields::Named(fields) => {
            let mut field_inits = Vec::new();
            for field in fields.named.iter() {
                let FieldAttributes { handeling, default } = FieldAttributes::parse(&field.attrs)?;
                let default_handeling = default.gen_code();
                let field_ty = &field.ty;
                let field = field.ident.as_ref().unwrap();

                field_inits.push(match handeling {
                    FieldHandeling::Child => {
                        quote! {#field: node.consume_optional_child_into::<#field_ty>(true)?.ok_or(starryconfig::ConfigError::expected_children(node, <#field_ty>::allowed_node_names()))#default_handeling }
                    }
                    FieldHandeling::Children => {
                        quote! {#field: node.consume_children_into::<_, #field_ty>()?}
                    }
                    FieldHandeling::Property(prop_name) => {
                        let field_name = prop_name.unwrap_or_else(|| field.to_string().into());
                        quote! {#field: node.consume_optional_property_into::<#field_ty>(#field_name)?.ok_or(starryconfig::ConfigError::expected_property(node, #field_name))#default_handeling }
                    }
                    FieldHandeling::Argument => {
                        quote! {#field: node.consume_optional_argument_into::<#field_ty>()?.ok_or(starryconfig::ConfigError::expected_argument(node))#default_handeling }
                    },
                    FieldHandeling::Arguments => {
                        quote! {#field: node.consume_arguments_into::<_, #field_ty>()? }
                    }
                    FieldHandeling::Flatten => {
                        quote! {#field: starryconfig::ParseConfigNode::consume_node(node, false)?}
                    }
                    FieldHandeling::Skip => {
                        quote! {#field: std::default::Default::default()}
                    },
                    FieldHandeling::NodeName => {
                        quote!{#field: node.name().into() }
                    },
                    FieldHandeling::NodeNameSpanned => {
                        quote!{#field: node.name_spanned().into() }
                    }
                })
            }

            Ok(quote! {
                {
                    #(#field_inits),*
                }
            })
        }
        Fields::Unnamed(unnamed) => {
            let first_field = unnamed.unnamed.iter().next();
            match first_field {
                None => Ok(quote! {()}),
                Some(_) => {
                    if unnamed.unnamed.len() > 1 {
                        return Err(syn::Error::new(
                            unnamed.unnamed.span(),
                            "Only 1 field allowed in Tuple structs",
                        ));
                    }
                    Ok(quote! {(starryconfig::ParseConfigNode::consume_node(node, true)?)})
                }
            }
        }
        Fields::Unit => Ok(quote! {}),
    }
}
