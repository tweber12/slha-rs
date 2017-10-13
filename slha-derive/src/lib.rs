// Copyright 2017 Torsten Weber
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

#[proc_macro_derive(SlhaDeserialize, attributes(slha))]
pub fn slha_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = impl_slha_deserialize(&ast);
    gen.parse().unwrap()
}

fn impl_slha_deserialize(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let body = match ast.body {
        syn::Body::Struct(ref body) => body,
        _ => panic!("#[derive(slha_deserialize)] is only defined for structs!"),
    };
    let fields = match *body {
        syn::VariantData::Struct(ref fields) => fields,
        _ => panic!("#[derive(slha_deserialize)] is not defined for tuple structs!"),
    };
    let (blocks, has_decays) = extract_decays(fields);
    let let_bindings = generate_let_bindings(&blocks, has_decays);
    let match_arm_blocks = generate_match_arm_blocks(&blocks);
    let match_arm_decays = generate_match_arm_decays(has_decays);
    let assignments = generate_struct_assignments(&blocks, has_decays);
    quote! {
        impl slha::SlhaDeserialize for #name {
            fn deserialize(input: &str) -> slha::errors::Result<#name> {
                use slha::errors::ResultExt;
                #(#let_bindings)*
                let mut lines = input.lines().peekable();
                while let Some(segment) = slha::internal::parse_segment(&mut lines) {
                    match segment? {
                        #match_arm_blocks
                        #match_arm_decays
                    }
                }

                Ok(#name {
                    #(#assignments)*
                })
            }
        }
    }
}

struct Block<'a> {
    field: &'a syn::Field,
    name: &'a syn::Ident,
    ty: &'a syn::Ty,
    block_name: String,
    attributes: Vec<&'a syn::NestedMetaItem>,
}
impl<'a> Block<'a> {
    fn from_syn_field(field: &'a syn::Field) -> Block<'a> {
        let name = field.ident.as_ref().expect(
            "BUG: This should be a struct (with named fields)",
        );
        let attributes = normalize_attrs(&field.attrs);
        let block_name = extract_block_name(&attributes).unwrap_or_else(|| name.to_string());
        Block {
            field,
            name,
            ty: &field.ty,
            block_name,
            attributes,
        }
    }
}

fn normalize_attrs(attrs: &[syn::Attribute]) -> Vec<&syn::NestedMetaItem> {
    let mut norm = Vec::new();
    let header = syn::Ident::new("slha");
    for attr in attrs {
        if attr.is_sugared_doc {
            continue;
        }
        let list = match attr.value {
            syn::MetaItem::List(ref name, ref list) if name == &header => list,
            _ => continue,
        };
        norm.extend(list);
    }
    norm
}

fn extract_block_name(attrs: &[&syn::NestedMetaItem]) -> Option<String> {
    let rename = syn::Ident::new("rename");
    for attr in attrs {
        let item = match **attr {
            syn::NestedMetaItem::Literal(_) => continue,
            syn::NestedMetaItem::MetaItem(ref item) => item,
        };
        let new_lit = match *item {
            syn::MetaItem::NameValue(ref name, ref lit) if name == &rename => lit,
            _ => continue,
        };
        let new_name = match *new_lit {
            syn::Lit::Str(ref str, _) => str.clone(),
            _ => panic!("Invalid block name, only strings are allowed in the 'rename' attribute"),
        };
        return Some(new_name);
    }
    None
}

fn extract_decays<'a>(fields: &'a [syn::Field]) -> (Vec<Block<'a>>, bool) {
    let mut blocks = Vec::new();
    let mut decay = false;
    let decay_ident = syn::Ident::new("decays");
    for field in fields {
        if field.ident.as_ref().unwrap() == &decay_ident {
            decay = true;
        } else {
            blocks.push(Block::from_syn_field(&field));
        }
    }
    (blocks, decay)
}

fn generate_let_bindings(blocks: &[Block], has_decays: bool) -> Vec<quote::Tokens> {
    let mut bindings: Vec<_> = blocks
        .iter()
        .map(|field| {
            let name = field.name;
            let ty = &field.ty;
            quote! {
                let mut #name: <#ty as slha::internal::WrappedBlock<slha::errors::Error>>::Wrapper =
                    <#ty as slha::internal::WrappedBlock<slha::errors::Error>>::Wrapper::default();
            }
        })
        .collect();
    if has_decays {
        bindings.push(
            quote! {
                let mut decays: ::std::collections::HashMap<i64, slha::DecayTable> = ::std::collections::HashMap::new();
            }
        );
    }
    bindings
}

fn generate_match_arm_decays(has_decays: bool) -> quote::Tokens {
    if has_decays {
        quote! {
            slha::internal::Segment::Decay { pdg_id, width, decays: decay_table } => {
                let duplicate = decays.insert(pdg_id, slha::DecayTable { width, decays: decay_table });
                if duplicate.is_some() {
                    return Err(slha::errors::ErrorKind::DuplicateDecay(pdg_id).into());
                }
            },
        }
    } else {
        quote! {
            slha::internal::Segment::Decay { .. } => continue,
        }
    }
}

fn generate_match_arm_blocks(blocks: &[Block]) -> quote::Tokens {
    let arms = generate_match_arms_block_name(blocks);
    if arms.is_empty() {
        quote!{
            slha::internal::Segment::Block { .. } => continue,
        }
    } else {
        quote!{
            slha::internal::Segment::Block { name, block, scale } => {
                match name.as_ref() {
                    #(#arms)*
                    _ => continue,
                }
            },
        }
    }
}

fn generate_match_arms_block_name(blocks: &[Block]) -> Vec<quote::Tokens> {
    blocks
        .iter()
        .map(|block| {
            let ty = block.ty;
            let name = block.name;
            let match_str = &block.block_name;
            quote! {
                #match_str => {
                    <#ty as slha::internal::WrappedBlock<slha::errors::Error>>::parse_into(&block, scale, &mut #name, #match_str)?;
                }
            }
        })
        .collect()
}

fn generate_struct_assignments(blocks: &[Block], has_decays: bool) -> Vec<quote::Tokens> {
    let mut assignments: Vec<_> = blocks
        .iter()
        .map(|field| {
            let name = field.name;
            let ty = field.ty;
            let name_str = &field.block_name;
            quote! {
                #name: <#ty as slha::internal::WrappedBlock<slha::errors::Error>>::unwrap(#name_str, #name)?,
            }
        })
        .collect();
    if has_decays {
        assignments.push(quote! { decays, })
    }
    assignments
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
