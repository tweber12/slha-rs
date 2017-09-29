#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use std::collections::HashSet;

#[proc_macro_derive(SlhaDeserialize)]
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
    let has_decays = fields
        .iter()
        .find(|f| f.ident == Some(syn::Ident::new("decays")))
        .is_some();
    let is_vec: HashSet<_> = fields
        .iter()
        .filter(|f| if let syn::Ty::Path(_, ref path) = f.ty {
            path.segments.len() == 1 && path.segments[0].ident == syn::Ident::new("Vec")
        } else {
            false
        })
        .collect();
    let vars = let_bindings(&fields, &is_vec);
    let matches = match_arms(&fields, &is_vec);
    let assign = struct_assign(&fields, has_decays, &is_vec);
    quote! {
        impl slha::SlhaDeserialize for #name {
            fn deserialize(input: &str) -> #name {
                #(#vars)*
                let mut decay_tables = ::std::collections::HashMap::new();

                let mut lines = input.lines().peekable();
                while let Some(segment) = slha::parse_segment(&mut lines) {
                    match segment.unwrap() {
                        slha::Segment::Block { name, block, scale } => {
                            match name.as_ref() {
                                #(#matches)*
                                _ => continue,
                            }
                        },
                        slha::Segment::Decay { pdg_id, width, decays } => {
                            if #has_decays {
                                decay_tables.insert(pdg_id, slha::DecayTable { width, decays });
                            } else {
                                continue;
                            }
                        },
                    }
                }

                #name {
                    #(#assign)*
                }
            }
        }
    }
}

fn let_bindings(fields: &[syn::Field], is_vec: &HashSet<&syn::Field>) -> Vec<quote::Tokens> {
    fields
        .iter()
        .filter(|field| field.ident != Some(syn::Ident::new("decays")))
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            if is_vec.contains(&field) {
                quote! { let mut #name: #ty = Vec::new(); }
            } else {
                quote! { let mut #name: Option<#ty> = None; }
            }
        })
        .collect()
}

fn struct_assign(
    fields: &[syn::Field],
    has_decays: bool,
    is_vec: &HashSet<&syn::Field>,
) -> Vec<quote::Tokens> {
    let mut vars: Vec<_> = fields
        .iter()
        .filter(|field| field.ident != Some(syn::Ident::new("decays")))
        .map(|field| if is_vec.contains(&field) {
            let name = field.ident.as_ref().unwrap();
            quote! { #name, }
        } else {
            let name = field.ident.as_ref().unwrap();
            quote! { #name: #name.expect("Missing field"), }
        })
        .collect();
    if has_decays {
        vars.push(quote! { decays: decay_tables, });
    }
    vars
}

fn match_arms(fields: &[syn::Field], is_vec: &HashSet<&syn::Field>) -> Vec<quote::Tokens> {
    fields
        .iter()
        .filter(|field| field.ident != Some(syn::Ident::new("decays")))
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            let match_str = format!("{}", name).to_lowercase();
            if is_vec.contains(&field) {
                quote! {
                    #match_str => {
                        #name.push(slha::parse_block_from(&block).unwrap())
                    }
                }
            } else {
                quote! {
                    #match_str => { #name = if #name.is_some() {
                        panic!("The block {} appears twice!", name)
                    } else {
                        Some(slha::parse_block_from(&block).unwrap())
                    }},
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
