use std::collections::BTreeMap;
use std::iter::once;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::visit_mut::VisitMut;
use syn::Block;

use crate::placeholder::{
    replace_with_placeholder, wrap_placeholder_block_mut, wrap_placeholder_expr_mut, PlaceholderId,
};
use crate::util::{drain_filter, unique_ident};

pub fn cain(input: TokenStream) -> syn::Result<TokenStream> {
    let stmts = Block::parse_within.parse2(input)?;
    let stmts = chain_stmts(stmts)?;

    // wrap the result in a block expression
    Ok(quote! {
        { #(#stmts)* }
    })
}

fn chain_stmts(mut stmts: Vec<syn::Stmt>) -> syn::Result<Vec<syn::Stmt>> {
    let mut items = drain_filter(&mut stmts, |stmt| matches!(stmt, syn::Stmt::Item(_)));

    let stmts = stmts.into_iter().rev().try_fold(Vec::new(), chain_stmt)?;

    items.extend(stmts);

    Ok(items)
}

fn chain_stmt(rest: Vec<syn::Stmt>, stmt: syn::Stmt) -> syn::Result<Vec<syn::Stmt>> {
    match stmt {
        syn::Stmt::Expr(expr) => {
            let expr = chain_expr(expr, None)?;
            Ok(once(syn::Stmt::Expr(expr)).chain(rest).collect())
        }

        syn::Stmt::Semi(expr, semi) => {
            let expr = chain_expr(expr, None)?;
            Ok(once(syn::Stmt::Semi(expr, semi)).chain(rest).collect())
        }

        syn::Stmt::Local(mut local) => {
            if let Some((_, init)) = &mut local.init {
                let (placeholder_id, init_expr) = replace_with_placeholder(init);

                let inner_expr: syn::Expr = syn::parse_quote! {
                    { #local #(#rest)* }
                };

                let expr = chain_expr(init_expr, Some((placeholder_id, inner_expr)))?;

                if let syn::Expr::Block(expr_block) = expr {
                    Ok(expr_block.block.stmts)
                } else {
                    Ok(vec![syn::Stmt::Expr(expr)])
                }
            } else {
                Ok(once(syn::Stmt::Local(local)).chain(rest).collect())
            }
        }

        syn::Stmt::Item(_) => unreachable!(),
    }
}

fn chain_expr(
    mut expr: syn::Expr,
    wrap_expr: Option<(PlaceholderId, syn::Expr)>,
) -> syn::Result<syn::Expr> {
    let mut visitor = Visitor::default();
    visitor.visit_expr_mut(&mut expr);

    if let Some(err) = visitor.error {
        return Err(err);
    }

    if let Some((placeholder_id, wrap_expr)) = wrap_expr {
        wrap_placeholder_expr_mut(&mut expr, placeholder_id, wrap_expr)?;
    }

    visitor
        .branches
        .into_iter()
        .rev()
        .try_fold(expr, |expr, (branch_id, mut branch_expr)| {
            match &mut branch_expr {
                syn::Expr::Match(match_expr) => {
                    for arm in &mut match_expr.arms {
                        let mut pat_idents = BTreeMap::new();
                        replace_pat_idents(&mut arm.pat, &mut pat_idents)?;

                        if !pat_idents.is_empty() {
                            let expr_bindings = pat_idents.iter().map(
                                |(old, (new, mutability))| quote! { let #mutability #old = #new; },
                            );

                            let arm_body = &arm.body;
                            arm.body = syn::parse_quote! {
                                {
                                    #( #expr_bindings )*
                                    #arm_body
                                }
                            };
                        }

                        wrap_placeholder_expr_mut(&mut arm.body, branch_id, expr.clone())?;
                    }
                }

                syn::Expr::If(if_expr) => {
                    if let syn::Expr::Let(expr_let) = &mut *if_expr.cond {
                        let mut pat_idents = BTreeMap::new();
                        replace_pat_idents(&mut expr_let.pat, &mut pat_idents)?;

                        if !pat_idents.is_empty() {
                            let expr_bindings = pat_idents.iter().map(
                                |(old, (new, mutability))| quote! { let #mutability #old = #new; },
                            );

                            let then_branch = &if_expr.then_branch;
                            if_expr.then_branch = syn::parse_quote! {
                                {
                                    #( #expr_bindings )*
                                    #then_branch
                                }
                            };
                        }
                    }

                    wrap_placeholder_block_mut(&mut if_expr.then_branch, branch_id, expr.clone())?;

                    if let Some((_, else_branch)) = &mut if_expr.else_branch {
                        wrap_placeholder_expr_mut(else_branch, branch_id, expr)?;
                    }
                }

                _ => unreachable!(),
            }

            Ok(branch_expr)
        })
}

#[derive(Default)]
struct Visitor {
    branches: Vec<(PlaceholderId, syn::Expr)>,
    error: Option<syn::Error>,
}

impl Visitor {
    fn fail(&mut self, error: syn::Error) {
        self.error.get_or_insert(error);
    }
}

impl VisitMut for Visitor {
    fn visit_block_mut(&mut self, i: &mut Block) {
        i.stmts = match chain_stmts(i.stmts.clone()) {
            Ok(stmts) => stmts,
            Err(err) => return self.fail(err),
        };
    }

    fn visit_expr_closure_mut(&mut self, i: &mut syn::ExprClosure) {
        i.body = match chain_expr((*i.body).clone(), None) {
            Ok(expr) => Box::new(expr),
            Err(err) => return self.fail(err),
        };
    }

    fn visit_expr_mut(&mut self, i: &mut syn::Expr) {
        match i {
            syn::Expr::Match(_) => {
                let (branch_id, mut expr) = replace_with_placeholder(i);
                let match_expr = match &mut expr {
                    syn::Expr::Match(match_expr) => match_expr,
                    _ => unreachable!(),
                };

                self.visit_expr_match_mut(match_expr);

                self.branches.push((branch_id, expr));
            }

            syn::Expr::If(_) => {
                let (branch_id, mut expr) = replace_with_placeholder(i);
                let if_expr = match &mut expr {
                    syn::Expr::If(if_expr) => if_expr,
                    _ => unreachable!(),
                };

                self.visit_expr_if_mut(if_expr);

                self.branches.push((branch_id, expr));
            }

            _ => syn::visit_mut::visit_expr_mut(self, i),
        }
    }
}

fn replace_pat_idents(
    pat: &mut syn::Pat,
    ident_map: &mut BTreeMap<syn::Ident, (syn::Ident, Option<syn::token::Mut>)>,
) -> syn::Result<()> {
    match pat {
        syn::Pat::Ident(pat_ident) => {
            let (ident, mutability) = ident_map
                .entry(pat_ident.ident.clone())
                .or_insert_with(|| (unique_ident(), None));

            pat_ident.ident = ident.clone();

            if mutability.is_none() {
                *mutability = pat_ident.mutability.take();
            }

            if let Some((_, subpat)) = &mut pat_ident.subpat {
                replace_pat_idents(&mut *subpat, ident_map)?
            }

            Ok(())
        }

        syn::Pat::Lit(_)
        | syn::Pat::Path(_)
        | syn::Pat::Range(_)
        | syn::Pat::Rest(_)
        | syn::Pat::Wild(_) => Ok(()),

        syn::Pat::Box(pat) => replace_pat_idents(&mut pat.pat, ident_map),
        syn::Pat::Or(pat_or) => pat_or
            .cases
            .iter_mut()
            .try_for_each(|pat| replace_pat_idents(pat, ident_map)),
        syn::Pat::Reference(pat_ref) => replace_pat_idents(&mut pat_ref.pat, ident_map),
        syn::Pat::Slice(pat_slice) => pat_slice
            .elems
            .iter_mut()
            .try_for_each(|pat| replace_pat_idents(pat, ident_map)),
        syn::Pat::Struct(pat_struct) => pat_struct
            .fields
            .iter_mut()
            .try_for_each(|pat_field| replace_pat_idents(&mut pat_field.pat, ident_map)),
        syn::Pat::Tuple(pat_tuple) => pat_tuple
            .elems
            .iter_mut()
            .try_for_each(|pat| replace_pat_idents(pat, ident_map)),
        syn::Pat::TupleStruct(pat_tuple_struct) => pat_tuple_struct
            .pat
            .elems
            .iter_mut()
            .try_for_each(|pat| replace_pat_idents(pat, ident_map)),
        syn::Pat::Type(pat_type) => replace_pat_idents(&mut pat_type.pat, ident_map),

        syn::Pat::Macro(_) => Err(syn::Error::new_spanned(
            pat,
            "cain! does not support macros in patterns",
        )),
        _ => Err(syn::Error::new_spanned(
            pat,
            "cain! does not support this pattern",
        )),
    }
}
