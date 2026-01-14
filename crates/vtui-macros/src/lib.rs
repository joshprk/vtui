use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, Signature, parse_macro_input};

fn is_mut_component_ref(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Reference(r) if r.mutability.is_some() => {
            matches!(
                &*r.elem,
                syn::Type::Path(p)
                if p.path
                    .segments
                    .last()
                    .map(|s| s.ident == "Component")
                    .unwrap_or(false)
            )
        }
        _ => false,
    }
}

fn is_inner_return(ty: &syn::Type) -> bool {
    matches!(
        ty,
        syn::Type::Path(p)
            if p.qself.is_none()
                && p.path.segments.last().is_some_and(|seg| {
                    seg.ident == "Inner"
                        && matches!(seg.arguments, syn::PathArguments::None)
                })
    )
}

fn validate_props_type(ty: &syn::Type) -> syn::Result<()> {
    match ty {
        syn::Type::Path(p)
            if p.qself.is_none()
                && p.path.segments.iter().all(|seg| {
                    matches!(seg.arguments, syn::PathArguments::None)
                }) =>
        {
            Ok(())
        }
        _ => Err(syn::Error::new_spanned(
            ty,
            "component props must be a concrete, non-generic, owned type implementing `Props`",
        )),
    }
}

fn validate_component_sig(sig: &Signature) -> syn::Result<()> {
    if sig.asyncness.is_some() {
        return Err(syn::Error::new_spanned(
            sig.asyncness,
            "component functions must not be async",
        ));
    }

    if !sig.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &sig.generics,
            "component functions must not be generic",
        ));
    }

    match sig.inputs.len() {
        1 | 2 => {}
        _ => {
            return Err(syn::Error::new_spanned(
                &sig.inputs,
                "component functions must take `&mut Component` or `&mut Component, Props`",
            ));
        }
    }

    match sig.inputs.first().unwrap() {
        syn::FnArg::Typed(pat) if is_mut_component_ref(&pat.ty) => {}
        syn::FnArg::Receiver(recv) => {
            return Err(syn::Error::new_spanned(
                recv,
                "component functions must not take `self`",
            ));
        }
        _ => {
            return Err(syn::Error::new_spanned(
                &sig.inputs,
                "first argument must be `&mut Component`",
            ));
        }
    }

    if sig.inputs.len() == 2 {
        match sig.inputs.iter().nth(1).unwrap() {
            syn::FnArg::Typed(pat) => {
                validate_props_type(&pat.ty)?;
            }
            _ => unreachable!(),
        }
    }

    match &sig.output {
        syn::ReturnType::Type(_, ty) if is_inner_return(ty) => {}
        syn::ReturnType::Type(_, ty) => {
            return Err(syn::Error::new_spanned(
                ty,
                "component functions must return `Inner`",
            ));
        }
        syn::ReturnType::Default => {
            return Err(syn::Error::new_spanned(
                &sig.ident,
                "component functions must return `Inner`",
            ));
        }
    }

    Ok(())
}

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(item as ItemFn);

    if let Err(err) = validate_component_sig(&func.sig) {
        return err.to_compile_error().into();
    }

    // If only (&mut Component) is provided, append a dummy props argument of type ()
    if func.sig.inputs.len() == 1 {
        func.sig.inputs.push(syn::parse_quote! {
            _: ()
        });
    }

    // Enforce Props bound on the second argument via where-clause
    if func.sig.inputs.len() == 2 {
        let props_ty = match func.sig.inputs.iter().nth(1).unwrap() {
            syn::FnArg::Typed(pat) => &pat.ty,
            _ => unreachable!(),
        };

        func.sig
            .generics
            .where_clause
            .get_or_insert_with(|| syn::parse_quote!(where))
            .predicates
            .push(syn::parse_quote! {
                #props_ty: ::vtui::prelude::Props
            });
    }

    quote! {
        #[allow(non_snake_case)]
        #func
    }
    .into()
}

#[proc_macro]
pub fn vtui(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        Inner::default()
    };
    expanded.into()
}
