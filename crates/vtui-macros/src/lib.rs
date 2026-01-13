use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, Signature, parse_macro_input};

fn is_mut_component_ref(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Reference(r) => {
            r.mutability.is_some()
                && matches!(&*r.elem, syn::Type::Path(p) if p.path.is_ident("Component"))
        }
        _ => false,
    }
}

fn validate_component_sig(sig: &Signature) -> syn::Result<()> {
    // no async
    if sig.asyncness.is_some() {
        return Err(syn::Error::new_spanned(
            sig.asyncness,
            "component functions must not be async",
        ));
    }

    // no generics
    if !sig.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &sig.generics,
            "component functions must not be generic",
        ));
    }

    // exactly one argument
    if sig.inputs.len() != 1 {
        return Err(syn::Error::new_spanned(
            &sig.inputs,
            "component functions must take exactly one argument: `&mut Component`",
        ));
    }

    // argument must be `&mut Component`
    match sig.inputs.first().unwrap() {
        syn::FnArg::Typed(pat) => {
            if !is_mut_component_ref(&pat.ty) {
                return Err(syn::Error::new_spanned(
                    &pat.ty,
                    "expected argument type `&mut Component`",
                ));
            }
        }
        syn::FnArg::Receiver(_) => {
            return Err(syn::Error::new_spanned(
                &sig.inputs,
                "component functions must not take `self`",
            ));
        }
    }

    Ok(())
}

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);

    if let Err(err) = validate_component_sig(&func.sig) {
        return err.to_compile_error().into();
    }

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

#[proc_macro]
pub fn vtui(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        Inner::default()
    };
    expanded.into()
}
