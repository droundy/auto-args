// Copyright 2018 David Roundy <roundyd@physics.oregonstate.edu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This crate is custom derive for `AutoArgs`. It should not be used
//! directly.

#![recursion_limit = "256"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro2;

use syn::*;

fn get_doc_comment(attrs: &[syn::Attribute]) -> String {
    let mut doc_comments = attrs
        .iter()
        .filter_map(|attr| {
            let path = attr.path();
            if quote!(#path).to_string() == "doc" {
                Some(attr.meta.clone())
            } else {
                None
            }
        })
        .filter_map(|attr| {
            use Lit::*;
            use Meta::*;
            if let NameValue(MetaNameValue {
                path, value: Expr::Lit(ExprLit { lit: Str(s), .. }), ..
            }) = attr
            {
                if !path.is_ident("doc") {
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
        });

    doc_comments.next().unwrap_or_else(String::new)
}

/// Generates the body of `parse_internal` for a struct's named fields (or
/// an enum variant's named fields when used in "new style" subcommand
/// mode). `am_enum_variant` selects between two genuinely different
/// behaviors:
///
/// * `false` (a plain struct, or a subcommand-style enum variant): every
///   field is parsed in two phases. Phase 1 parses every field that is
///   *not* the struct's sole subcommand-shaped field (if it has exactly
///   one) as an ordinary flag, wherever it occurs in `args`. Phase 2
///   resolves the deferred field (if any) by looking at whatever is left
///   in `args`, using a bare/positional key if the struct itself was
///   reached in bare context (`_prefix == "--"`), or an ordinary
///   prefixed flag key otherwise (i.e. if some ancestor already fell
///   back to flag-style parsing).
/// * `true` (a legacy flag-style enum variant, used only for the
///   multi-enum-field fallback case): unchanged from the original
///   implementation — single-pass, prefixed flags, plus the
///   `check_main_flag` all-optional-fields disambiguation.
fn return_with_fields(
    f: syn::Fields,
    name: proc_macro2::TokenStream,
    am_enum_variant: bool,
) -> proc_macro2::TokenStream {
    let join_prefix = create_join_prefix();
    match f {
        syn::Fields::Named(ref fields) => {
            let f: Vec<_> = fields.named.clone().into_iter().collect();
            if am_enum_variant {
                let names = f
                    .iter()
                    .map(|x| snake_case_to_kebab(&x.ident.clone().unwrap().to_string()));
                let types = f.iter().map(|x| x.ty.clone());
                let types2 = types.clone();
                let idents = f.iter().map(|x| x.ident.clone().unwrap());
                let check_main_flag = quote! {
                    if #( <#types2 as auto_args::AutoArgs>::REQUIRES_INPUT ||)* false {
                        // Nothing special to do, something below requires input.
                    } else if !bool::parse_internal(&_prefix, args)? {
                        return Err(auto_args::Error::MissingOption(_prefix.clone()));
                    }
                };
                quote! {
                    #check_main_flag
                    let join_prefix = #join_prefix;
                    Ok( #name {
                        #( #idents:
                            <#types as auto_args::AutoArgs>::parse_internal(&join_prefix(#names),
                                                                            args)?,  )*
                    })
                }
            } else {
                let names_phase1 = f
                    .iter()
                    .map(|x| snake_case_to_kebab(&x.ident.clone().unwrap().to_string()));
                let names_phase2 = f
                    .iter()
                    .map(|x| snake_case_to_kebab(&x.ident.clone().unwrap().to_string()));
                let types_count = f.iter().map(|x| x.ty.clone());
                let types_phase1 = f.iter().map(|x| x.ty.clone());
                let types_phase2 = f.iter().map(|x| x.ty.clone());
                let idents_final = f.iter().map(|x| x.ident.clone().unwrap());
                let tmp_decl = (0..f.len())
                    .map(|i| quote::format_ident!("__auto_args_tmp_{}", i));
                let tmp_use1 = (0..f.len())
                    .map(|i| quote::format_ident!("__auto_args_tmp_{}", i));
                quote! {
                    let join_prefix = #join_prefix;
                    let __auto_args_subcommand_count: usize =
                        0usize #( + if <#types_count as auto_args::AutoArgs>::IS_SUBCOMMAND { 1 } else { 0 })*;
                    #(
                        let #tmp_decl: Option<#types_phase1> =
                            if __auto_args_subcommand_count == 1
                                && <#types_phase1 as auto_args::AutoArgs>::IS_SUBCOMMAND {
                                None
                            } else {
                                Some(<#types_phase1 as auto_args::AutoArgs>::parse_internal(
                                    &join_prefix(#names_phase1), args)?)
                            };
                    )*
                    Ok( #name {
                        #( #idents_final: match #tmp_use1 {
                            Some(v) => v,
                            None => <#types_phase2 as auto_args::AutoArgs>::parse_internal(
                                &(if _prefix == "--" { "".to_string() } else { join_prefix(#names_phase2) }),
                                args)?,
                        }, )*
                    })
                }
            }
        }
        syn::Fields::Unit => {
            quote! {
                if bool::parse_internal(&_prefix, args)? {
                    Ok( #name )
                } else {
                    Err(auto_args::Error::MissingOption(_prefix.clone()))
                }
            }
        }
        syn::Fields::Unnamed(ref unnamed) if unnamed.unnamed.len() == 1 => {
            let f = unnamed
                .unnamed
                .iter()
                .next()
                .expect("we should have one field");
            let mytype = f.ty.clone();
            quote! {
                <#mytype as auto_args::AutoArgs>::parse_internal(&_prefix, args).map(|x| #name(x))
            }
        }
        _ => {
            panic!("AutoArgs only supports named fields so far!")
        }
    }
}

