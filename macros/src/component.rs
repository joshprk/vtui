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
            func.sig.inputs.push(syn::parse_quote!(_: ()));
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
        let func = self.func;
        let props_type = self.props_type;
        let fn_name = &func.sig.ident;

        quote! {
            const _: () = {
                extern crate vtui as _vtui;

                let _: _vtui::prelude::Factory<#props_type> = #fn_name;

                const fn _assert_props<T: _vtui::prelude::Props>() {}
                _assert_props::<#props_type>();
            };

            #[allow(non_snake_case)]
            #func
        }
    }
}
