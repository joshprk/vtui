use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, parse_macro_input};

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

    // Extract props type for trait bound check
    let props_type = if func.sig.inputs.len() == 2 {
        // Get the second argument's type
        match func.sig.inputs.iter().nth(1) {
            Some(FnArg::Typed(pat)) => pat.ty.clone(),
            _ => syn::parse_quote!(()),
        }
    } else {
        // Add hidden props argument if not present
        func.sig.inputs.push(syn::parse_quote!(__props: ()));
        syn::parse_quote!(())
    };

    let fn_name = &func.sig.ident;

    // Generate a compile-time assertion that Props is implemented
    // This creates a const function that will fail to compile if the trait bound isn't satisfied
    let props_check = quote! {
        const _: fn() = || {
            // This function is never called, but the compiler checks the trait bound
            fn assert_props_impl<T: Props>() {}
            assert_props_impl::<#props_type>();
        };
    };

    quote! {
        #[allow(non_snake_case)]
        #func

        // Compile-time check that props implements Props trait
        // This is scoped to avoid naming conflicts
        #[allow(non_snake_case, dead_code)]
        mod #fn_name {
            use super::*;
            #props_check
        }
    }
    .into()
}

#[proc_macro]
pub fn vtui(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        Node::from(c)
    };
    expanded.into()
}
