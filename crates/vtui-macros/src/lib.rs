use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);

    let attrs = func.attrs;
    let vis = func.vis;
    let sig = func.sig;
    let block = func.block;

    let expanded = quote! {
        #(#attrs)*
        #[allow(non_snake_case)]
        #vis #sig #block
    };

    expanded.into()
}
