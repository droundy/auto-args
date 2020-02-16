// Copyright 2018 David Roundy <roundyd@physics.oregonstate.edu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This crate is custom derive for `AutoArgs`. It should not be used
//! directly.

#![recursion_limit="256"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro2;

use syn::*;

fn get_doc_comment(attrs: &[syn::Attribute]) -> String {
    let mut doc_comments: Vec<_> = attrs
        .iter()
        .filter_map(|attr| {
            let path = &attr.path;
            if quote!(#path).to_string() == "doc" {
                attr.interpret_meta()
            } else {
                None
            }
        })
        .filter_map(|attr| {
            use Lit::*;
            use Meta::*;
            if let NameValue(MetaNameValue {ident, lit: Str(s), ..}) = attr {
                if ident != "doc" {
                    return None;
                }
                let value = s.value();
                let text = value
                    .trim_start_matches("//!")
                    .trim_start_matches("///")
                    .trim_start_matches("/*!")
                    .trim_start_matches("/**")
                    .trim_end_matches("*/")
                    .trim();
                if text.is_empty() {
                    Some("\n\n".to_string())
                } else {
                    Some(text.to_string())
                }
            } else {
                None
            }
        })
        .collect();
    if doc_comments.len() > 0 {
        doc_comments.pop().unwrap_or("".to_string())
    } else {
        "".to_string()
    }
}

fn return_with_fields(f: syn::Fields,
                      name: proc_macro2::TokenStream,
                      am_enum_variant: bool) -> proc_macro2::TokenStream {
    let join_prefix = create_join_prefix();
    match f {
        syn::Fields::Named(ref fields) => {
            let f: Vec<_> = fields.named.clone().into_iter().collect();
            let names = f.iter().map(|x| snake_case_to_kebab(&x.ident.clone().unwrap().to_string()));
            let types = f.iter().map(|x| x.ty.clone());
            let types2 = types.clone();
            let idents = f.iter().map(|x| x.ident.clone().unwrap());
            let check_main_flag = if am_enum_variant {
                quote!{
                    if #( <#types2 as auto_args::AutoArgs>::REQUIRES_INPUT ||)* false {
                        // Nothing special to do, something below requires input.
                    } else if !bool::parse_internal(&_prefix, args)? {
                        return Err(auto_args::Error::MissingOption(_prefix.clone()));
                    }
                }
            } else {
                quote!{}
            };
            quote! {
                #check_main_flag
                let join_prefix = #join_prefix;
                return Ok( #name {
                    #( #idents:
                        <#types as auto_args::AutoArgs>::parse_internal(&join_prefix(#names),
                                                                        args)?,  )*
                });
            }
        },
        syn::Fields::Unit => {
            quote!{
                if bool::parse_internal(&_prefix, args)? {
                    return Ok( #name );
                } else {
                    return Err(auto_args::Error::MissingOption(_prefix.clone()));
                }
            }
        },
        syn::Fields::Unnamed(ref unnamed) if unnamed.unnamed.len() == 1 => {
            let f = unnamed.unnamed.iter().next().expect("we should have one field");
            let mytype = f.ty.clone();
            quote!{
                if let Ok(x) = <#mytype as auto_args::AutoArgs>::parse_internal(&_prefix, args) {
                    return Ok(#name(x));
                }
            }
        },
        _ => {
            panic!("AutoArgs only supports named fields so far!")
        },
    }
}

fn usage_with_fields(f: syn::Fields,
                     _name: proc_macro2::TokenStream,
                     am_enum_variant: bool) -> proc_macro2::TokenStream {
    let join_prefix = create_join_prefix();
    match f {
        syn::Fields::Named(ref fields) => {
            let f: Vec<_> = fields.named.clone().into_iter().collect();
            let names = f.iter().map(|x| snake_case_to_kebab(&x.ident.clone().unwrap().to_string()));
            let types = f.iter().map(|x| x.ty.clone());
            let types2 = types.clone();
            let check_main_flag = if am_enum_variant {
                quote!{
                    if #( <#types2 as auto_args::AutoArgs>::REQUIRES_INPUT ||)* false {
                        // Nothing special to do, something below requires input.
                    } else {
                        doc.push_str(&format!("{} ", _prefix));
                    }
                }
            } else {
                quote!{}
            };
            quote! {
                let mut doc = String::new();
                #check_main_flag
                let join_prefix = #join_prefix;
                #( doc.push_str(
                    &format!(" {}",
                             <#types as auto_args::AutoArgs>::tiny_help_message(&join_prefix(#names))));
                )*
                doc
            }
        },
        syn::Fields::Unit => {
            quote!( _prefix.clone() )
        },
        syn::Fields::Unnamed(ref unnamed) if unnamed.unnamed.len() == 1 => {
            let f = unnamed.unnamed.iter().next().expect("we should have one field");
            let mytype = f.ty.clone();
            quote!{
                <#mytype as auto_args::AutoArgs>::tiny_help_message(&_prefix)
            }
        },
        _ => {
            panic!("AutoArgs only supports named fields so far!")
        },
    }
}


