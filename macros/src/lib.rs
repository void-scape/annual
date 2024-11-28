use proc_macro::TokenStream;
use quote::{quote, TokenStreamExt};
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
pub fn t(input: TokenStream) -> TokenStream {
    let input_str = parse_macro_input!(input as LitStr).value();
    let mut result = Vec::new();

    let input = &mut &*input_str;

    while let Ok(text) = parse_normal(input) {
        let token = bevy_bits::DialogueBoxToken::Section(bevy_bits::tokens::TextSection::from(
            text.to_owned(),
        ));

        if !text.is_empty() {
            result.push(token);
        }

        if input.peek_token().is_some() {
            match parse_command(input) {
                Ok(command) => result.push(command),
                Err(e) => panic!("{e}"),
            }
        } else {
            break;
        }
    }

    // result.push(bevy_bits::DialogueBoxToken::Command(
    //     bevy_bits::tokens::TextCommand::Clear,
    // ));
    let result = result.into_iter().map(WrapperToken).collect::<Vec<_>>();

    let output = quote! {
        bevy_bits::DialogueBoxToken::Sequence(std::borrow::Cow::Borrowed(&[#(#result),*]))
    };

    output.into()
}

fn parse_normal<'a>(input: &mut &'a str) -> PResult<&'a str> {
    take_while(0.., |c| c != '[' && c != '{').parse_next(input)
}

fn parse_command(input: &mut &str) -> PResult<bevy_bits::DialogueBoxToken> {
    '['.parse_next(input)?;
    let args: Result<&str, winnow::error::ErrMode<winnow::error::ContextError>> =
        take_while(1.., |c| c != ']').parse_next(input);
    ']'.parse_next(input)?;
    '('.parse_next(input)?;
    let cmd = take_while(1.., |c| c != ')').parse_next(input)?;
    ')'.parse_next(input)?;

    Ok(bevy_bits::DialogueBoxToken::parse_command(
        match args {
            Ok(args) => Some(args),
            Err(_) => None,
        },
        cmd,
    ))
}

struct WrapperToken(bevy_bits::DialogueBoxToken);
struct WrapperEffect<'a>(&'a bevy_bits::TextEffect);
struct WrapperColor<'a>(&'a bevy_bits::TextColor);

impl quote::ToTokens for WrapperEffect<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(quote! { bevy_bits::TextEffect:: });
        tokens.append_all(match &self.0 {
            bevy_bits::TextEffect::Wave => quote! { Wave },
        });
    }
}

impl quote::ToTokens for WrapperColor<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match &self.0 {
            bevy_bits::TextColor::Red => tokens.append_all(quote! { bevy_bits::TextColor::Red }),
            bevy_bits::TextColor::Green => {
                tokens.append_all(quote! { bevy_bits::TextColor::Green })
            }
            bevy_bits::TextColor::Blue => tokens.append_all(quote! { bevy_bits::TextColor::Blue }),
        }
    }
}

impl quote::ToTokens for WrapperToken {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match &self.0 {
            bevy_bits::DialogueBoxToken::Section(section) => {
                let text = &section.text;
                let color = &section.color.as_ref().map(WrapperColor);
                let color = if let Some(color) = color {
                    quote! { #color }
                } else {
                    quote! { None }
                };
                let effects = section.effects.iter().map(WrapperEffect).collect::<Vec<_>>();

                tokens.append_all(quote! {
                bevy_bits::DialogueBoxToken::Section(
                    bevy_bits::tokens::TextSection {
                        text: std::borrow::Cow::Borrowed(#text),
                        color: #color,
                        effects: std::borrow::Cow::Borrowed(&[#(#effects),*])
                    })
                });
            }
            bevy_bits::DialogueBoxToken::Command(cmd) => match &cmd {
                // bevy_bits::TextCommand::Clear => tokens.append_all(
                //     quote! { bevy_bits::DialogueBoxToken::Command(bevy_bits::tokens::TextCommand::Clear) },
                // ),
                bevy_bits::TextCommand::Speed(speed) => tokens.append_all(
                    quote! { bevy_bits::DialogueBoxToken::Command(bevy_bits::tokens::TextCommand::Speed(#speed)) },
                ),
                bevy_bits::TextCommand::Pause(pause) => tokens.append_all(
                    quote! { bevy_bits::DialogueBoxToken::Command(bevy_bits::tokens::TextCommand::Pause(#pause)) },
                ),
            },
            bevy_bits::DialogueBoxToken::Sequence(seq) => {
                let seq = seq.iter().map(|t| WrapperToken(t.clone())).collect::<Vec<_>>();
                 tokens.append_all(
                    quote! { bevy_bits::DialogueBoxToken::Sequence(std::borrow::Cow::Borrowed(&[#(#seq),*])) },
                );
            },
        }
    }
}
