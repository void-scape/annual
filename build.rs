use bevy_ecs_ldtk::ldtk::{FieldValue, LdtkJson};
use convert_case::Casing;
use quote::{format_ident, quote, TokenStreamExt};
use std::{env, iter::zip, path::Path};

fn entity_names(files: &[LdtkJson]) -> proc_macro2::TokenStream {
    let all_entities: Vec<_> = files
        .iter()
        .flat_map(|w| w.defs.entities.iter().map(|e| e.identifier.as_str()))
        .collect();

    let variants: Vec<_> = all_entities.iter().map(|e| format_ident!("{e}")).collect();
    let idents = zip(&variants, &all_entities).map(|(v, a)| {
        quote! {
            Self::#v => #a
        }
    });

    quote! {
        /// An enum of all entities in all LDtk files.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Entities {
            #(#variants),*
        }

        impl Entities {
            /// Return the identifier string that LDtk uses.
            pub const fn identifier(&self) -> &'static str {
                match self {
                    #(#idents),*
                }
            }
        }
    }
}

fn interactions(files: &[LdtkJson]) -> proc_macro2::TokenStream {
    let all_interactions: Vec<_> = files
        .iter()
        .flat_map(|w| {
            w.levels.iter().flat_map(|l| {
                l.layer_instances.iter().flatten().flat_map(|i| {
                    i.entity_instances.iter().filter_map(|e| {
                        if e.identifier == "Interaction" {
                            e.field_instances.iter().find_map(|i| {
                                if i.identifier == "Name" {
                                    match &i.value {
                                        FieldValue::String(v) => v.as_ref(),
                                        _ => None,
                                    }
                                } else {
                                    None
                                }
                            })
                        } else {
                            None
                        }
                    })
                })
            })
        })
        .collect();

    let variants: Vec<_> = all_interactions
        .iter()
        .map(|e| {
            let pascal = e.to_case(convert_case::Case::Pascal);
            format_ident!("{pascal}")
        })
        .collect();

    let idents = zip(&variants, &all_interactions).map(|(v, a)| {
        quote! {
            Self::#v => #a
        }
    });

    let from = zip(&variants, &all_interactions).map(|(v, a)| {
        quote! {
            #a => Some(Self::#v)
        }
    });

    quote! {
        /// An enum of all interaction values in all LDtk files.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Interactions {
            #(#variants),*
        }

        impl Interactions {
            /// Return the identifier string that LDtk uses.
            pub const fn identifier(&self) -> &'static str {
                match self {
                    #(#idents),*
                }
            }

            pub fn from_identifier(ident: &str) -> Option<Self> {
                match ident {
                    #(#from,)*
                    _ => None
                }
            }

            pub fn iter() -> impl Iterator<Item = Self> {
                [
                    #(Self::#variants),*
                ]
                    .into_iter()
            }
        }
    }
}

fn main() {
    println!("cargo::rerun-if-changed=assets/ldtk/");

    let mut worlds = Vec::new();

    for entry in walkdir::WalkDir::new("assets/ldtk") {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().is_some_and(|e| e == "ldtk") {
            let file = std::fs::read(path)
                .unwrap_or_else(|e| panic!("Unable to open LDtk file {path:?}: {e}"));
            let world: bevy_ecs_ldtk::ldtk::LdtkJson = serde_json::from_slice(&file)
                .unwrap_or_else(|e| panic!("Error reading LDtk file {path:?}: {e}"));
            worlds.push(world);
        }
    }

    let mut entity_names = entity_names(&worlds);
    let interactions = interactions(&worlds);

    entity_names.append_all(interactions);

    let parsed: syn::File = syn::parse2(entity_names).expect("Error parsing generated code");
    let output = prettyplease::unparse(&parsed);

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("entities.rs");

    std::fs::write(dest_path, output.as_bytes()).expect("error writing LDtk entities file");
}