fn help_with_fields(f: syn::Fields,
                    _name: proc_macro2::TokenStream,
                    am_enum_variant: bool) -> proc_macro2::TokenStream {
    let join_prefix = create_join_prefix();
    match f {
        syn::Fields::Named(ref fields) => {
            let f: Vec<_> = fields.named.clone().into_iter().collect();
            let docs: Vec<_> = f.iter().map(|x| get_doc_comment(&x.attrs)).collect();
            let names = f.iter().map(|x| snake_case_to_kebab(&x.ident.clone().unwrap().to_string()));
            let types = f.iter().map(|x| x.ty.clone());
            let types2 = types.clone();
            let check_main_flag = if am_enum_variant {
                quote!{
                    if #( <#types2 as auto_args::AutoArgs>::REQUIRES_INPUT ||)* false {
                        // Nothing special to do, something below requires input.
                    } else {
                        doc.push_str(&format!("{} {}", _prefix, variant_doc));
                    }
                }
            } else {
                quote!{}
            };
            quote! {
                let mut doc = String::new();
                #check_main_flag
                let join_prefix = #join_prefix;
                #( doc.push_str(
                    &<#types as auto_args::AutoArgs>::help_message(&join_prefix(#names),
                                                                   #docs));
                   if !doc.ends_with("\n") {
                       doc.push('\n');
                   }
                )*
                doc
            }
        },
        syn::Fields::Unit => {
            quote!( format!("\t{}\t{}\n", _prefix, variant_doc) )
        },
        syn::Fields::Unnamed(ref unnamed) if unnamed.unnamed.len() == 1 => {
            let f = unnamed.unnamed.iter().next().expect("we should have one field");
            let mytype = f.ty.clone();
            quote!{
                <#mytype as auto_args::AutoArgs>::help_message(&_prefix, &variant_doc)
            }
        },
        _ => {
            panic!("AutoArgs only supports named fields so far!")
        },
    }
}

fn create_join_prefix() -> proc_macro2::TokenStream {
    quote!{
        move |name: &str| -> String {
            if name.len() == 0 {
                let mut x = _prefix.to_string();
                // x.pop();
                x
            } else if _prefix.chars().last() == Some('-') {
                format!("{}{}", _prefix, name)
            } else {
                format!("{}-{}", _prefix, name)
            }
        }
    }
}
fn create_find_prefix() -> proc_macro2::TokenStream {
    quote!{
        match key.chars().next() {
            None | Some('_') => "--".to_string(),
            _ => match key.chars().last() {
                Some('-') => key.to_string(),
                _ => format!("{}-", key),
            }
        }
    }
}