fn usage_with_fields(
    f: syn::Fields,
    _name: proc_macro2::TokenStream,
    am_enum_variant: bool,
) -> proc_macro2::TokenStream {
    let join_prefix = create_join_prefix();
    match f {
        syn::Fields::Named(ref fields) => {
            let f: Vec<_> = fields.named.clone().into_iter().collect();
            let types = f.iter().map(|x| x.ty.clone());
            let types2 = types.clone();
            let check_main_flag = if am_enum_variant {
                quote! {
                    if #( <#types2 as auto_args::AutoArgs>::REQUIRES_INPUT ||)* false {
                        // Nothing special to do, something below requires input.
                    } else {
                        doc.push_str(&format!("{} ", _prefix));
                    }
                }
            } else {
                quote! {}
            };
            let names2 = f
                .iter()
                .map(|x| snake_case_to_kebab(&x.ident.clone().unwrap().to_string()));
            let types3 = f.iter().map(|x| x.ty.clone());
            let body = if am_enum_variant {
                quote! {
                    #( doc.push_str(
                        &format!(" {}",
                                 <#types3 as auto_args::AutoArgs>::tiny_help_message(&join_prefix(#names2))));
                    )*
                }
            } else {
                let types_count = f.iter().map(|x| x.ty.clone());
                quote! {
                    let __auto_args_subcommand_count: usize =
                        0usize #( + if <#types_count as auto_args::AutoArgs>::IS_SUBCOMMAND { 1 } else { 0 })*;
                    #( doc.push_str(
                        &format!(" {}",
                                 <#types3 as auto_args::AutoArgs>::tiny_help_message(
                                     &(if __auto_args_subcommand_count == 1
                                          && <#types3 as auto_args::AutoArgs>::IS_SUBCOMMAND {
                                         if _prefix == "--" { "".to_string() } else { join_prefix(#names2) }
                                     } else {
                                         join_prefix(#names2)
                                     }))));
                    )*
                }
            };
            quote! {
                let mut doc = String::new();
                #check_main_flag
                let join_prefix = #join_prefix;
                #body
                doc
            }
        }
        syn::Fields::Unit => {
            quote!(_prefix.clone())
        }
        syn::Fields::Unnamed(ref unnamed) if unnamed.unnamed.len() == 1 => {
            let f = unnamed
                .unnamed
                .iter()
                .next()
                .expect("we should have one field");
            let mytype = f.ty.clone();
            quote! {
                <#mytype as auto_args::AutoArgs>::tiny_help_message(&_prefix)
            }
        }
        _ => {
            panic!("AutoArgs only supports named fields so far!")
        }
    }
}

