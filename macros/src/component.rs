use quote::quote;
use syn::parse::{Parse, ParseStream};

pub(crate) struct ComponentFn {
    func: syn::ItemFn,
}

impl Parse for ComponentFn {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let func = input.parse::<syn::ItemFn>()?;
        if func.sig.inputs.is_empty() {
            return Err(syn::Error::new_spanned(
                &func.sig,
                "expected a Component parameter",
            ));
        }
        for arg in &func.sig.inputs {
            match arg {
                syn::FnArg::Typed(arg) if matches!(&*arg.pat, syn::Pat::Ident(pat) if pat.by_ref.is_none() && pat.subpat.is_none()) =>
                    {}
                _ => return Err(syn::Error::new_spanned(arg, "expected a named parameter")),
            }
        }
        if func.sig.asyncness.is_some()
            || func.sig.constness.is_some()
            || func.sig.unsafety.is_some()
            || func.sig.abi.is_some()
            || func.sig.variadic.is_some()
        {
            return Err(syn::Error::new_spanned(
                &func.sig,
                "expected a plain component function",
            ));
        }
        Ok(Self { func })
    }
}

impl ComponentFn {
    pub fn expand(self) -> proc_macro2::TokenStream {
        let func = self.func;
        let name = &func.sig.ident;
        let vis = &func.vis;
        let attrs = &func.attrs;
        let generics = &func.sig.generics;
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let context = func.sig.inputs.first().unwrap();
        let props: Vec<_> = func
            .sig
            .inputs
            .iter()
            .skip(1)
            .map(|arg| {
                let syn::FnArg::Typed(arg) = arg else {
                    unreachable!()
                };
                let syn::Pat::Ident(pat) = &*arg.pat else {
                    unreachable!()
                };
                (pat, &arg.ty)
            })
            .collect();
        let fields = props.iter().map(|(pat, ty)| {
            let ident = &pat.ident;
            quote! { #vis #ident: #ty }
        });
        let bindings = props.iter().map(|(pat, _)| pat);
        let definition = if props.is_empty() {
            quote! { #vis struct #name #generics #where_clause; }
        } else {
            quote! { #vis struct #name #generics #where_clause { #(#fields),* } }
        };
        let body = &func.block.stmts;
        let output = &func.sig.output;

        quote! {
            #(#attrs)*
            #[derive(Clone)]
            #definition

            impl #impl_generics ::vtui::element::Element for #name #ty_generics #where_clause {
                fn run(self, #context) #output {
                    let Self { #(#bindings),* } = self;
                    #(#body)*
                }
            }
        }
    }
}
