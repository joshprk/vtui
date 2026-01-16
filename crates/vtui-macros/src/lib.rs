use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

use crate::component::validate_component_signature;

mod component;

/// Procedural macro attribute for defining component functions
///
/// # Usage
///
/// ```ignore
/// #[component]
/// fn MyComponent(c: Component) -> Node {
///     // component logic
///     vtui! {}
/// }
///
/// #[component]
/// fn MyComponentWithProps(c: Component, props: MyProps) -> Node {
///     // component logic with props
///     vtui! {}
/// }
/// ```
///
/// # Requirements
///
/// - Function must not be async or generic
/// - Must take `Component` as first argument
/// - Optionally takes a props type implementing `Props` as second argument
/// - Must return `Node`
///
/// # Implementation Details
///
/// If no props argument is provided, a hidden `__props: ()` argument is added
/// to maintain a consistent internal function signature.
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(item as ItemFn);

    // Validate the function signature
    if let Err(err) = validate_component_signature(&func.sig) {
        return err.to_compile_error().into();
    }

    // Add hidden props argument if not present
    if func.sig.inputs.len() == 1 {
        func.sig.inputs.push(syn::parse_quote!(__props: ()));
    }

    // The function body remains unchanged - we just add the allow attribute
    // to prevent warnings about non-snake_case component names (PascalCase is idiomatic)
    quote! {
        #[allow(non_snake_case)]
        #func
    }
    .into()
}

#[proc_macro]
pub fn vtui(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        Node::try_from(c).ok().unwrap()
    };
    expanded.into()
}
