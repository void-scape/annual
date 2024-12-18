// fn interactions(files: &[LdtkJson]) -> proc_macro2::TokenStream {
//     let all_interactions: Vec<_> = files
//         .iter()
//         .flat_map(|w| {
//             w.levels.iter().flat_map(|l| {
//                 l.layer_instances.iter().flatten().flat_map(|i| {
//                     i.entity_instances.iter().filter_map(|e| {
//                         if e.identifier == "Interaction" {
//                             e.field_instances.iter().find_map(|i| {
//                                 if i.identifier == "Name" {
//                                     match &i.value {
//                                         FieldValue::String(v) => v.as_ref(),
//                                         _ => None,
//                                     }
//                                 } else {
//                                     None
//                                 }
//                             })
//                         } else {
//                             None
//                         }
//                     })
//                 })
//             })
//         })
//         .collect();
//
//     let variants: Vec<_> = all_interactions
//         .iter()
//         .map(|e| {
//             let pascal = e.to_case(convert_case::Case::Pascal);
//             format_ident!("{pascal}")
//         })
//         .collect();
//
//     let idents = zip(&variants, &all_interactions).map(|(v, a)| {
//         quote! {
//             Self::#v => #a
//         }
//     });
//
//     let from = zip(&variants, &all_interactions).map(|(v, a)| {
//         quote! {
//             #a => Some(Self::#v)
//         }
//     });
//
//     quote! {
//         /// An enum of all interaction values in all LDtk files.
//         #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//         pub enum Interactions {
//             #(#variants),*
//         }
//
//         impl Interactions {
//             /// Return the identifier string that LDtk uses.
//             pub const fn identifier(&self) -> &'static str {
//                 match self {
//                     #(#idents),*
//                 }
//             }
//
//             pub fn from_identifier(ident: &str) -> Option<Self> {
//                 match ident {
//                     #(#from,)*
//                     _ => None
//                 }
//             }
//
//             pub fn iter() -> impl Iterator<Item = Self> {
//                 [
//                     #(Self::#variants),*
//                 ]
//                     .into_iter()
//             }
//         }
//     }
// }

fn main() {
    println!("cargo::rerun-if-changed=assets/ldtk/");

    bevy_ldtk_scene::comp::build_all_ldtk_scenes("assets/ldtk").unwrap();
    let output = bevy_ldtk_scene::comp::entities("assets/ldtk").unwrap();

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("ldtk.rs");
    std::fs::write(dest_path, output.as_bytes()).expect("error writing LDtk file");
}
