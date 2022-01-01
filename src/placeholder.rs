use std::fmt::{self, Display};
use std::sync::atomic::{AtomicUsize, Ordering};

use proc_macro2::TokenStream;
use quote::{format_ident, IdentFragment, ToTokens};
use syn::{ext::IdentExt, visit_mut::VisitMut};

static NEXT_PLACEHOLDER_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct PlaceholderId(usize);

impl IdentFragment for PlaceholderId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, fmt)
    }
}

impl PlaceholderId {
    pub fn new() -> PlaceholderId {
        PlaceholderId(NEXT_PLACEHOLDER_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub fn to_ident(self) -> syn::Ident {
        format_ident!("__cain_placeholder__{}", self)
    }
}

impl ToTokens for PlaceholderId {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.to_ident().to_tokens(tokens)
    }
}

pub fn replace_with_placeholder(expr: &mut syn::Expr) -> (PlaceholderId, syn::Expr) {
    let id = PlaceholderId::new();
    let placeholder_expr = syn::Expr::Path(syn::ExprPath {
        attrs: Vec::new(),
        qself: None,
        path: id.to_ident().into(),
    });

    let expr = std::mem::replace(expr, placeholder_expr);
    (id, expr)
}

fn get_placeholder_id(path: &syn::Path) -> Option<PlaceholderId> {
    if path.leading_colon.is_none() && path.segments.len() == 1 {
        let segment = &path.segments[0];

        if segment.arguments.is_empty() {
            let ident = segment.ident.unraw().to_string();

            if let Some(placeholder_id) = ident.strip_prefix("__cain_placeholder__") {
                if let Ok(placeholder_id) = placeholder_id.parse() {
                    return Some(PlaceholderId(placeholder_id));
                }
            }
        }
    }

    None
}

fn replace_expr(
    mut expr: syn::Expr,
    placeholder_id: PlaceholderId,
    target: syn::Expr,
) -> syn::Result<syn::Expr> {
    let mut visitor = ReplaceVisitor::new(|id| {
        if id == placeholder_id {
            Some(target.clone())
        } else {
            None
        }
    });

    visitor.visit_expr_mut(&mut expr);
    visitor.into_error().map(|()| expr)
}

pub fn wrap_placeholder_expr_mut(
    expr: &mut syn::Expr,
    placeholder_id: PlaceholderId,
    outer: syn::Expr,
) -> syn::Result<()> {
    *expr = replace_expr(outer, placeholder_id, expr.clone())?;
    Ok(())
}

pub fn wrap_placeholder_block_mut(
    block: &mut syn::Block,
    placeholder_id: PlaceholderId,
    outer: syn::Expr,
) -> syn::Result<()> {
    let target = syn::Expr::Block(syn::ExprBlock {
        attrs: Vec::new(),
        label: None,
        block: block.clone(),
    });

    let expr = replace_expr(outer, placeholder_id, target)?;

    *block = syn::Block {
        brace_token: Default::default(),
        stmts: vec![syn::Stmt::Expr(expr)],
    };
    Ok(())
}

struct ReplaceVisitor<F> {
    lookup_func: F,
    error: Option<syn::Error>,
}

impl<F> ReplaceVisitor<F> {
    fn new(lookup_func: F) -> ReplaceVisitor<F> {
        ReplaceVisitor {
            lookup_func,
            error: None,
        }
    }

    fn into_error(self) -> syn::Result<()> {
        match self.error {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    fn fail<T, D>(&mut self, tokens: T, message: D)
    where
        T: ToTokens,
        D: Display,
    {
        if self.error.is_none() {
            self.error = Some(syn::Error::new_spanned(tokens, message))
        }
    }
}

impl<F> VisitMut for ReplaceVisitor<F>
where
    F: Fn(PlaceholderId) -> Option<syn::Expr>,
{
    fn visit_expr_mut(&mut self, i: &mut syn::Expr) {
        match i {
            syn::Expr::Path(syn::ExprPath {
                attrs,
                qself: None,
                path,
            }) => {
                if let Some(placeholder_id) = get_placeholder_id(path) {
                    if !attrs.is_empty() {
                        self.fail(
                            i,
                            "internal macro error: placeholder may not have attributes",
                        );
                        return;
                    }

                    if let Some(expr) = (self.lookup_func)(placeholder_id) {
                        *i = expr;
                    }
                } else {
                    syn::visit_mut::visit_expr_mut(self, i)
                }
            }

            _ => syn::visit_mut::visit_expr_mut(self, i),
        }
    }
}
