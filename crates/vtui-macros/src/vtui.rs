use proc_macro::TokenStream;
use proc_macro2::{Span, TokenTree};
use quote::quote;
use syn::{
    Expr, Ident, Token, braced, ext::IdentExt, parse::{Parse, ParseStream, discouraged::Speculative}, 
    punctuated::Punctuated, token::{Brace, PathSep}
};

// Trait for anything that can provide completions
trait Completable {
    fn completions(&self) -> Vec<proc_macro2::TokenStream>;
}

// Top-level items in the vtui! macro
enum RootItem {
    FlowDirection(syn::Expr),  // Recognized: Flow::Horizontal, Flow::Vertical
    Child(Child),              // Recognized: Header { ... }
    Incomplete(syn::Expr),     // Unrecognized but parseable - provides completions
}

impl Parse for RootItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Peek ahead to see if this is a child component (has braces)
        if input.peek(Ident) && input.peek2(Brace) {
            return Ok(Self::Child(input.parse()?));
        }
        
        // Parse as expression
        let expr = input.parse::<syn::Expr>()?;
        
        // Try to recognize it
        if let Some(enum_name) = is_expr_enum(&expr) {
            match enum_name.as_str() {
                "Flow" => return Ok(Self::FlowDirection(expr)),
                _ => {}
            }
        }
        
        // Unrecognized - but still provide completions
        Ok(Self::Incomplete(expr))
    }
}

impl Completable for RootItem {
    fn completions(&self) -> Vec<proc_macro2::TokenStream> {
        match self {
            Self::FlowDirection(expr) => vec![quote! { #expr; }],
            Self::Child(child) => child.completions(),
            Self::Incomplete(expr) => vec![quote! { #expr; }],
        }
    }
}

// Items that can appear inside a child component
enum ChildItem {
    Measure(syn::Expr),        // Recognized: Measure::Exact(10), Measure::Fill
    Incomplete(syn::Expr),     // Unrecognized but parseable - provides completions
    // Future: Prop(syn::Ident, syn::Expr),
}

impl Parse for ChildItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let expr = input.parse::<syn::Expr>()?;
        
        // Try to recognize it
        if let Some(enum_name) = is_expr_enum(&expr) {
            match enum_name.as_str() {
                "Measure" => return Ok(Self::Measure(expr)),
                _ => {}
            }
        }
        
        // Unrecognized - but still provide completions
        Ok(Self::Incomplete(expr))
    }
}

impl Completable for ChildItem {
    fn completions(&self) -> Vec<proc_macro2::TokenStream> {
        match self {
            Self::Measure(expr) => vec![quote! { #expr; }],
            Self::Incomplete(expr) => vec![quote! { #expr; }],
        }
    }
}

// A child component like Header { Measure::Exact(10) }
struct Child {
    name: syn::Path,
    items: Vec<ChildItem>,
}

impl Parse for Child {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<syn::Path>()?;
        let content;
        braced!(content in input);
        
        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse::<ChildItem>()?);
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }
        
        Ok(Self { name, items })
    }
}

impl Completable for Child {
    fn completions(&self) -> Vec<proc_macro2::TokenStream> {
        let name = &self.name;
        let mut comps = vec![quote! { #name; }];
        
        // Add completions from all child items
        for item in &self.items {
            comps.extend(item.completions());
        }
        
        comps
    }
}

// The root vtui! macro
struct VtuiMacro {
    items: Vec<RootItem>,
}

impl Parse for VtuiMacro {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();
        
        while !input.is_empty() {
            items.push(input.parse::<RootItem>()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        
        Ok(Self { items })
    }
}

impl VtuiMacro {
    pub fn expand(self) -> proc_macro2::TokenStream {
        // Collect all completions from all items
        let completions: Vec<_> = self.items
            .iter()
            .flat_map(|item| item.completions())
            .collect();
        
        quote! {
            mod completions__ {
                fn ignore() {
                    #(#completions)*
                }
            }
            Node::from(c)
        }
    }
}

pub(crate) fn transform_vtui(input: TokenStream) -> TokenStream {
    match syn::parse::<VtuiMacro>(input) {
        Ok(vtui) => vtui.expand().into(),
        Err(err) => {
            let error = err.to_compile_error();
            let tokens = quote! {
                #error
                Node::from(c)
            };
            tokens.into()
        }
    }
}

pub fn is_expr_enum(expr: &Expr) -> Option<String> {
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