fn help_with_fields(
    f: syn::Fields,
    _name: proc_macro2::TokenStream,
    am_enum_variant: bool,
) -> proc_macro2::TokenStream {
    let join_prefix = create_join_prefix();
    match f {
        syn::Fields::Named(ref fields) => {
            let f: Vec<_> = fields.named.clone().into_iter().collect();
            let docs: Vec<_> = f.iter().map(|x| get_doc_comment(&x.attrs)).collect();
            let types = f.iter().map(|x| x.ty.clone());
            let types2 = types.clone();
            let check_main_flag = if am_enum_variant {
                quote! {
                    if #( <#types2 as auto_args::AutoArgs>::REQUIRES_INPUT ||)* false {
                        // Nothing special to do, something below requires input.
                    } else {
                        doc.push_str(&format!("\t{}\t{}\n", _prefix, variant_doc));
                    }
                }
            } else {
                quote! {}
            };
            let names2 = f
                .iter()
                .map(|x| snake_case_to_kebab(&x.ident.clone().unwrap().to_string()));
            let types3 = f.iter().map(|x| x.ty.clone());
            let body = if am_enum_variant {
                quote! {
                    #( doc.push_str(
                        &<#types3 as auto_args::AutoArgs>::help_message(&join_prefix(#names2),
                                                                       #docs));
                       if !doc.ends_with("\n") {
                           doc.push('\n');
                       }
                    )*
                }
            } else {
                let types_count = f.iter().map(|x| x.ty.clone());
                quote! {
                    let __auto_args_subcommand_count: usize =
                        0usize #( + if <#types_count as auto_args::AutoArgs>::IS_SUBCOMMAND { 1 } else { 0 })*;
                    #( doc.push_str(
                        &<#types3 as auto_args::AutoArgs>::help_message(
                            &(if __auto_args_subcommand_count == 1
                                 && <#types3 as auto_args::AutoArgs>::IS_SUBCOMMAND {
                                if _prefix == "--" { "".to_string() } else { join_prefix(#names2) }
                            } else {
                                join_prefix(#names2)
                            }),
                            #docs));
                       if !doc.ends_with("\n") {
                           doc.push('\n');
                       }
                    )*
                }
            };
            quote! {
                let mut doc = String::new();
                #check_main_flag
                let join_prefix = #join_prefix;
                #body
                doc
            }
        }
        syn::Fields::Unit => {
            quote!(format!("\t{}\t{}\n", _prefix, variant_doc))
        }
        syn::Fields::Unnamed(ref unnamed) if unnamed.unnamed.len() == 1 => {
            let f = unnamed
                .unnamed
                .iter()
                .next()
                .expect("we should have one field");
            let mytype = f.ty.clone();
            quote! {
                <#mytype as auto_args::AutoArgs>::help_message(&_prefix, &variant_doc)
            }
        }
        _ => {
            panic!("AutoArgs only supports named fields so far!")
        }
    }
}

