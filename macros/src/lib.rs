use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;

// fn fragment(input: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
//     let input: syn::DeriveInput = syn::parse(input)?;
//
//     let ident = &input.ident;
//     let (impl_gen, ty_gen, where_gen) = input.generics.split_for_impl();
//
//     Ok(quote! {
//         impl #impl_gen crate::dialogue::fragment::IntoFragment for #ident #ty_gen #where_gen {
//             type Fragment<Data> = crate::dialogue::fragment::Leaf<#ident #ty_gen>;
//
//             fn into_fragment<Data>(
//                 self,
//                 _: &mut bevy::prelude::Commands,
//             ) -> (Self::Fragment<Data>, crate::dialogue::fragment::FragmentNode) {
//                 crate::dialogue::fragment::Leaf::new(self)
//             }
//         }
//     })
// }
//
// #[proc_macro_derive(Fragment)]
// pub fn derive_fragment(input: TokenStream) -> TokenStream {
//     fragment(input)
//         .unwrap_or_else(syn::Error::into_compile_error)
//         .into()
// }

fn character(input: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let input: syn::DeriveInput = syn::parse(input)?;

    let ident = &input.ident;
    let fn_name = proc_macro2::Ident::new(&ident.to_string().to_lowercase(), Span::call_site());
    let trait_name = proc_macro2::Ident::new(&format!("Into{}", ident), Span::call_site());
    let into_frag = quote! { crate::textbox::prelude::IntoBox<C> };

    Ok(quote! {
        pub trait #trait_name <C>
        {
            fn #fn_name(self) -> impl #into_frag
                where Self: #into_frag;
        }

        impl<C, T> #trait_name <C> for T
        where
            T: #into_frag,
            C: 'static
        {
            fn #fn_name(self) -> impl #into_frag
                where Self: #into_frag
            {
                use crate::characters::CharacterAssets;
                use crate::textbox::prelude::*;
                self.sfx_char(#ident::SFX).portrait(#ident::POR)
            }
        }
    })
}

#[proc_macro_derive(Character)]
pub fn derive_character(input: TokenStream) -> TokenStream {
    character(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
