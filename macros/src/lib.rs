use proc_macro::TokenStream;

use crate::{component::transform_component, vtui::transform_vtui};

mod component;
mod vtui;

#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    transform_component(attr, item)
}

#[proc_macro]
pub fn vtui(input: TokenStream) -> TokenStream {
    transform_vtui(input)
}
