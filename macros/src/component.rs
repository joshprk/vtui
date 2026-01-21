use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, PathArguments, ReturnType, Signature, Type, TypePath, parse_macro_input};

/// Expected identifier for the Component type in function signatures
const COMPONENT_TYPE_IDENT: &str = "Component";

/// Expected identifier for the Node return type
const NODE_TYPE_IDENT: &str = "Node";

pub(crate) fn transform_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
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

/// Checks if a type is the `Component` type by examining its path segments
pub(crate) fn is_component_type(ty: &Type) -> bool {
    match ty {
        Type::Path(TypePath { qself: None, path }) => path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == COMPONENT_TYPE_IDENT),
        _ => false,
    }
}

/// Checks if a type is the `Node` return type by examining its path segments
pub(crate) fn is_node_type(ty: &Type) -> bool {
    match ty {
        Type::Path(TypePath { qself: None, path }) => path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == NODE_TYPE_IDENT),
        _ => false,
    }
}

/// Validates that the props type is a concrete, owned type with no generic arguments
///
/// Props must be:
/// - A path type (not a reference, tuple, etc.)
/// - Without qualified self (no `<Type as Trait>::AssocType`)
/// - With no generic arguments on any segment
pub(crate) fn validate_props_type(ty: &Type) -> syn::Result<()> {
    match ty {
        Type::Path(TypePath { qself: None, path })
            if path
                .segments
                .iter()
                .all(|seg| matches!(seg.arguments, PathArguments::None)) =>
        {
            Ok(())
        }
        _ => Err(syn::Error::new_spanned(
            ty,
            "props must be a concrete, owned type implementing `Props` (no generics, references, or complex types)",
        )),
    }
}

/// Validates the component function signature
///
/// Requirements:
/// - Not async
/// - No generic parameters
/// - 1 or 2 arguments (Component or Component + Props)
/// - First argument must be Component (not self)
/// - Second argument (if present) must be valid props type
/// - Must return Node
pub(crate) fn validate_component_signature(sig: &Signature) -> syn::Result<()> {
    // Check for async - components must be synchronous
    if sig.asyncness.is_some() {
        return Err(syn::Error::new_spanned(
            sig.asyncness,
            "component functions cannot be async",
        ));
    }

    // Check for generics - components must be concrete
    if !sig.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &sig.generics,
            "component functions cannot have generic parameters",
        ));
    }

    // Validate argument count
    let arg_count = sig.inputs.len();
    if !(1..=2).contains(&arg_count) {
        return Err(syn::Error::new_spanned(
            &sig.inputs,
            "component functions must take either `Component` or `Component, Props`",
        ));
    }

    // Validate first argument is Component (not self)
    match sig.inputs.first() {
        Some(FnArg::Typed(pat)) if is_component_type(&pat.ty) => {}
        Some(FnArg::Receiver(recv)) => {
            return Err(syn::Error::new_spanned(
                recv,
                "component functions cannot take `self` - first argument must be `Component`",
            ));
        }
        Some(arg) => {
            return Err(syn::Error::new_spanned(
                arg,
                "first argument must be of type `Component`",
            ));
        }
        None => unreachable!("argument count already validated"),
    }

    // Validate second argument if present
    if arg_count == 2 {
        match sig.inputs.iter().nth(1) {
            Some(FnArg::Typed(pat)) => validate_props_type(&pat.ty)?,
            Some(FnArg::Receiver(_)) => unreachable!("receiver cannot be second argument"),
            None => unreachable!("argument count already validated"),
        }
    }

    // Validate return type
    match &sig.output {
        ReturnType::Type(_, ty) if is_node_type(ty) => Ok(()),
        ReturnType::Type(_, ty) => Err(syn::Error::new_spanned(
            ty,
            "component functions must return `Node`",
        )),
        ReturnType::Default => Err(syn::Error::new_spanned(
            &sig.ident,
            "component functions must have an explicit return type of `Node`",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_is_component_type() {
        let ty: Type = parse_quote!(Component);
        assert!(is_component_type(&ty));

        let ty: Type = parse_quote!(some::path::Component);
        assert!(is_component_type(&ty));

        let ty: Type = parse_quote!(NotComponent);
        assert!(!is_component_type(&ty));

        let ty: Type = parse_quote!(&Component);
        assert!(!is_component_type(&ty));
    }

    #[test]
    fn test_is_node_type() {
        let ty: Type = parse_quote!(Node);
        assert!(is_node_type(&ty));

        let ty: Type = parse_quote!(some::path::Node);
        assert!(is_node_type(&ty));

        let ty: Type = parse_quote!(NotNode);
        assert!(!is_node_type(&ty));
    }

    #[test]
    fn test_validate_props_type() {
        // Valid props types
        let ty: Type = parse_quote!(MyProps);
        assert!(validate_props_type(&ty).is_ok());

        let ty: Type = parse_quote!(some::module::MyProps);
        assert!(validate_props_type(&ty).is_ok());

        // Invalid props types
        let ty: Type = parse_quote!(MyProps<T>);
        assert!(validate_props_type(&ty).is_err());

        let ty: Type = parse_quote!(&MyProps);
        assert!(validate_props_type(&ty).is_err());

        let ty: Type = parse_quote!((i32, String));
        assert!(validate_props_type(&ty).is_err());
    }

    #[test]
    fn test_validate_component_signature() {
        // Valid: Component only
        let sig: Signature = parse_quote!(fn my_component(c: Component) -> Node);
        assert!(validate_component_signature(&sig).is_ok());

        // Valid: Component with props
        let sig: Signature = parse_quote!(fn my_component(c: Component, props: MyProps) -> Node);
        assert!(validate_component_signature(&sig).is_ok());

        // Invalid: async
        let sig: Signature = parse_quote!(async fn my_component(c: Component) -> Node);
        assert!(validate_component_signature(&sig).is_err());

        // Invalid: generic
        let sig: Signature = parse_quote!(fn my_component<T>(c: Component) -> Node);
        assert!(validate_component_signature(&sig).is_err());

        // Invalid: self
        let sig: Signature = parse_quote!(fn my_component(self) -> Node);
        assert!(validate_component_signature(&sig).is_err());

        // Invalid: no return type
        let sig: Signature = parse_quote!(fn my_component(c: Component));
        assert!(validate_component_signature(&sig).is_err());

        // Invalid: wrong return type
        let sig: Signature = parse_quote!(fn my_component(c: Component) -> String);
        assert!(validate_component_signature(&sig).is_err());

        // Invalid: too many arguments
        let sig: Signature =
            parse_quote!(fn my_component(c: Component, props: MyProps, extra: i32) -> Node);
        assert!(validate_component_signature(&sig).is_err());
    }
}
