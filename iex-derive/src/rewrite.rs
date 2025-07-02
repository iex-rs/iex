use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use std::collections::HashMap;
use syn::spanned::Spanned;
use syn::{
    Block, Expr, ExprMacro, ExprMethodCall, Label, Lifetime, Stmt, StmtMacro,
    fold::{Fold, fold_expr, fold_stmt},
};

#[derive(Clone, Copy)]
pub enum ErrorType {
    ForReturn,
    ForTry,
}

impl ErrorType {
    // We don't currently support `try` blocks, but if we did, `return_phantom` vs `try_phantom`
    // would be used to accept different error types depending on whether the expression is `e?` or
    // `return e`.
    fn phantom(self) -> TokenStream {
        match self {
            Self::ForReturn => quote_spanned!(Span::mixed_site()=> return_phantom),
            Self::ForTry => quote_spanned!(Span::mixed_site()=> try_phantom),
        }
    }
}

pub struct Rewrite<'ctx> {
    // Can only be `Some` for expressions, expression-like statements, and blocks.
    do_unwrap_value: Option<ErrorType>,

    // Whether to unwrap arguments of `break expr;`
    do_unwrap_last_loop: Option<ErrorType>,

    // Whether to unwrap arguments of `break 'name expr;`
    labeled_loops_to_unwrap: &'ctx mut HashMap<Lifetime, ErrorType>,
}

impl Rewrite<'_> {
    fn with_do_unwrap_value(&mut self, do_unwrap_value: Option<ErrorType>) -> Rewrite<'_> {
        Rewrite {
            do_unwrap_value,
            do_unwrap_last_loop: self.do_unwrap_last_loop,
            labeled_loops_to_unwrap: self.labeled_loops_to_unwrap,
        }
    }

    fn with_loop<T>(
        &mut self,
        label: Option<Label>,
        do_unwrap: Option<ErrorType>,
        callback: impl FnOnce(&mut Rewrite<'_>) -> T,
    ) -> T {
        let old_last = self.do_unwrap_last_loop;
        self.do_unwrap_last_loop = do_unwrap;

        let mut old_labeled = None;
        if let Some(label) = label {
            old_labeled = self.labeled_loops_to_unwrap.remove_entry(&label.name);
            if let Some(error_type) = do_unwrap {
                self.labeled_loops_to_unwrap.insert(label.name, error_type);
            }
        }

        let value = callback(self);

        if let Some((k, v)) = old_labeled {
            self.labeled_loops_to_unwrap.insert(k, v);
        }
        self.do_unwrap_last_loop = old_last;

        value
    }
}

fn stmt_is_expr_like(node: &Stmt) -> bool {
    matches!(
        node,
        Stmt::Expr(_, None)
            | Stmt::Macro(StmtMacro {
                semi_token: None,
                ..
            }),
    )
}

