use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use std::collections::HashMap;
use syn::spanned::Spanned;
use syn::{
    Block, Expr, ExprBlock, ExprMacro, ExprMethodCall, Ident, Label, Lifetime, Stmt, StmtMacro,
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

fn generic_unwrap(outcome: Expr, error_type: ErrorType, expect_divergent: bool) -> Expr {
    // These two are quite different, even though they both are, on the lowest level, just calls to
    // `unwrap_or_throw`. The differences include:
    // - `e?` uses `From` conversion, but `return e` doesn't.
    // - `e?` should error when `e` is `!`, but `return e` should compile.
    // - For `e?`, the expected error type is the error type of the most nested `try` block, while
    //   for `return e`, it's the error type of the most nested function; these can be different. We
    //   don't support `try` blocks yet, but it still makes sense to support these differences.
    // - `e?` and `return e` should emit different diagnostics on type errors.
    // It turns out that there's so little common between the two that it doesn't make sense to
    // share code.
    //
    // Perhaps the strangest thing the two have in common is the use of a mangled `__iex_outcome`
    // name instead of hygiene. It's weird, but for some reason diagnostics are influenced not only
    // by `located_at`, but also by `resolved_at`, so we have to make do with call-site hygiene.
    match error_type {
        ErrorType::ForReturn => unwrap_for_return(outcome, expect_divergent),
        ErrorType::ForTry => unwrap_for_try(outcome),
    }
}

fn unwrap_for_return(outcome: Expr, expect_divergent: bool) -> Expr {
    let return_phantom = quote_spanned!(Span::mixed_site()=> return_phantom);
    let method_call = if expect_divergent {
        // If a block needs to be returned and doesn't end with an expression, we assert that its
        // type is `Result<T, E>`. If it diverges, `!` will correctly coerce to `Result<T, E>`. If
        // it doesn't diverge, the user will get a neat error. This has better behavior than
        // `do_return` if never type fallbacks to `()`, i.e. on edition 2021 or earlier.
        //
        // This is not to be confused with the case when diverging expressions like `panic!()` are
        // returned directly, without being wrapped in a block. That still uses `do_return` and is
        // handled by implementing `Outcome` for `!`. But that only really works well on edition
        // 2024, while this approach works for 2021 as well.
        quote_spanned!(outcome.span()=> #return_phantom.do_return_divergent(__iex_outcome))
    } else {
        // If the type evaluates to `()`, we want to emit different diagnostics depending on the
        // edition of the current crate. If it's 2021 or earlier, `()` could be due to a never type
        // fallback and we can emit a helpful diagnostic to help resolve this case. If it's 2024 or
        // newer, `()` is guaranteed to be a user error and we don't want to show an unhelpful
        // message.
        quote_spanned! {
            outcome.span()=>
            __iex_detect_edition!(
                _,
                #return_phantom.do_return_2021(__iex_outcome),
                #return_phantom.do_return_2024(__iex_outcome),
            )
        }
    };
    Expr::Verbatim(quote_spanned! {outcome.span()=>
        // Clippy is angry at `let ... = <divergent expr>;`, but not at a `match`. This handles `!`
        // being returned gracefully.
        match #outcome {
            #[allow(unreachable_code, unreachable_patterns)]
            __iex_outcome => unsafe { #method_call },
        }
    })
}

fn unwrap_for_try(outcome: Expr) -> Expr {
    let try_phantom = quote_spanned!(Span::mixed_site()=> try_phantom);
    Expr::Verbatim(quote_spanned! {outcome.span()=>
        // Lifetimes of temporaries are extended to the nearest block, so we can't emit a `let`
        // statement and then a function call. Hence we use a single `match` expression here.
        match #outcome {
            __iex_outcome => unsafe { #try_phantom.do_try(__iex_outcome) },
            // This looks strange, and understandably so. This is necessary to support code like
            // `Err(())?;`. Rust normally desugars `e?` to something like
            //     match e {
            //         Ok(x) => x,
            //         Err(e) => return Err(From::from(e)),
            //     }
            // ...so even if the type of `x` is a free variable, it gets unified with the type of
            // `return ...`, i.e. `!`, and so `T = !` is inferred and the code compiles. But if we
            // simply call `do_try`, we'll just have an inference error because the type variable
            // remains free. So we need to unify the value with `!` as well.
            #[allow(unreachable_patterns)]
            _ => unsafe { ::core::hint::unreachable_unchecked() },
        }
    })
}

fn unwrap_special_method(outcome: Expr, error_type: ErrorType, method: Ident, arg: Expr) -> Expr {
    // We need to intercept the exception in `outcome`, if present, and then map the error and
    // rethrow it. It's more tricky than calling a method like `do_try_with_map_err`, though.
    // Consider a snippet like:
    //     f(&mut x).map_err(|e| x.method_taking_mut_self(e))?;
    // If `f(&mut x)` is `IexResult` here, the closure stored inside it captures `x`, and so does
    // the closure in the `map_err` argument. These captures end up aliasing, and borrowck
    // rightfully rejects such code. We have to make sure the `IexResult` closure has already been
    // invoked by the time the `map_err` argument is created, resulting in something like this:
    //     let outcome = f(&mut x);
    //     let intercepted = outcome.intercept();
    //     match intercepted {
    //         Ok(value) => value,
    //         Err((error, handle)) => handle.rethrow(x.method_taking_mut_self(error)),
    //     }
    //
    // It's also important not to change semantics by only evaluating the closure expression in
    // `Err` case, so the expansion is closer to:
    //     let outcome = f(&mut x);
    //     let intercepted = outcome.intercept();
    //     let map = |e| x.method_taking_mut_self(e);
    //     match intercepted {
    //         Ok(value) => value,
    //         Err((error, handle)) => handle.rethrow(map(error)),
    //     }
    //
    // This leaves out just one detail. `e.map_err(..)?` needs to activate never type fallback, and
    // that happens implicitly here because `rethrow` returns `!`. But `return e.map_err(..)` needs
    // to avoid such type inference. We achieve this by making `rethrow` generic over the return
    // type, i.e. `fn rethrow<T>(..) -> T` with `T` otherwise unmentioned, and this is enough to
    // disable never type fallback simulation.

    let rethrown_err = match &*method.to_string() {
        "map_err" => quote_spanned!(method.span()=> __iex_arg(__iex_err)),
        "inspect_err" => quote_spanned! {method.span()=> {
            __iex_arg(&__iex_err);
            __iex_err
        }},
        "context" => quote_spanned! {method.span()=>
            ::core::result::Result::Err::<(), _>(__iex_err)
                .context(__iex_arg)
                .unwrap_err()
        },
        "with_context" => quote_spanned! {method.span()=>
            ::core::result::Result::Err::<(), _>(__iex_err)
                .with_context(__iex_arg)
                .unwrap_err()
        },
        "wrap_err" => quote_spanned! {method.span()=>
            ::core::result::Result::Err::<(), _>(__iex_err)
                .wrap_err(__iex_arg)
                .unwrap_err()
        },
        "wrap_err_with" => quote_spanned! {method.span()=>
            ::core::result::Result::Err::<(), _>(__iex_err)
                .wrap_err_with(__iex_arg)
                .unwrap_err()
        },
        _ => unreachable!(),
    };

    // We can be generic over the error type here because the behavior of `!` is equivalent between
    // `e?` and `return e` in this case, since these methods don't exist on `!`.
    let phantom = error_type.phantom();

    // Just like in `generic_unwrap`, we use `match` instead of `let` due to temporary lifetime
    // extension.
    Expr::Verbatim(quote_spanned! {outcome.span()=>
        match #outcome {
            __iex_outcome => match (
                unsafe { ::iex::intercept(__iex_outcome) },
                #arg // needs to be evaluated after outcome is intercepted
            ) {
                (::core::result::Result::Ok(__iex_value), _) => __iex_value,
                (::core::result::Result::Err((__iex_err, __iex_handle)), mut __iex_arg) => { // XXX: we need to fix type inference to remove mut here :/
                    let __iex_err = #rethrown_err;
                    unsafe { #phantom.rethrow(__iex_err, __iex_handle) }
                }
            }
        }
    })
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
                    false,
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
        let tail_expression = if let Some((i, stmt)) = last_non_item
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
                    self.do_unwrap_value.filter(|_| tail_expression == Some(i)),
                )
                .fold_stmt(stmt)
            })
            .collect();

        // If the block doesn't have a tail expression, we have to unwrap it directly. It's a bit
        // ugly because we need to return a block here and `generic_unwrap` returns an expression,
        // but otherwise it's straightforward.
        //
        // The only tricky thing is that we tell `generic_unwrap` we expect the expression to
        // diverge to get good behavior on edition 2021 and earlier, where never type would
        // otherwise fallback to `()` and cause a type error in valid code.
        if tail_expression.is_none()
            && let Some(error_type) = self.do_unwrap_value
        {
            block = Block {
                brace_token: block.brace_token.clone(),
                stmts: vec![Stmt::Expr(
                    generic_unwrap(
                        Expr::Block(ExprBlock {
                            attrs: Vec::new(),
                            label: None,
                            block,
                        }),
                        error_type,
                        true,
                    ),
                    None,
                )],
            }
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
                    generic_unwrap(expr, error_type, false)
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
                unwrap_special_method(*outcome, error_type, method, arg.into_value())
            }

            // General case
            expr => {
                let expr = fold_expr(&mut self.with_do_unwrap_value(None), expr);
                if let Some(error_type) = self.do_unwrap_value {
                    generic_unwrap(expr, error_type, false)
                } else {
                    expr
                }
            }
        }
    }
}
