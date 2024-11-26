use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};
use winnow::{stream::Stream, token::take_while, PResult, Parser};

#[proc_macro]
pub fn tokens(input: TokenStream) -> TokenStream {
    let input_str = parse_macro_input!(input as LitStr).value();
    let mut result = Vec::new();
    let path = quote! { crate::text };

    let input = &mut &*input_str;

    while let Ok(text) = parse_normal(input) {
        result.push(quote! { #text.into_token() });
        if let Ok(commands) = parse_commands(input, path.clone()) {
            result.extend(commands);
        }
    }

    let output = quote! {
        {
            [#(#result),*]
        }
    };

    output.into()
}

fn parse_normal<'a>(input: &mut &'a str) -> PResult<&'a str> {
    take_while(1.., |c| c != '[').parse_next(input)
}

fn parse_commands(
    input: &mut &str,
    text_path: proc_macro2::TokenStream,
) -> PResult<Vec<proc_macro2::TokenStream>> {
    let mut commands = Vec::new();

    // TODO: recursive effets might not be the best solution, don't work here anyway
    '['.parse_next(input)?;
    if let Some((_, token)) = input.peek_token() {
        if token == '[' {
            commands.extend(parse_commands(input, text_path.clone())?);
        }
    }
    let args = take_while(1.., |c| c != ']').parse_next(input)?;
    ']'.parse_next(input)?;
    '('.parse_next(input)?;
    let cmd = take_while(1.., |c| c != ')').parse_next(input)?;
    ')'.parse_next(input)?;

    commands.push(quote! { #text_path::TextToken::parse_command(#args, #cmd) });

    Ok(commands)
}
