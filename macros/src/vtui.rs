use quote::{ToTokens, quote};
use syn::{
    Token, braced,
    parse::{Parse, ParseStream, discouraged::Speculative},
    punctuated::Punctuated,
    spanned::Spanned,
};

pub(crate) struct NodeInput(Vec<NodeItem>);

impl Parse for NodeInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();

        while !input.is_empty() {
            let item = input.parse::<NodeItem>()?;

            items.push(item);

            let _ = input.parse::<Token![,]>();
        }

        Ok(Self(items))
    }
}

impl NodeInput {
    pub fn expand(self) -> proc_macro2::TokenStream {
        let mut children = Vec::new();
        let mut flows = Vec::new();
        let mut errors = Vec::new();

        for item in self.0 {
            match item {
                NodeItem::Child(child) => {
                    for invalid in child.invalid_exprs {
                        errors.push(invalid)
                    }

                    let props = quote!(());

                    let path = child.path;

                    let measure = match child.measure {
                        Some(measure) => measure.into_token_stream(),
                        None => quote!(Measure::default()),
                    };

                    children.push(quote! {
                        .child(#measure, #path, #props)
                    });
                }
                NodeItem::Flow(e) => flows.push(quote!(.set_flow(#e))),
                NodeItem::Invalid(e) => errors.push(e),
            }
        }

        quote! {
            mod __completions {
                fn ignore() {
                    #(#errors)*
                }
            }

            Node::from(c)
                #(#children)*
                #(#flows)*
        }
    }
}

enum NodeItem {
    Child(NodeChild),
    Flow(syn::Expr),
    Invalid(InvalidExpr),
}

impl Parse for NodeItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Ident) && input.peek2(Token![::]) {
            let fork = input.fork();

            if let Ok(expr) = fork.parse()
                && is_expr_enum(&expr).as_deref() == Some("Flow")
            {
                input.advance_to(&fork);
                return Ok(NodeItem::Flow(expr));
            }
        }

        if input.peek(syn::Ident) && input.peek2(syn::token::Brace) {
            let fork = input.fork();

            if let Ok(child) = fork.parse() {
                input.advance_to(&fork);
                return Ok(Self::Child(child));
            }
        }

        if input.peek(syn::Ident) {
            let fork = input.fork();

            if let Ok(path) = fork.parse() {
                input.advance_to(&fork);
                return Ok(InvalidExpr::Incomplete(path).into());
            }
        }

        let tt: proc_macro2::TokenTree = input.parse()?;

        Ok(InvalidExpr::Invalid(syn::Expr::Verbatim(tt.into_token_stream())).into())
    }
}

impl From<InvalidExpr> for NodeItem {
    fn from(value: InvalidExpr) -> Self {
        Self::Invalid(value)
    }
}

struct NodeChild {
    path: syn::Path,
    measure: Option<syn::Expr>,
    invalid_exprs: Vec<InvalidExpr>,
}

impl Parse for NodeChild {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse::<syn::Path>()?;

        let content;
        braced!(content in input);

        let mut measure = None;
        let mut invalid_exprs = Vec::new();

        while !content.is_empty() {
            let item = content.parse::<NodeChildItem>()?;

            match item {
                NodeChildItem::Measure(expr) => {
                    if measure.is_some() {
                        return Err(syn::Error::new(
                            path.span(),
                            "measure must be defined only once",
                        ));
                    }

                    measure = Some(expr);
                }
                NodeChildItem::Invalid(invalid) => invalid_exprs.push(invalid),
            }

            let _ = content.parse::<Token![,]>();
        }

        Ok(Self {
            path,
            measure,
            invalid_exprs,
        })
    }
}

enum NodeChildItem {
    Measure(syn::Expr),
    Invalid(InvalidExpr),
}

impl From<InvalidExpr> for NodeChildItem {
    fn from(value: InvalidExpr) -> Self {
        Self::Invalid(value)
    }
}

impl Parse for NodeChildItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Ident) && input.peek2(Token![::]) {
            let fork = input.fork();

            if let Ok(expr) = fork.parse()
                && is_expr_enum(&expr).as_deref() == Some("Measure")
            {
                input.advance_to(&fork);
                return Ok(NodeChildItem::Measure(expr));
            }
        }

        if input.peek(syn::Ident) {
            let fork = input.fork();

            if let Ok(path) = fork.parse() {
                input.advance_to(&fork);
                return Ok(InvalidExpr::Incomplete(path).into());
            }
        }

        Ok(InvalidExpr::Invalid(input.parse::<syn::Expr>()?).into())
    }
}

enum InvalidExpr {
    Incomplete(syn::Path),
    Invalid(syn::Expr),
}

impl ToTokens for InvalidExpr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let expr_tokens = match self {
            Self::Incomplete(p) => quote!(#p;),
            Self::Invalid(e) => {
                let err = syn::Error::new(e.span(), "unexpected token").into_compile_error();
                quote! {
                    #err
                    #e;
                }
            }
        };

        tokens.extend(expr_tokens)
    }
}

pub fn is_expr_enum(expr: &syn::Expr) -> Option<String> {
    let path = match expr {
        syn::Expr::Path(p) => &p.path,
        syn::Expr::Call(c) => match &*c.func {
            syn::Expr::Path(p) => &p.path,
            _ => return None,
        },
        _ => return None,
    };
    Some(path.segments.first()?.ident.to_string())
}
