
use std::cmp::Ordering::*;

use proc_macro::TokenStream;
use syn::{braced, parse_macro_input, token, Ident, Result, Token, Variant, Error};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

#[allow(dead_code)]
struct SortedEnum {
    enum_token: Token![enum],
    ident: Ident,
    brace_token: token::Brace,
    variants: Punctuated<Variant, Token![,]>,
}

impl Parse for SortedEnum {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![struct]) {
            return Err(lookahead.error());
        } else if lookahead.peek(Token![enum]) {
            ()
        } else {
            return Err(lookahead.error());
        }

        let content;
        let new = SortedEnum {
            enum_token: input.parse()?,
            ident: input.parse()?,
            brace_token: braced!(content in input),
            variants: content.parse_terminated(Variant::parse, Token![,])?
        };

        // Check that the variants are sorted
        let mut variants_iter = new.variants.iter().map(|var| &var.ident);
        let Some(mut ident1) = variants_iter.next() else { return Ok(new); };
        

        for ident2 in variants_iter {
            if ident1.cmp(ident2) != Less {
                return Err(Error::new(
                        ident1.span(), 
                        format!("{} is out of order. Please sort your variants correctly.", ident1)
                    ));
            }
            ident1 = ident2;
        }

        Ok(new)
    }
}

#[proc_macro_attribute]
pub fn sorted(_args: TokenStream, tokens: TokenStream) -> TokenStream {
    let token_stream = tokens.clone();
    let _input = parse_macro_input!(tokens as SortedEnum);
    token_stream
}
