use proc_macro::TokenStream;

use crate::component::ComponentFn;

mod component;

/// Marks a function as a UI component.
///
/// # Features
///
/// - Validates the function signature and returns helpful errors.
/// - Injects a default second argument when one is not provided.
///
/// # Example
///
/// ```rust,ignore
/// // This example is compiled in a crate that depends on vtui.
/// use vtui::prelude::*;
/// use vtui_macros::component;
///
/// #[component]
/// fn HelloWorld(c: Component) -> Node {
///     c.compose(|_| {})
/// }
/// ```
#[proc_macro_attribute]
pub fn component(_: TokenStream, item: TokenStream) -> TokenStream {
    match syn::parse::<ComponentFn>(item) {
        Ok(input) => input.expand().into(),
        Err(err) => err.into_compile_error().into(),
    }
}
