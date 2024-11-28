use bevy_bits::{DialogueBoxToken, TokenGroup};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse::Parse, spanned::Spanned};

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
    tokens(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn parse_closure(expr: &syn::Expr) -> syn::Result<Option<(&syn::Ident, &syn::Expr)>> {
    match expr {
        syn::Expr::Closure(closure) => {
            if closure.inputs.len() != 1 {
                return Err(syn::Error::new(
                    closure.inputs.span(),
                    "Expected a closure with exactly one input",
                ));
            }

            let name = closure.inputs.iter().next().unwrap();
            let name = match name {
                syn::Pat::Ident(ident) => &ident.ident,
                n => return Err(syn::Error::new(n.span(), "Expected a simple identifier")),
            };

            Ok(Some((name, closure.body.as_ref())))
        }
        _ => Ok(None),
    }
}

/// Recursively descend token groups while plucking off expressions.
fn descend_group(
    span: Span,
    group: &[TokenGroup],
    expressions: &mut impl Iterator<Item = syn::Expr>,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut items = Vec::new();
    for item in group.iter() {
        match item {
            TokenGroup::Bare(b) => {
                items.push(b.to_token_stream());
            }
            TokenGroup::Group(g) => {
                // pop off the next expression
                let expr = expressions.next().ok_or_else(|| {
                    syn::Error::new(
                        span,
                        "Expression argument count does not match expression groups",
                    )
                })?;

                let closure = parse_closure(&expr)?;
                let inner = descend_group(span, &g, expressions)?;

                let value = match closure {
                    Some((ident, body)) => {
                        quote! {
                            {
                                let #ident = #inner;
                                #body
                            }
                        }
                    }
                    None => {
                        quote! {
                            #expr(#inner)
                        }
                    }
                };

                items.push(value);
            }
        }
    }

    Ok(quote! {
        (
            #(#items,)*
        )
    })
}

fn tokens(input: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let Dialogue {
        string,
        expressions,
    } = syn::parse(input)?;

    let input = string.value();

    let mut tokens = bevy_bits::tokens::parse_groups(&input)
        .map_err(|e| syn::Error::new(string.span(), e.to_string()))?;

    if bevy_bits::tokens::TokenGroup::bare(&tokens) {
        let tokens = tokens.into_iter().map(|t| match t {
            bevy_bits::TokenGroup::Bare(b) => b,
            _ => unreachable!(),
        });

        let output = quote! {
            bevy_bits::DialogueBoxToken::Sequence(std::borrow::Cow::Borrowed(&[#(#tokens),*]))
        };

        Ok(output)
    } else {
        let mut expressions = expressions.into_iter().rev();
        tokens.push(TokenGroup::Bare(DialogueBoxToken::Command(
            bevy_bits::TextCommand::AwaitClear,
        )));
        Ok(descend_group(string.span(), &tokens, &mut expressions)?)
    }
}

struct Dialogue {
    string: syn::LitStr,
    expressions: syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>,
}

impl Parse for Dialogue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let string = input.parse()?;

        if let Ok(_) = <syn::Token![,] as Parse>::parse(input) {
            let expressions =
                syn::punctuated::Punctuated::<_, syn::Token![,]>::parse_terminated_with(
                    input,
                    syn::Expr::parse,
                )?;

            Ok(Self {
                string,
                expressions,
            })
        } else {
            Ok(Self {
                string,
                expressions: Default::default(),
            })
        }
    }
}
