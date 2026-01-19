use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn transform_vtui(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        {
            Node::from(c)
        }
    };

    expanded.into()
}
