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

fn one_field_name(f: syn::Fields) -> proc_macro2::TokenStream {
    let join_prefix = create_join_prefix();
    match f {
        syn::Fields::Named(ref fields) => {
            let f: Vec<_> = fields.named.clone().into_iter().collect();
            let names = f.iter().map(|x| snake_case_to_kebab(&x.ident.clone().unwrap().to_string()));
            let types = f.iter().map(|x| x.ty.clone());
            quote! {
                {
                    let mut flagname: Option<String> = None;
                    let join_prefix = #join_prefix;
                    #(
                        let thisname = join_prefix(#names);
                        let reqs = <#types as auto_args::AutoArgs>::requires_flags(&thisname);
                        if let Some(x) = reqs.first() {
                            flagname = Some(x.clone());
                        }
                    )*
                    flagname.expect("enum must have one required field!")
                }
            }
        },
        syn::Fields::Unit => {
            quote!{
                _name.to_string()
            }
        },
        syn::Fields::Unnamed(ref unnamed) => {
            let f = unnamed.unnamed.iter().next().expect("we should have one field");
            let mytype = f.ty.clone();
            quote!{{
                let reqs = <#mytype as auto_args::AutoArgs>::requires_flags(&_name);
                if let Some(x) = reqs.first() {
                    x.clone()
                } else {
                    panic!("enum {:?} must have one required field!", _name)
                }
            }}
        },
    }
}

fn return_with_fields(f: syn::Fields,
                      name: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let join_prefix = create_join_prefix();
    match f {
        syn::Fields::Named(ref fields) => {
            let f: Vec<_> = fields.named.clone().into_iter().collect();
            let names = f.iter().map(|x| snake_case_to_kebab(&x.ident.clone().unwrap().to_string()));
            let types = f.iter().map(|x| x.ty.clone());
            let idents = f.iter().map(|x| x.ident.clone().unwrap());
            quote! {
                let join_prefix = #join_prefix;
                return Ok( #name {
                    #( #idents:
                        <#types as auto_args::AutoArgs>::parse_internal(&join_prefix(#names),
                                                                        args)?,  )*
                });
            }
        },
        syn::Fields::Unit => {
            quote!( return Ok( #name ); )
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

fn create_join_prefix() -> proc_macro2::TokenStream {
    quote!{
        move |name: &str| -> String {
            if name.len() == 0 {
                let mut x = _prefix.to_string();
                x.pop();
                x
            } else {
                format!("{}{}", _prefix, name)
            }
        }
    }
}
fn create_find_prefix() -> proc_macro2::TokenStream {
    quote!{
        match key.chars().next() {
            None | Some('_') => "--".to_string(),
            _ => format!("{}-", key),
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
    let join_prefix = create_join_prefix();
    let myimpl = match input.data {
        Struct(DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }) => {
            let f: Vec<_> = fields.named.clone().into_iter().collect();
            let types3 = f.iter().rev().map(|x| x.ty.clone());
            let names3 = f.iter().rev().map(|x| snake_case_to_kebab(&x.ident.clone().unwrap().to_string()));
            let return_struct = return_with_fields(syn::Fields::Named(fields.clone()),
                                                   quote!(#name));
            quote!{
                fn parse_internal(key: &str, args: &mut Vec<OsString>)
                                  -> Result<Self, Error> {
                    let _prefix = #find_prefix;
                    #return_struct
                }
                fn tiny_help_message(key: &str) -> String {
                    "fixme".to_string()
                }
            }
        },
        Struct(DataStruct {
            fields: syn::Fields::Unit,
            ..
        }) => {
            quote!{
                fn with_clap<AutoArgsT>(mut _info: auto_args::ArgInfo,
                                app: auto_args::clap::App,
                                f: impl FnOnce(auto_args::clap::App) -> AutoArgsT)
                                      -> AutoArgsT {
                    f(app)
                }
                fn from_clap<'a,'b>(_name: &str, _matches: &auto_args::clap::ArgMatches) -> Option<Self> {
                    Some( #name )
                }
                fn requires_flags(_name: &str) -> Vec<String> {
                    Vec::new()
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
            let return_struct = return_with_fields(syn::Fields::Unnamed(unnamed.clone()),
                                                   quote!(#name));
            let f = unnamed.unnamed.iter().next().expect("There should be a field here!");
            let mytype = f.ty.clone();
            quote!{
                fn with_clap<AutoArgsT>(mut info: auto_args::ArgInfo,
                                app: auto_args::clap::App,
                                f: impl FnOnce(auto_args::clap::App) -> AutoArgsT)
                                      -> AutoArgsT {
                    let _name = info.name;
                    f(app)
                }
                fn from_clap<'a,'b>(_name: &str, _matches: &auto_args::clap::ArgMatches) -> Option<Self> {
                    #return_struct
                }
                fn requires_flags(_name: &str) -> Vec<String> {
                    <#mytype as auto_args::AutoArgs>::requires_flags(_name)
                }
            }
        },
        Enum(ref e) => {
            let v: Vec<_> = e.variants.iter().collect();
            let vnames: Vec<_> = e.variants.iter().map(|v| camel_case_to_kebab(&v.ident.to_string())).collect();
            let only_one_variant = vnames.len() == 1;
            // If only_one_variant is true, this is a special case,
            // and the code below won't work, because required_unless
            // logic fails when the list of "unless" fields is empty.
            // Really, we should treat this thing as a struct with an
            // additional layer of prefixing going on.
            let vnames2 = vnames.clone();
            let vnames3 = vnames.clone();
            let vnames4 = vnames.clone();
            let vnames5 = vnames.clone();
            let vnames6 = vnames.clone();
            // println!("variant names are {:?}", names);
            let fields: Vec<_> = v.iter().map(|x| x.fields.clone()).collect();
            let one_field: Vec<_> = fields.iter().map(|f| one_field_name(f.clone())).collect();
            let one_field2 = one_field.clone();
            let one_field3 = one_field.clone();
            let return_enum = v.iter().map(|v| {
                let variant_name = v.ident.clone();
                return_with_fields(v.fields.clone(), quote!(#name::#variant_name))
            });
            let find_prefix = create_find_prefix();
            let s = quote! {
                fn parse_internal(key: &str, args: &mut Vec<OsString>)
                                  -> Result<Self, Error>
                {
                    let _prefix = #find_prefix;

                    #(
                        {
                            let variant = #vnames;
                            let _prefix = match _prefix.chars().next() {
                                None | Some('_') | Some('-') => format!("--{}", variant),
                                _ => format!("{}-{}", _prefix, variant),
                            };
                            #return_enum
                        }

                    )*
                    Err(auto_args::Error::MissingOption("a missing thing".to_string()))
                }
                fn tiny_help_message(key: &str) -> String {
                    "fixme".to_string()
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
        impl#bounds auto_args::AutoArgs for #name#generics {
            #myimpl
        }
    };
    println!("\n\n{}", tokens2);
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