/// Generates the `AutoArgs` impl.
#[proc_macro_derive(AutoArgs)]
pub fn auto_args(raw_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use syn::Data::*;
    let input: DeriveInput = syn::parse(raw_input.clone()).unwrap();

    let name = &input.ident;
    let generics = &input.generics;
    let find_prefix = create_find_prefix();
    let myimpl = match input.data {
        Struct(DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }) => {
            let f: Vec<_> = fields.named.clone().into_iter().collect();
            let types3 = f.iter().rev().map(|x| x.ty.clone());
            let return_struct = return_with_fields(syn::Fields::Named(fields.clone()),
                                                   quote!(#name), false);
            let usage_struct = usage_with_fields(syn::Fields::Named(fields.clone()),
                                                 quote!(#name), false);
            let help_struct = help_with_fields(syn::Fields::Named(fields.clone()),
                                               quote!(#name), false);
            quote!{
                const REQUIRES_INPUT: bool = #(
                    <#types3 as auto_args::AutoArgs>::REQUIRES_INPUT ||)* false;
                fn parse_internal(key: &str, args: &mut Vec<std::ffi::OsString>)
                                  -> Result<Self, auto_args::Error> {
                    let _prefix = #find_prefix;
                    #return_struct
                }
                fn tiny_help_message(key: &str) -> String {
                    let _prefix = #find_prefix;
                    #usage_struct
                }
                fn help_message(key: &str, _doc: &str) -> String {
                    let _prefix = #find_prefix;
                    #help_struct
                }
            }
        },
        Struct(DataStruct {
            fields: syn::Fields::Unit,
            ..
        }) => {
            quote!{
                const REQUIRES_INPUT: bool = false;
                fn parse_internal(key: &str, args: &mut Vec<std::ffi::OsString>)
                                  -> Result<Self, auto_args::Error> {
                    Ok( #name )
                }
                fn tiny_help_message(key: &str) -> String {
                    "".to_string()
                }
            }
        },
        Struct(DataStruct {
            fields: syn::Fields::Unnamed(ref unnamed),
            ..
        }) => {
            if unnamed.unnamed.len() != 1 {
                panic!("AutoArgs does not handle tuple structs with more than one field");
            }
            let f = unnamed.unnamed.iter().next().expect("There should be a field here!");
            let mytype = f.ty.clone();
            quote!{
                const REQUIRES_INPUT: bool =
                    <#mytype as auto_args::AutoArgs>::REQUIRES_INPUT;
                fn parse_internal(key: &str, args: &mut Vec<std::ffi::OsString>)
                                  -> Result<Self, auto_args::Error> {
                    <#mytype as auto_args::AutoArgs>::parse_internal(key, args)
                        .map(|x| #name(x))
                }
                fn tiny_help_message(key: &str) -> String {
                    "fixme unnamed".to_string()
                }
            }
        },
        Enum(ref e) => {
            let v: Vec<_> = e.variants.iter().collect();
            let vnames: Vec<_> = e.variants.iter().map(|v| camel_case_to_kebab(&v.ident.to_string())).collect();
            let variant_docs: Vec<_> = e.variants.iter().map(|v| get_doc_comment(&v.attrs)).collect();
            let vnames = &vnames;
            // println!("variant names are {:?}", names);
            let return_enum = v.iter().map(|v| {
                let variant_name = v.ident.clone();
                return_with_fields(v.fields.clone(), quote!(#name::#variant_name), true)
            });
            let helps = v.iter().map(|v| {
                let variant_name = v.ident.clone();
                help_with_fields(v.fields.clone(),
                                 quote!(#name::#variant_name), true)
            });
            let usages = v.iter().map(|v| {
                let variant_name = v.ident.clone();
                usage_with_fields(v.fields.clone(),
                                  quote!(#name::#variant_name), true)
            });
            let s = quote! {
                const REQUIRES_INPUT: bool = true;
                fn parse_internal(key: &str, args: &mut Vec<std::ffi::OsString>)
                                  -> Result<Self, auto_args::Error>
                {
                    let _prefix = match key.chars().next() {
                        None | Some('_') => "--".to_string(),
                        _ => match key.chars().last() {
                            Some('-') => key.to_string(),
                            _ => format!("{}-", key),
                        }
                    };
                    let orig_args = args;
                    #(
                        {
                            let mut args = orig_args.clone();
                            let args = &mut args;
                            let variant = #vnames;
                            let _prefix = format!("{}{}", _prefix, variant);
                            let mut closure = || -> Result<_, auto_args::Error> {
                                #return_enum
                                Err(auto_args::Error::MissingOption("ooo".to_string()))
                            };
                            if let Ok(v) = closure() {
                                *orig_args = args.clone();
                                return Ok(v);
                            }
                        }

                    )*
                    Err(auto_args::Error::MissingOption("a missing thing".to_string()))
                }
                fn help_message(key: &str, doc: &str) -> String {
                    let _prefix = match key.chars().next() {
                        None | Some('_') => "--".to_string(),
                        _ => match key.chars().last() {
                            Some('-') => key.to_string(),
                            _ => format!("{}-", key),
                        }
                    };
                    let mut doc = String::new();
                    doc.push_str("\tEITHER\t\n");
                    #(
                        {
                            let variant = #vnames;
                            let _prefix = format!("{}{}", _prefix, variant);
                            let variant_doc = #variant_docs;
                            doc.push_str(&{ #helps });
                            if !doc.ends_with("\n") {
                                doc.push_str("\n");
                            }
                            doc.push_str("\tOR\t\n");
                        }

                    )*
                    for _ in 0.."\tOR\t\n".len() {
                        doc.pop();
                    }
                    doc.push_str("\t\t");
                    doc
                }
                fn tiny_help_message(key: &str) -> String {
                    let _prefix = match key.chars().next() {
                        None | Some('_') => "--".to_string(),
                        _ => match key.chars().last() {
                            Some('-') => key.to_string(),
                            _ => format!("{}-", key),
                        }
                    };
                    let mut doc = String::new();
                    doc.push_str("( ");
                    #(
                        {
                            let variant = #vnames;
                            let _prefix = format!("{}{}", _prefix, variant);
                            doc.push_str(&{ #usages });
                            doc.push_str(" | ");
                        }

                    )*
                    for _ in 0..3 {
                        doc.pop();
                    }
                    doc.push_str(" )");
                    doc
                }
            };
            s
        },
        _ => panic!("AutoArgs only supports non-tuple structs"),
    };

    let generic_types = input.generics.type_params();
    let bounds = quote!{
        <#(#generic_types: auto_args::AutoArgs),*>
    };

    let tokens2: proc_macro2::TokenStream = quote!{
        #[allow(unreachable_code)]
        impl#bounds auto_args::AutoArgs for #name#generics {
            #myimpl
        }
    };
    // println!("\n\n{}", tokens2);
    tokens2.into()
}

fn camel_case_to_kebab(name: &str) -> String {
    if name.chars().next() == Some('_') {
        "".to_string()
    } else if name.contains('_') {
        let mut out = name.to_string().replace("_", "-");
        if out.chars().last() == Some('-') {
            out.pop();
        }
        out
    } else {
        let mut out = String::new();
        let mut am_on_cap = true;
        for c in name.chars() {
            if !am_on_cap && c.is_ascii_uppercase() {
                out.push('-');
            }
            am_on_cap = c.is_ascii_uppercase();
            out.push(c.to_ascii_lowercase());
        }
        out
    }
}

fn snake_case_to_kebab(name: &str) -> String {
    if name.chars().next() == Some('_') {
        "".to_string()
    } else {
        name.to_string().replace("_", "-")
    }
}
