use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};
use winnow::{stream::Stream, token::take_while, PResult, Parser};

fn fragment(input: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let input: syn::DeriveInput = syn::parse(input)?;

    let ident = &input.ident;
    let (impl_gen, ty_gen, where_gen) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_gen crate::dialogue::fragment::IntoFragment for #ident #ty_gen #where_gen {
            type Fragment<Data> = crate::dialogue::fragment::Leaf<#ident #ty_gen>;

            fn into_fragment<Data>(
                self,
                _: &mut bevy::prelude::Commands,
            ) -> (Self::Fragment<Data>, crate::dialogue::fragment::FragmentNode) {
                crate::dialogue::fragment::Leaf::new(self)
            }
        }
    })
}

#[proc_macro_derive(Fragment)]
pub fn derive_fragment(input: TokenStream) -> TokenStream {
    fragment(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn tokens(input: TokenStream) -> TokenStream {
    let input_str = parse_macro_input!(input as LitStr).value();
    let mut result = Vec::new();

    let input = &mut &*input_str;

    while let Ok(text) = parse_normal(input) {
        if !text.is_empty() {
            result.push(quote! { #text.into_token() });
        }

        if input.peek_token().is_some() {
            match parse_commands(input) {
                Ok(commands) => result.extend(commands),
                Err(e) => panic!("{e}"),
            }
        } else {
            break;
        }
    }

    let output = quote! {
        {
            use crate::dialogue_box::IntoDialogueBoxToken;
            [#(#result),*]
        }
    };

    output.into()
}

fn parse_normal<'a>(input: &mut &'a str) -> PResult<&'a str> {
    take_while(0.., |c| c != '[').parse_next(input)
}

fn parse_commands(input: &mut &str) -> PResult<Vec<proc_macro2::TokenStream>> {
    let mut commands = Vec::new();

    '['.parse_next(input)?;
    let args = take_while(1.., |c| c != ']').parse_next(input)?;
    ']'.parse_next(input)?;
    '('.parse_next(input)?;
    let cmd = take_while(1.., |c| c != ')').parse_next(input)?;
    ')'.parse_next(input)?;

    commands.push(quote! { crate::dialogue_box::DialogueBoxToken::parse_command(#args, #cmd) });

    Ok(commands)
}
