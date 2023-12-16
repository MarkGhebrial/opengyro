use proc_macro::{Delimiter, Group, TokenStream, TokenTree};
use std::iter::FromIterator;

#[proc_macro]
pub fn hex(input: TokenStream) -> TokenStream {
    let ts = TokenStream::from_iter(input.into_iter());
    TokenStream::from(TokenTree::Group(Group::new(Delimiter::Bracket, ts)))
}

#[proc_macro]
pub fn pipeline(input: TokenStream) -> TokenStream {
    println!("{:#?}", input);

    TokenStream::new()
}