fn create_join_prefix() -> proc_macro2::TokenStream {
    quote! {
        |name: &str| -> String {
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
    quote! {
        match key.chars().next() {
            None | Some('_') => "--".to_string(),
            _ => match key.chars().last() {
                Some('-') => key.to_string(),
                _ => format!("{}-", key),
            }
        }
    }
}

/// Produces the `Result<Self, auto_args::Error>`-valued expression used
/// to finish parsing a single subcommand-style enum variant, once its
/// keyword (if any) has already been consumed from `args`.
fn new_style_variant_body(f: &syn::Fields, name: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    match f {
        syn::Fields::Named(_) => {
            let body = return_with_fields(f.clone(), name, false);
            quote! {
                {
                    let _prefix = "--".to_string();
                    #body
                }
            }
        }
        syn::Fields::Unit => {
            quote! { Ok(#name) }
        }
        syn::Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1 => {
            let ty = unnamed
                .unnamed
                .iter()
                .next()
                .expect("we should have one field")
                .ty
                .clone();
            quote! {
                <#ty as auto_args::AutoArgs>::parse_internal("", args).map(|x| #name(x))
            }
        }
        _ => panic!("AutoArgs only supports named fields so far!"),
    }
}

/// A tiny one-line usage synopsis for a single subcommand-style variant's
/// own fields (used in the `SUBCOMMANDS` help block).
fn new_style_variant_usage(f: &syn::Fields, name: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    match f {
        syn::Fields::Named(_) => {
            let body = usage_with_fields(f.clone(), name, false);
            quote! {
                {
                    let _prefix = "--".to_string();
                    #body
                }
            }
        }
        syn::Fields::Unit => quote! { "".to_string() },
        syn::Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1 => {
            let ty = unnamed
                .unnamed
                .iter()
                .next()
                .expect("we should have one field")
                .ty
                .clone();
            quote! { <#ty as auto_args::AutoArgs>::tiny_help_message("").to_string() }
        }
        _ => panic!("AutoArgs only supports named fields so far!"),
    }
}

/// The detailed (multi-line) help for a single subcommand-style variant's
/// own fields.
fn new_style_variant_help(f: &syn::Fields, name: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    match f {
        syn::Fields::Named(_) => {
            let body = help_with_fields(f.clone(), name, false);
            quote! {
                {
                    let _prefix = "--".to_string();
                    #body
                }
            }
        }
        syn::Fields::Unit => quote! { String::new() },
        syn::Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1 => {
            let ty = unnamed
                .unnamed
                .iter()
                .next()
                .expect("we should have one field")
                .ty
                .clone();
            quote! { <#ty as auto_args::AutoArgs>::help_message("", "") }
        }
        _ => panic!("AutoArgs only supports named fields so far!"),
    }
}

/// A `bool`-valued expression: does this variant's own fields require any
/// input, i.e. can it match with zero further tokens?
fn requires_input_for_fields(f: &syn::Fields) -> proc_macro2::TokenStream {
    match f {
        syn::Fields::Named(fields) => {
            let types: Vec<_> = fields.named.iter().map(|x| x.ty.clone()).collect();
            quote! { ( #( <#types as auto_args::AutoArgs>::REQUIRES_INPUT ||)* false ) }
        }
        syn::Fields::Unit => quote! { false },
        syn::Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1 => {
            let ty = unnamed
                .unnamed
                .iter()
                .next()
                .expect("we should have one field")
                .ty
                .clone();
            quote! { <#ty as auto_args::AutoArgs>::REQUIRES_INPUT }
        }
        _ => panic!("AutoArgs only supports named fields so far!"),
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
            let types4 = f.iter().map(|x| x.ty.clone());
            let return_struct =
                return_with_fields(syn::Fields::Named(fields.clone()), quote!(#name), false);
            let usage_struct =
                usage_with_fields(syn::Fields::Named(fields.clone()), quote!(#name), false);
            let help_struct =
                help_with_fields(syn::Fields::Named(fields.clone()), quote!(#name), false);
            quote! {
                const REQUIRES_INPUT: bool = #(
                    <#types3 as auto_args::AutoArgs>::REQUIRES_INPUT ||)* false;
                const IS_SUBCOMMAND: bool =
                    (0usize #( + if <#types4 as auto_args::AutoArgs>::IS_SUBCOMMAND { 1 } else { 0 })*) == 1;
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
        }
        Struct(DataStruct {
            fields: syn::Fields::Unit,
            ..
        }) => {
            quote! {
                const REQUIRES_INPUT: bool = false;
                fn parse_internal(key: &str, args: &mut Vec<std::ffi::OsString>)
                                  -> Result<Self, auto_args::Error> {
                    Ok( #name )
                }
                fn tiny_help_message(key: &str) -> String {
                    "".to_string()
                }
            }
        }
        Struct(DataStruct {
            fields: syn::Fields::Unnamed(ref unnamed),
            ..
        }) => {
            if unnamed.unnamed.len() != 1 {
                panic!("AutoArgs does not handle tuple structs with more than one field");
            }
            let f = unnamed
                .unnamed
                .iter()
                .next()
                .expect("There should be a field here!");
            let mytype = f.ty.clone();
            quote! {
                const REQUIRES_INPUT: bool =
                    <#mytype as auto_args::AutoArgs>::REQUIRES_INPUT;
                const IS_SUBCOMMAND: bool =
                    <#mytype as auto_args::AutoArgs>::IS_SUBCOMMAND;
                fn parse_internal(key: &str, args: &mut Vec<std::ffi::OsString>)
                                  -> Result<Self, auto_args::Error> {
                    <#mytype as auto_args::AutoArgs>::parse_internal(key, args)
                        .map(|x| #name(x))
                }
                fn tiny_help_message(key: &str) -> String {
                    "fixme unnamed".to_string()
                }
            }
        }
        Enum(ref e) => {
            let v: Vec<_> = e.variants.iter().collect();
            let vnames: Vec<String> = e
                .variants
                .iter()
                .map(|v| camel_case_to_kebab(&v.ident.to_string()))
                .collect();
            let variant_docs: Vec<String> = e
                .variants
                .iter()
                .map(|v| get_doc_comment(&v.attrs))
                .collect();

            if vnames.iter().filter(|n| n.is_empty()).count() > 1 {
                panic!(
                    "AutoArgs: an enum may have at most one variant with an all-underscore \
                     name (used as a keyword-less default/subcommand variant)"
                );
            }

            let vnames_for_old = &vnames;
            // println!("variant names are {:?}", names);
            let return_enum = v.iter().map(|v| {
                let variant_name = v.ident.clone();
                return_with_fields(v.fields.clone(), quote!(#name::#variant_name), true)
            });
            let helps = v.iter().map(|v| {
                let variant_name = v.ident.clone();
                help_with_fields(v.fields.clone(), quote!(#name::#variant_name), true)
            });
            let usages = v.iter().map(|v| {
                let variant_name = v.ident.clone();
                usage_with_fields(v.fields.clone(), quote!(#name::#variant_name), true)
            });

            let keyworded_idx: Vec<usize> =
                (0..v.len()).filter(|&i| !vnames[i].is_empty()).collect();
            let default_idx: Option<usize> = (0..v.len()).find(|&i| vnames[i].is_empty());

            let keyworded_names: Vec<String> =
                keyworded_idx.iter().map(|&i| vnames[i].clone()).collect();
            let keyworded_names_for_err: Vec<String> = keyworded_names.clone();

            let keyworded_arms = keyworded_idx.iter().map(|&i| {
                let variant_name = v[i].ident.clone();
                let vname = &vnames[i];
                let body = new_style_variant_body(&v[i].fields, quote!(#name::#variant_name));
                quote! {
                    Some(#vname) => {
                        args.remove(0);
                        #body
                    }
                }
            });

            let default_arm = if let Some(i) = default_idx {
                let variant_name = v[i].ident.clone();
                let body = new_style_variant_body(&v[i].fields, quote!(#name::#variant_name));
                quote! { #body }
            } else {
                quote! {
                    if let Some(w) = word {
                        Err(auto_args::Error::UnknownSubcommand(
                            w,
                            vec![ #(#keyworded_names_for_err.to_string()),* ],
                        ))
                    } else {
                        Err(auto_args::Error::MissingSubcommand(
                            vec![ #(#keyworded_names_for_err.to_string()),* ],
                        ))
                    }
                }
            };

            let help_lines = keyworded_idx.iter().map(|&i| {
                let variant_name = v[i].ident.clone();
                let vname = &vnames[i];
                let vdoc = &variant_docs[i];
                let usage = new_style_variant_usage(&v[i].fields, quote!(#name::#variant_name));
                let detail = new_style_variant_help(&v[i].fields, quote!(#name::#variant_name));
                quote! {
                    {
                        let usage: String = #usage;
                        doc.push_str(&format!("\t{}\t{}\t{}\n", #vname, usage, #vdoc));
                        let detail: String = #detail;
                        doc.push_str(&detail);
                    }
                }
            });
            let default_help_line = if let Some(i) = default_idx {
                let variant_name = v[i].ident.clone();
                let vdoc = &variant_docs[i];
                let usage = new_style_variant_usage(&v[i].fields, quote!(#name::#variant_name));
                let detail = new_style_variant_help(&v[i].fields, quote!(#name::#variant_name));
                quote! {
                    {
                        let usage: String = #usage;
                        doc.push_str(&format!("\t(default)\t{}\t{}\n", usage, #vdoc));
                        let detail: String = #detail;
                        doc.push_str(&detail);
                    }
                }
            } else {
                quote! {}
            };

            let tiny_names = keyworded_names.clone();

            let requires_input_expr = match default_idx {
                None => quote! { true },
                Some(i) => requires_input_for_fields(&v[i].fields),
            };

            let s = quote! {
                const REQUIRES_INPUT: bool = #requires_input_expr;
                const IS_SUBCOMMAND: bool = true;
                fn parse_internal(key: &str, args: &mut Vec<std::ffi::OsString>)
                                  -> Result<Self, auto_args::Error>
                {
                    if key.is_empty() {
                        let word: Option<String> =
                            args.get(0).and_then(|s| s.to_str()).map(|s| s.to_string());
                        match word.as_deref() {
                            #( #keyworded_arms )*
                            _ => { #default_arm }
                        }
                    } else {
                        let _prefix = match key.chars().next() {
                            None | Some('_') => "--".to_string(),
                            _ => match key.chars().last() {
                                Some('-') => key.to_string(),
                                _ => format!("{}-", key),
                            }
                        };
                        let orig_args = args;
                        let mut error_args = orig_args.clone();
                        let mut most_used = 0;
                        let mut best_err = auto_args::Error::MissingOption("a missing thingy".to_string());
                        #(
                            {
                                let mut args = orig_args.clone();
                                let args = &mut args;
                                let variant = #vnames_for_old;
                                let _prefix = format!("{}{}", _prefix, variant);
                                let mut closure = || -> Result<_, auto_args::Error> {
                                    #return_enum
                                };
                                match closure() {
                                    Ok(v) => {
                                        *orig_args = args.clone();
                                        return Ok(v);
                                    }
                                    Err(e) => {
                                        let args_used = orig_args.len() - args.len();
                                        if args_used >= most_used {
                                            most_used = args_used;
                                            best_err = e;
                                            error_args = args.clone();
                                        }
                                    }
                                }
                            }

                        )*
                        *orig_args = error_args;
                        Err(best_err)
                    }
                }
                fn help_message(key: &str, doc: &str) -> String {
                    if key.is_empty() {
                        let mut doc = String::new();
                        doc.push_str("\tSUBCOMMANDS\t\t\n");
                        #( #help_lines )*
                        #default_help_line
                        doc
                    } else {
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
                                let variant = #vnames_for_old;
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
                }
                fn tiny_help_message(key: &str) -> String {
                    if key.is_empty() {
                        let mut doc = String::new();
                        doc.push_str("<");
                        #( doc.push_str(#tiny_names); doc.push_str("|"); )*
                        if doc.ends_with('|') {
                            doc.pop();
                        }
                        doc.push_str(">");
                        doc
                    } else {
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
                                let variant = #vnames_for_old;
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
                }
            };
            s
        }
        _ => panic!("AutoArgs only supports non-tuple structs"),
    };

    let generic_types = input.generics.type_params();
    let bounds = quote! {
        <#(#generic_types: auto_args::AutoArgs),*>
    };

    let tokens2: proc_macro2::TokenStream = quote! {
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
