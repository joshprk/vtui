use quote::quote;
use syn::parse::{Parse, ParseStream};

pub(crate) struct ComponentFn {
    func: syn::ItemFn,
    props_type: syn::Type,
}

impl Parse for ComponentFn {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut func = input.parse::<syn::ItemFn>()?;

        if func.sig.inputs.len() == 1 {
            func.sig.inputs.push(syn::parse_quote!(__props: ()));
        }

        let props_type = match func.sig.inputs.iter().nth(1).unwrap() {
            syn::FnArg::Typed(pattern) => *pattern.ty.clone(),
            _ => unreachable!(),
        };

        Ok(Self { func, props_type })
    }
}

impl ComponentFn {
    pub fn expand(self) -> proc_macro2::TokenStream {
        let mut func = self.func;
        let props_type = self.props_type;
        let fn_name = &func.sig.ident;

        let component_ident = match func.sig.inputs.first() {
            Some(syn::FnArg::Typed(pat)) => match &*pat.pat {
                syn::Pat::Ident(ident) => ident.ident.clone(),
                _ => {
                    return syn::Error::new_spanned(
                        &pat.pat,
                        "component parameter must be an identifier",
                    )
                    .into_compile_error();
                }
            },
            _ => {
                return syn::Error::new_spanned(
                    &func.sig,
                    "component must take a Component parameter",
                )
                .into_compile_error();
            }
        };

        let mut new_stmts = Vec::new();
        let block = &mut func.block;

        for stmt in block.stmts.drain(..) {
            if matches!(stmt, syn::Stmt::Macro(ref m) if m.mac.path.is_ident("vtui")) {
                new_stmts.push(syn::parse_quote! {
                    let __vtui_component = #component_ident;
                });
            }
            new_stmts.push(stmt);
        }

        block.stmts = new_stmts;

        quote! {
            const _: () = {
                extern crate vtui as _vtui;

                let _: _vtui::component::FactoryFn<#props_type> = #fn_name;

                const fn _assert_props<T: _vtui::component::Props>() {}
                _assert_props::<#props_type>();
            };

            #[allow(non_snake_case)]
            #func
        }
    }
}
