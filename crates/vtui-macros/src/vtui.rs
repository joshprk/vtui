use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::{Expr, Ident, Token, braced, parse::{ParseStream, discouraged::Speculative}, punctuated::Punctuated};

enum Item {
    Flow(Expr),
    Layer(Expr),
    Child(Child),
}

struct Child {
    path: syn::Path,
    measure: Option<syn::Expr>,
}

impl syn::parse::Parse for Child {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path: syn::Path = input.parse()?;

        let content;
        braced!(content in input);

        let mut measure = None;

        let exprs = Punctuated::<Expr, Token![,]>::parse_terminated(&content)?;

        for expr in exprs {
            if let Some(enum_name) = is_enum_attr(&expr) && enum_name == "Measure" {
                measure = Some(expr);
            } else {
                return Err(syn::Error::new_spanned(expr, "unexpected token"));
            }
        }

        Ok(Self { path, measure })
    }
}

#[derive(Default)]
struct NodeDescription {
    items: Vec<Item>,
}

impl syn::parse::Parse for NodeDescription {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();

        while !input.is_empty() {
            let fork = input.fork();

            if let Ok(child) = fork.parse::<Child>() {
                input.advance_to(&fork);
                items.push(Item::Child(child));
            } else {
                let expr: Expr = input.parse()?;

                if let Some(enum_name) = is_enum_attr(&expr) && enum_name == "Flow" {
                    items.push(Item::Flow(expr));
                } else {
                    return Err(syn::Error::new_spanned(expr, "unexpected token"));
                }
            }

            let _ = input.parse::<Token![,]>();
        }

        Ok(Self { items })
    }
}

impl NodeDescription {
    pub fn expand(self) -> TokenStream {
        let calls = self.items.into_iter().map(|item| {
            match item {
                Item::Flow(expr) => {
                    quote! { .set_flow(#expr) }
                },
                Item::Child(child) => {
                    let factory = child.path;
                    let measure = match child.measure {
                        Some(m) => quote! { #m },
                        None => quote! { Measure::Exact(10) },
                    };

                    quote! { .child(#measure, #factory, ()) }
                },
                _ => {
                    quote! { }
                },
            }
        });

        let stream = quote! {
            Node::from(c)
                #(#calls)*
        };

        stream.into()
    }
}

struct Completion {
    last_ident: Option<Ident>,
}

impl syn::parse::Parse for Completion {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut last_ident = None;

        while !input.is_empty() {
            // Capture identifiers
            if let Ok(ident) = input.parse::<Ident>() {
                last_ident = Some(ident);
            }
            // Recurse into braces
            else if input.peek(syn::token::Brace) {
                let content;
                braced!(content in input);

                if let Ok(inner) = content.parse::<Completion>() {
                    if inner.last_ident.is_some() {
                        last_ident = inner.last_ident;
                    }
                }
            }
            // Skip expressions
            else if input.parse::<Expr>().is_ok() {
                // ignore
            }
            // Skip commas
            else if input.parse::<Token![,]>().is_ok() {
                // ignore
            }
            // Fallback: consume one token unconditionally
            else if input.parse::<TokenTree>().is_ok() {
                // ignore
            }
            // Should never happen, but guarantees termination
            else {
                break;
            }
        }

        Ok(Self { last_ident })
    }
}

pub(crate) fn transform_vtui(input: TokenStream) -> TokenStream {
    match syn::parse::<NodeDescription>(input.clone()) {
        Ok(desc) => desc.expand(),
        Err(err) => {
            let error = err.to_compile_error();
            let completion = syn::parse::<Completion>(input).unwrap();

            let last_ident = completion.last_ident;
            let context = quote! {
                mod completions__ {
                    use vtui::prelude::Flow;
                    fn complete() {
                        #last_ident;
                    }
                }
            };

            let tokens = quote! {
                #error
                #context
                Node::from(c)
            };

            tokens.into()
        }
    }
}

pub fn is_enum_attr(expr: &Expr) -> Option<String> {
    let path = match expr {
        Expr::Path(p) => &p.path,
        Expr::Call(c) => match &*c.func {
            Expr::Path(p) => &p.path,
            _ => return None,
        },
        _ => return None,
    };

    Some(path.segments.first()?.ident.to_string())
}
