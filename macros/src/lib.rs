use proc_macro::TokenStream;
use quote::quote;

use crate::{component::ComponentFn, vtui::NodeInput};

mod component;
mod vtui;

#[proc_macro_attribute]
pub fn component(_: TokenStream, item: TokenStream) -> TokenStream {
    match syn::parse::<ComponentFn>(item) {
        Ok(input) => input.expand().into(),
        Err(err) => err.into_compile_error().into(),
    }
}

#[proc_macro]
pub fn vtui(input: TokenStream) -> TokenStream {
    match syn::parse::<NodeInput>(input) {
        Ok(input) => input.expand().into(),
        Err(err) => {
            let err = err.into_compile_error();
            let tokens = quote! {
                {
                    extern crate vtui as _vtui;
                    #err
                    _vtui::component::Node::from(__vtui_component)
                }
            };

            tokens.into()
        }
    }
}
