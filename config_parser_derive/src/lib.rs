use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{
    Attribute, Data, DeriveInput, Fields, Generics, Ident, WherePredicate, parse_macro_input,
};

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

    let mut generics = input.generics;
    gen_where_generics(&input.attrs, &mut generics)?;
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let impl_generics = &generics.params;

    let (match_node, self_init) = gen_code(&name, &input.attrs, &input.data)?;

    Ok(quote! {
        impl<'c, #impl_generics> config_parser::ParseConfigNode<'c> for #name #ty_generics #where_clause {
            fn match_node(node: &config_parser::parse::ConfigNode<'c>) -> bool {
                #match_node
            }

            fn consume_node(node: &mut config_parser::parse::ConfigNode<'c>, terminate: bool) -> config_parser::Result<Self> {
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

    let mut generics = input.generics;
    gen_where_generics(&input.attrs, &mut generics)?;
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let impl_generics = &generics.params;

    let code = gen_value_code(input.data)?;

    Ok(quote! {
        impl<'c, #impl_generics> config_parser::ParseConfigValue<'c> for #name #ty_generics #where_clause {
            fn consume_value(value: config_parser::parse::Spanned<config_parser::parse::ConfigValue<'c>>) -> config_parser::Result<Self> {
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
                match value.inner.as_str().ok_or(config_parser::ConfigError::type_error(&value, config_parser::parse::ConfigValueType::String))? {
                    #(#option_names => Ok(#option_constructors),)*
                    val=>Err(config_parser::ConfigError::message(value.span, format!(concat!("Invalid value '{}'. Valid options are: ", #(#option_names),*), val)))
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

fn gen_where_generics(attributes: &[Attribute], generics: &mut Generics) -> syn::Result<()> {
    let where_c = generics.make_where_clause();

    for attr in attributes.iter() {
        if attr.path().is_ident("config") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("impl_where") {
                    let added_predicates =
                        Punctuated::<WherePredicate, Comma>::parse_terminated(meta.value()?)?;
                    where_c.predicates.extend(added_predicates);
                    Ok(())
                } else {
                    Err(meta.error("Only impl_where attribute is valid on a struct/enum"))
                }
            })?;
        }
    }
    Ok(())
}

fn gen_code(
    name: &Ident,
    attributes: &[Attribute],
    data: &Data,
) -> syn::Result<(TokenStream, TokenStream)> {
    match data {
        Data::Struct(data) => {
            let variant_attributes = VariantAttributes::parse(attributes)?;
            let node_name_expr = variant_attributes.node_name_expr(&name);
            let self_init = gen_constructor(&data.fields)?;
            Ok((quote!(#node_name_expr), quote!(Self #self_init)))
        }
        Data::Enum(e) => {
            let variant_node_name_exprs = e
                .variants
                .iter()
                .map(|v| {
                    VariantAttributes::parse(&v.attrs).map(|vattr| vattr.node_name_expr(&v.ident))
                })
                .collect::<syn::Result<Vec<TokenStream>>>()?;

            let variants = e
                .variants
                .iter()
                .zip(variant_node_name_exprs)
                .map(|(variant, node_name_expr)| {
                    let variant_ident = &variant.ident;
                    let self_init = gen_constructor(&variant.fields)?;
                    Ok(quote! {
                        else if #node_name_expr {
                            return Ok(Self::#variant_ident #self_init)
                        }
                    })
                })
                .collect::<syn::Result<Vec<TokenStream>>>()?;

            Ok((
                quote! {
                    false #(|| #variant_node_name_exprs )*
                },
                quote! {
                    if false {}
                    #(#variants)*
                    else {
                        return Err(config_parser::ConfigError::unexpected_node(node, &[#(#variant_names),*]))
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
                let FieldAttributes {
                    handeling,
                    rename,
                    default,
                } = FieldAttributes::parse(&field.attrs)?;
                let default_handeling = default.gen_code();
                let field_ty = &field.ty;
                let field = field.ident.as_ref().unwrap();
                let field_name = rename.unwrap_or_else(|| field.to_string());

                field_inits.push(match handeling {
                    FieldHandeling::Child => {
                        quote! {#field: node.consume_optional_child_into::<#field_ty>(#field_name)?#default_handeling }
                    }
                    FieldHandeling::Children => {
                        quote! {#field: node.consume_children_into::<_, #field_ty>(#field_name)?}
                    }
                    FieldHandeling::Property => {
                        quote! {#field: node.consume_property(#field_name).map(|val| config_parser::ParseConfigValue::consume_value(val))#default_handeling? }
                    }
                    FieldHandeling::Argument => {
                        quote! {#field: node.consume_argument().map(|val| config_parser::ParseConfigValue::consume_value(val))#default_handeling? }
                    },
                    FieldHandeling::Arguments => {
                        quote! {#field: node.consume_arguments_into::<_, #field_ty>()? }
                    }
                    FieldHandeling::Flatten => {
                        quote! {#field: config_parser::ParseConfigNode::consume_node(node, false)?}
                    }
                    FieldHandeling::Skip => {
                        quote! {#field: std::default::Default::default()}
                    },
                    FieldHandeling::NodeName => {
                        quote!{#field: node.name.inner.into() }
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
                    Ok(quote! {(config_parser::ParseConfigNode::consume_node(node, true)?)})
                }
            }
        }
        Fields::Unit => Ok(quote! {}),
    }
}
