use proc_macro::TokenStream;

use crate::component::ComponentFn;

mod component;

#[proc_macro_attribute]
pub fn component(_: TokenStream, item: TokenStream) -> TokenStream {
    match syn::parse::<ComponentFn>(item) {
        Ok(input) => input.expand().into(),
        Err(err) => err.into_compile_error().into(),
    }
}
