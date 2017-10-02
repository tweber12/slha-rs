#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

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
    let (fields, has_decays) = Field::from_struct_ast(fields);
    let vars = let_bindings(&fields);
    let matches = match_arms(&fields);
    let decay = insert_decay(has_decays);
    let assign = struct_assign(&fields);
    quote! {
        impl slha::SlhaDeserialize for #name {
            fn deserialize(input: &str) -> Result<#name, slha::ParseError> {
                #(#vars)*
                let mut lines = input.lines().peekable();
                while let Some(segment) = slha::parse_segment(&mut lines) {
                    match segment? {
                        slha::Segment::Block { name, block, scale } => {
                            match name.as_ref() {
                                #(#matches)*
                                _ => continue,
                            }
                        },
                        #decay
                    }
                }

                Ok(#name {
                    #(#assign)*
                })
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FieldMode {
    Normal,
    Optional,
    Vector,
    Decays,
}

#[derive(Clone, Debug, PartialEq)]
struct Field<'a> {
    field: &'a syn::Field,
    mode: FieldMode,
}
impl<'a> Field<'a> {
    fn from_struct_ast(fields: &'a [syn::Field]) -> (Vec<Field<'a>>, bool) {
        let mut has_decays = false;
        let fs = fields
            .iter()
            .map(|field| {
                let mode = Field::classify_field(field);
                if mode == FieldMode::Decays {
                    has_decays = true;
                }
                Field { field, mode }
            })
            .collect();
        (fs, has_decays)
    }
    fn classify_field(field: &syn::Field) -> FieldMode {
        if field.ident == Some(syn::Ident::new("decays")) {
            return FieldMode::Decays;
        }
        if let syn::Ty::Path(_, ref path) = field.ty {
            if path.segments.len() == 1 {
                match &path.segments[0].ident {
                    id if id == &syn::Ident::new("Vec") => return FieldMode::Vector,
                    id if id == &syn::Ident::new("Option") => return FieldMode::Optional,
                    _ => (),
                }
            }
        }
        FieldMode::Normal
    }
}

fn let_bindings(fields: &[Field]) -> Vec<quote::Tokens> {
    fields
        .iter()
        .map(|field| {
            let name = field.field.ident.as_ref().unwrap();
            let ty = &field.field.ty;
            match field.mode {
                FieldMode::Normal => quote! { let mut #name: Option<#ty> = None; },
                FieldMode::Optional => quote! { let mut #name: #ty = None; },
                FieldMode::Vector => quote! { let mut #name: #ty = Vec::new(); },
                FieldMode::Decays => {
                    quote! { let mut decay_tables: #ty = ::std::collections::HashMap::new(); }
                }
            }
        })
        .collect()
}

fn insert_decay(has_decays: bool) -> quote::Tokens {
    if has_decays {
        quote! {
            slha::Segment::Decay { pdg_id, width, decays } => {
                decay_tables.insert(pdg_id, slha::DecayTable { width, decays });
            },
        }
    } else {
        quote! {
            slha::Segment::Decay { .. } => {
                continue;
            }
        }
    }
}

fn struct_assign(fields: &[Field]) -> Vec<quote::Tokens> {
    fields
        .iter()
        .map(|field| {
            let name = field.field.ident.as_ref().unwrap();
            match field.mode {
                FieldMode::Vector | FieldMode::Optional => quote! { #name, },
                FieldMode::Decays => quote! { decays: decay_tables, },
                FieldMode::Normal => quote! { #name: #name.expect("Missing field"), },
            }
        })
        .collect()
}

fn match_arms(fields: &[Field]) -> Vec<quote::Tokens> {
    fields
        .iter()
        .filter(|field| field.mode != FieldMode::Decays)
        .map(|field| {
            let name = field.field.ident.as_ref().unwrap();
            let match_str = format!("{}", name).to_lowercase();
            match field.mode {
                FieldMode::Vector => {
                    quote! {
                    #match_str => {
                        #name.push(slha::parse_block_from(&block, scale)?)
                    }
                }
                }
                FieldMode::Normal | FieldMode::Optional => {
                    quote! {
                    #match_str => { #name = if #name.is_some() {
                        panic!("The block {} appears twice!", name)
                    } else {
                        Some(slha::parse_block_from(&block, scale)?)
                    }},
                }
                }
                FieldMode::Decays => unreachable!("Filtered out before"),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
