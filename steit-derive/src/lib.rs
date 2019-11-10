#![feature(bind_by_move_pattern_guards)]

extern crate proc_macro;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

mod attr;
mod ctx;
mod derive;
mod r#impl;
mod string;

#[proc_macro_attribute]
pub fn steitize(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    derive::derive(args, input).into()
}