fn generic_unwrap(outcome: Expr, error_type: ErrorType) -> Expr {
    let phantom = error_type.phantom();
    // Insert `Into` conversion only for `e?`, not `return e`.
    let method = match error_type {
        ErrorType::ForReturn => quote_spanned!(outcome.span()=> unwrap_or_throw),
        ErrorType::ForTry => quote_spanned!(outcome.span()=> unwrap_or_throw_with_conversion),
    };
    Expr::Verbatim(quote_spanned! {outcome.span()=> {
        // Cannot use hygiene here because it'll mess up error origin formatting
        let __iex_outcome = #outcome;
        unsafe { ::iex::Outcome::#method(__iex_outcome, #phantom) }
    }})
}

pub fn rewrite_block(block: Block, do_unwrap_value: Option<ErrorType>) -> Block {
    Rewrite {
        do_unwrap_value,
        do_unwrap_last_loop: None,
        labeled_loops_to_unwrap: &mut HashMap::new(),
    }
    .fold_block(block)
}

pub fn rewrite_expr(expr: Expr, do_unwrap_value: Option<ErrorType>) -> Expr {
    Rewrite {
        do_unwrap_value,
        do_unwrap_last_loop: None,
        labeled_loops_to_unwrap: &mut HashMap::new(),
    }
    .fold_expr(expr)
}

impl Fold for Rewrite<'_> {
    fn fold_stmt(&mut self, stmt: Stmt) -> Stmt {
        if !stmt_is_expr_like(&stmt) {
            assert!(
                self.do_unwrap_value.is_none(),
                "this shouldn't have been pushed down",
            );
        }
        match stmt {
            Stmt::Expr(_, _) | Stmt::Local(_) => fold_stmt(self, stmt),
            Stmt::Macro(stmt) if self.do_unwrap_value.is_some() => Stmt::Expr(
                generic_unwrap(
                    Expr::Macro(ExprMacro {
                        attrs: stmt.attrs,
                        mac: stmt.mac,
                    }),
                    self.do_unwrap_value.unwrap(),
                ),
                None,
            ),
            // stop at item boundary or statement-like macro
            Stmt::Item(_) | Stmt::Macro(_) => stmt,
        }
    }

    fn fold_block(&mut self, mut block: Block) -> Block {
        let last_non_item = block
            .stmts
            .iter()
            .enumerate()
            .rev()
            .find(|(_, stmt)| matches!(stmt, Stmt::Expr(_, _) | Stmt::Macro(_)));
        let propagate_value_from_expr = if let Some((i, stmt)) = last_non_item
            && stmt_is_expr_like(stmt)
        {
            Some(i)
        } else {
            None
        };

        block.stmts = block
            .stmts
            .into_iter()
            .enumerate()
            .map(|(i, stmt)| {
                self.with_do_unwrap_value(
                    self.do_unwrap_value
                        .filter(|_| propagate_value_from_expr == Some(i)),
                )
                .fold_stmt(stmt)
            })
            .collect();

        // Add implicit `()` to the end of blocks that don't end with an expression so that we can
        // error on `()?`
        if propagate_value_from_expr.is_none()
            && let Some(error_type) = self.do_unwrap_value
        {
            block.stmts.push(Stmt::Expr(
                generic_unwrap(
                    Expr::Verbatim(quote_spanned!(block.span()=> ())),
                    error_type,
                ),
                None,
            ));
        }

        block
    }

    fn fold_expr(&mut self, expr: Expr) -> Expr {
        match expr {
            // Push unwrapping down if possible
            Expr::Block(mut expr) => {
                expr.block = self.fold_block(expr.block);
                Expr::Block(expr)
            }
            Expr::Group(mut expr) => {
                expr.expr = Box::new(self.fold_expr(*expr.expr));
                Expr::Group(expr)
            }
            Expr::Paren(mut expr) => {
                expr.expr = Box::new(self.fold_expr(*expr.expr));
                Expr::Paren(expr)
            }
            Expr::Unsafe(mut expr) => {
                expr.block = self.fold_block(expr.block);
                Expr::Unsafe(expr)
            }
            Expr::If(mut expr) => {
                expr.cond = Box::new(self.with_do_unwrap_value(None).fold_expr(*expr.cond));
                expr.then_branch = self.fold_block(expr.then_branch);
                expr.else_branch = expr.else_branch.map(|(else_token, else_branch)| {
                    (else_token, Box::new(self.fold_expr(*else_branch)))
                });
                Expr::If(expr)
            }
            Expr::Match(mut expr) => {
                expr.expr = Box::new(self.with_do_unwrap_value(None).fold_expr(*expr.expr));
                expr.arms = expr
                    .arms
                    .into_iter()
                    .map(|mut arm| {
                        arm.guard = arm.guard.map(|(if_token, guard)| {
                            (
                                if_token,
                                Box::new(self.with_do_unwrap_value(None).fold_expr(*guard)),
                            )
                        });
                        arm.body = Box::new(self.fold_expr(*arm.body));
                        arm
                    })
                    .collect();
                Expr::Match(expr)
            }
            // `for` and `while` always return unit, so `do_unwrap_value` should never be set. If
            // it is, the general-purpose implementation should catch that.
            Expr::Loop(mut expr) => {
                self.with_loop(expr.label.clone(), self.do_unwrap_value, |rewrite| {
                    expr.body = rewrite.fold_block(expr.body);
                    Expr::Loop(expr)
                })
            }
            Expr::TryBlock(expr) => Expr::Verbatim(quote_spanned! {expr.span()=>
                compile_error!("#[iex] does not support try blocks")
            }),

            // Don't recurse into closures
            Expr::Closure(expr) => Expr::Closure(expr),

            // Rewrite `?`
            Expr::Try(expr) => {
                let expr = self
                    .with_do_unwrap_value(Some(ErrorType::ForTry))
                    .fold_expr(*expr.expr);
                // If we're supposed to unwrap the result as well, unwrap it the second time
                // generally
                if let Some(error_type) = self.do_unwrap_value {
                    generic_unwrap(expr, error_type)
                } else {
                    expr
                }
            }
            // Rewrite `return` arguments
            Expr::Return(mut expr) => {
                expr.expr = Some(Box::new(
                    self.with_do_unwrap_value(Some(ErrorType::ForReturn))
                        .fold_expr(match expr.expr {
                            Some(value) => *value,
                            None => {
                                // Rewrite `return;` to `return ();` for debug purposes
                                Expr::Verbatim(quote_spanned!(expr.span()=> ()))
                            }
                        }),
                ));
                // `do_unwrap_value` can be ignored because `return` diverges.
                Expr::Return(expr)
            }
            // Rewrite `break`s from `loop`s whose output is unwrapped
            Expr::Break(mut expr) => {
                let do_unwrap_argument = match expr.label {
                    Some(ref label) => self.labeled_loops_to_unwrap.get(label).copied(),
                    None => self.do_unwrap_last_loop,
                };
                expr.expr = expr.expr.map(|expr| {
                    Box::new(
                        self.with_do_unwrap_value(do_unwrap_argument)
                            .fold_expr(*expr),
                    )
                });
                // `do_unwrap_value` can be ignored because `break` diverge.
                Expr::Break(expr)
            }
            // Diverges, can be skipped as an optimization even if `do_unwrap_value` is set.
            Expr::Continue(expr) => Expr::Continue(expr),

            // Unwrapping special methods
            Expr::MethodCall(ExprMethodCall {
                receiver: outcome,
                method,
                mut args,
                ..
            }) if matches!(
                &*method.to_string(),
                "map_err"
                    | "inspect_err"
                    | "context"
                    | "with_context"
                    | "wrap_err"
                    | "wrap_err_with",
            ) && args.len() == 1
                && self.do_unwrap_value.is_some() =>
            {
                let arg = args.pop().unwrap();

                let error_type = self.do_unwrap_value.unwrap();
                let phantom = error_type.phantom();

                let rethrow = match error_type {
                    ErrorType::ForReturn => quote_spanned! {outcome.span()=>
                        __iex_handle.rethrow(__iex_rethrown_err, #phantom)
                    },
                    ErrorType::ForTry => quote_spanned! {outcome.span()=>
                        __iex_handle.rethrow(
                            ::core::convert::Into::into(__iex_rethrown_err),
                            #phantom,
                        )
                    },
                };

                let rethrown_err = match &*method.to_string() {
                    "map_err" => quote_spanned!(method.span()=> (#arg)(__iex_err)),
                    "inspect_err" => quote_spanned! { method.span()=> {
                        (#arg)(&__iex_err);
                        __iex_err
                    }},
                    "context" => quote_spanned! {method.span()=>
                        ::core::result::Result::Err::<(), _>(__iex_err)
                            .context(#arg)
                            .unwrap_err()
                    },
                    "with_context" => quote_spanned! {method.span()=>
                        ::core::result::Result::Err::<(), _>(__iex_err)
                            .with_context(#arg)
                            .unwrap_err()
                    },
                    "wrap_err" => quote_spanned! {method.span()=>
                        ::core::result::Result::Err::<(), _>(__iex_err)
                            .wrap_err(#arg)
                            .unwrap_err()
                    },
                    "wrap_err_with" => quote_spanned! {method.span()=>
                        ::core::result::Result::Err::<(), _>(__iex_err)
                            .wrap_err_with(#arg)
                            .unwrap_err()
                    },
                    _ => unreachable!(),
                };

                Expr::Verbatim(quote_spanned! {outcome.span()=> {
                    // Cannot use hygiene here because it'll mess up error origin formatting
                    let __iex_outcome = #outcome;
                    match unsafe { ::iex::Outcome::intercept(__iex_outcome) } {
                        Ok(__iex_value) => __iex_value,
                        Err((__iex_err, __iex_handle)) => {
                            let __iex_rethrown_err = #rethrown_err;
                            unsafe { #rethrow }
                        }
                    }
                }})
            }

            // General case
            expr => {
                let expr = fold_expr(&mut self.with_do_unwrap_value(None), expr);
                if let Some(error_type) = self.do_unwrap_value {
                    generic_unwrap(expr, error_type)
                } else {
                    expr
                }
            }
        }
    }
}
