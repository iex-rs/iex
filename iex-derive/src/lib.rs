use darling::{ast::NestedMeta, FromAttributes, FromMeta};
use proc_macro2::{Group, Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use syn::{
    parse, parse_macro_input, parse_quote, parse_quote_spanned, parse_str,
    spanned::Spanned,
    visit_mut::{visit_expr_mut, VisitMut},
    Block, Expr, ExprClosure, ExprMethodCall, ExprTry, Ident, ImplItemFn, ItemFn, Lifetime, Macro,
    ReturnType, Signature, Stmt, TraitItemFn, Type,
};

#[derive(FromMeta)]
struct MacroArgs {
    #[darling(multiple)]
    captures: Vec<String>,
}

#[derive(FromAttributes, Debug)]
#[darling(attributes(iex))]
struct MapErrMacroArgs {
    #[darling(multiple)]
    shares: Vec<Ident>,
}

struct ReplaceSelf;

impl VisitMut for ReplaceSelf {
    fn visit_ident_mut(&mut self, node: &mut Ident) {
        if node == "self" {
            *node = Ident::new("iex_self", Span::mixed_site());
        }
    }
    fn visit_macro_mut(&mut self, node: &mut Macro) {
        // Best-effort
        fn visit_token_stream(tokens: TokenStream) -> TokenStream {
            tokens
                .into_iter()
                .map(|tree| match tree {
                    TokenTree::Ident(ident) if ident == "self" => {
                        TokenTree::Ident(Ident::new("iex_self", Span::mixed_site()))
                    }
                    TokenTree::Group(group) => {
                        let mut new_group =
                            Group::new(group.delimiter(), visit_token_stream(group.stream()));
                        new_group.set_span(group.span());
                        TokenTree::Group(new_group)
                    }
                    tree => tree,
                })
                .collect()
        }
        node.tokens = visit_token_stream(node.tokens.clone());
    }
    // Don't recurse into other functions
    fn visit_item_fn_mut(&mut self, _node: &mut ItemFn) {}
    fn visit_impl_item_fn_mut(&mut self, _node: &mut ImplItemFn) {}
    fn visit_trait_item_fn_mut(&mut self, _node: &mut TraitItemFn) {}
}

fn generate_map_inspect_err(
    outcome: &mut Expr,
    closure: &mut Expr,
    attrs: MapErrMacroArgs,
    method: &Ident,
) -> Expr {
    let shares_original = attrs.shares;

    let shares: Vec<_> = shares_original
        .iter()
        .map(|ident| {
            if ident == "self" {
                ReplaceSelf.visit_expr_mut(outcome);
                ReplaceSelf.visit_expr_mut(closure);
                Ident::new("iex_self", Span::mixed_site())
            } else {
                ident.clone()
            }
        })
        .collect();

    let body = if method == "map_err" {
        quote_spanned! { Span::mixed_site() => (#closure)(err) }
    } else if method == "inspect_err" {
        quote_spanned! { Span::mixed_site() => { (#closure)(&err); err } }
    } else {
        unreachable!()
    };

    parse_quote_spanned! {
        Span::mixed_site() => {
            let mut exception_mapper = ::iex::imp::ExceptionMapper::new(
                marker,
                (#(#shares_original,)*),
                |(#(mut #shares,)*), err| #body,
            );
            let marker = exception_mapper.get_in_marker();
            let (#(#shares,)*) = exception_mapper.get_state();
            #(let mut #shares = #shares;)*
            let value = (marker, ::core::mem::ManuallyDrop::new(#outcome))._iex_forward();
            exception_mapper.swallow();
            value
        }
    }
}

fn try_parse_map_inspect_err(expr: &mut Expr) -> darling::Result<Option<Expr>> {
    let Expr::MethodCall(ExprMethodCall {
        receiver: outcome,
        method,
        args,
        ..
    }) = expr
    else {
        return Ok(None);
    };

    if (method != "map_err" && method != "inspect_err") || args.len() != 1 {
        return Ok(None);
    }

    let Expr::Closure(ExprClosure { ref mut attrs, .. }) = args[0] else {
        return Ok(None);
    };

    let parsed_attrs = MapErrMacroArgs::from_attributes(attrs)?;
    if parsed_attrs.shares.is_empty() {
        // Don't accidentally rewrite code that wasn't ours
        return Ok(None);
    }

    attrs.retain(|attr| !attr.path().is_ident("iex"));
    Ok(Some(generate_map_inspect_err(
        outcome,
        &mut args[0],
        parsed_attrs,
        method,
    )))
}

struct ReplaceTry {
    errors: darling::error::Accumulator,
}

impl VisitMut for ReplaceTry {
    fn visit_expr_mut(&mut self, node: &mut Expr) {
        if let Expr::Try(ExprTry { expr, .. }) = node {
            *node = self
                .errors
                .handle_in(|| try_parse_map_inspect_err(expr))
                .unwrap_or(None)
                .unwrap_or_else(|| {
                    parse_quote_spanned! {
                        Span::mixed_site() =>
                        (marker, ::core::mem::ManuallyDrop::new(#expr))._iex_forward()
                    }
                });
        }
        visit_expr_mut(self, node);
    }
    // Don't recurse into other functions or closures
    fn visit_item_fn_mut(&mut self, _node: &mut ItemFn) {}
    fn visit_impl_item_fn_mut(&mut self, _node: &mut ImplItemFn) {}
    fn visit_trait_item_fn_mut(&mut self, _node: &mut TraitItemFn) {}
    fn visit_expr_closure_mut(&mut self, _node: &mut ExprClosure) {}
}

fn transform_trait_item_fn(captures: Vec<Lifetime>, input: TraitItemFn) -> proc_macro::TokenStream {
    // If default is Some(..), the input should have already been parsed as an ItemFn.
    assert!(input.default.is_none());

    let result_type = match input.sig.output {
        ReturnType::Default => parse_quote! { () },
        ReturnType::Type(_, ref result_type) => result_type.clone(),
    };
    let output_type: Type = parse_quote! { <#result_type as ::iex::Outcome>::Output };
    let error_type: Type = parse_quote! { <#result_type as ::iex::Outcome>::Error };
    let to_impl_outcome: ReturnType = parse_quote! {
        -> impl ::iex::Outcome<
            Output = #output_type,
            Error = #error_type,
        > #(+ ::iex::imp::fix_hidden_lifetime_bug::Captures<#captures>)*
    };

    // We used to add '#result_type: ::iex::Outcome' to the 'where' condition. This is wrong for the
    // same reason that *this* fails to typecheck:
    //     trait Trait {
    //         type Exact;
    //     }
    //     impl<T> Trait for T {
    //         type Exact = T;
    //     }
    //     fn f<T: Trait>() {
    //         let x: <T as Trait>::Exact = loop {};
    //         let y: T = x;
    //     }
    let wrapper_sig = Signature {
        output: to_impl_outcome,
        ..input.sig.clone()
    };

    let mut wrapper_attrs = input.attrs.clone();
    wrapper_attrs.insert(0, parse_quote! { #[cfg(not(doc))] });
    let wrapper_fn = TraitItemFn {
        attrs: wrapper_attrs,
        sig: wrapper_sig,
        default: None,
        semi_token: input.semi_token,
    };

    let name = &input.sig.ident;

    let doc = format!(
        "
    <span></span>

    <style>
        body.fn .item-decl code::before, #tymethod\\.{name} .code-header::before {{
            content: '#[iex] ';
        }}
    </style>"
    );
    let mut doc_attrs = input.attrs;
    doc_attrs.insert(0, parse_quote! { #[cfg(doc)] });
    doc_attrs.push(parse_quote! { #[doc = #doc] });
    let doc_fn = TraitItemFn {
        attrs: doc_attrs,
        sig: input.sig,
        default: None,
        semi_token: input.semi_token,
    };

    quote! {
        #wrapper_fn
        #doc_fn
    }
    .into()
}

fn transform_item_fn(captures: Vec<Lifetime>, input: ItemFn) -> proc_macro::TokenStream {
    let input_span = input.span();

    if let Some(constness) = input.sig.constness {
        return quote_spanned! {
            constness.span() => compile_error!("#[iex] does not support const functions");
        }
        .into();
    }
    if let Some(asyncness) = input.sig.asyncness {
        return quote_spanned! {
            asyncness.span() => compile_error!("#[iex] does not support async functions");
        }
        .into();
    }

    let result_type = match input.sig.output {
        ReturnType::Default => parse_quote! { () },
        ReturnType::Type(_, ref result_type) => result_type.clone(),
    };
    let output_type: Type = parse_quote! { <#result_type as ::iex::Outcome>::Output };
    let error_type: Type = parse_quote! { <#result_type as ::iex::Outcome>::Error };
    let to_impl_outcome: ReturnType = parse_quote! {
        -> impl ::iex::Outcome<
            Output = #output_type,
            Error = #error_type,
        > #(+ ::iex::imp::fix_hidden_lifetime_bug::Captures<#captures>)*
    };

    // We used to add '#result_type: ::iex::Outcome' to the 'where' condition. This is wrong for the
    // same reason that *this* fails to typecheck:
    //     trait Trait {
    //         type Exact;
    //     }
    //     impl<T> Trait for T {
    //         type Exact = T;
    //     }
    //     fn f<T: Trait>() {
    //         let x: <T as Trait>::Exact = loop {};
    //         let y: T = x;
    //     }
    let wrapper_sig = Signature {
        output: to_impl_outcome,
        ..input.sig.clone()
    };

    let mut closure_block = input.block;
    let mut replace_try = ReplaceTry {
        errors: darling::Error::accumulator(),
    };
    replace_try.visit_block_mut(&mut closure_block);
    if let Err(err) = replace_try.errors.finish() {
        return err.write_errors().into();
    }

    let no_copy: Ident = parse_quote_spanned! { Span::mixed_site() => no_copy };

    let mut closure: ExprClosure = parse_quote_spanned! {
        Span::mixed_site() => move |marker: ::iex::imp::Marker<#error_type>| {
            let #no_copy = #no_copy; // Force FnOnce inference
            #closure_block
        }
    };

    closure.attrs = input
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("doc") && !attr.path().is_ident("inline"))
        .cloned()
        .collect();
    closure.attrs.insert(0, parse_quote! { #[inline(always)] });

    let name = input.sig.ident.clone();

    // Doc comments must stay in the wrapper even without #[cfg(doc)] because rustc applies the
    // missing_docs lint without cfg(doc).
    let mut wrapper_attrs: Vec<_> = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .cloned()
        .collect();
    wrapper_attrs.extend([
        parse_quote! { #[cfg(not(doc))] },
        parse_quote! {
            #[::iex::imp::fix_hidden_lifetime_bug::fix_hidden_lifetime_bug(
                crate = ::iex::imp::fix_hidden_lifetime_bug
            )]
        },
        // FIXME: removal blocked on
        // https://github.com/danielhenrymantilla/fix_hidden_lifetime_bug.rs/issues/14
        parse_quote! { #[allow(clippy::needless_lifetimes)] },
        parse_quote! { #[inline(always)] },
    ]);

    let inline_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("inline"));
    let wrapper_fn = ItemFn {
        attrs: wrapper_attrs,
        vis: input.vis.clone(),
        sig: wrapper_sig,
        block: parse_quote_spanned! {
            // This span is required for dead code diagnostic
            input_span =>
            {
                #[allow(unused_imports)]
                use ::iex::imp::_IexForward;
                let #no_copy = ::iex::imp::NoCopy; // Force FnOnce inference
                // We need { .. } to support the #[inline] attribute on the closure
                #[allow(unused_mut)]
                let mut #name = { #closure };
                ::iex::imp::IexResult(
                    #inline_attr move |marker| {
                        ::iex::Outcome::get_value_or_panic(#name(marker), marker)
                    },
                    ::core::marker::PhantomData,
                )
            }
        },
    };

    let doc = format!(
        "
    <span></span>

    <style>
        body.fn .item-decl code::before {{
            display: block;
            content: '#[iex]';
        }}
        #method\\.{name} .code-header::before {{
            content: '#[iex] ';
        }}
    </style>"
    );
    let mut doc_attrs = input.attrs;
    doc_attrs.insert(0, parse_quote! { #[cfg(doc)] });
    doc_attrs.push(parse_quote! { #[doc = #doc] });
    let doc_fn = ItemFn {
        attrs: doc_attrs,
        vis: input.vis,
        sig: input.sig,
        block: parse_quote! {{}},
    };

    quote! {
        #wrapper_fn
        #doc_fn
    }
    .into()
}

fn transform_closure(captures: Vec<Lifetime>, input: ExprClosure) -> proc_macro::TokenStream {
    if !captures.is_empty() {
        return quote! {
            compile_error!("#[iex(captures = ..)] is useless on closures")
        }
        .into();
    }

    if let Some(constness) = input.constness {
        return quote_spanned! {
            constness.span() => compile_error!("#[iex] does not support const closures");
        }
        .into();
    }
    if let Some(asyncness) = input.asyncness {
        return quote_spanned! {
            asyncness.span() => compile_error!("#[iex] does not support async closures");
        }
        .into();
    }

    let input_span = input.span();

    let output_type: Type;
    let error_type: Type;
    match input.output {
        ReturnType::Default => {
            output_type = parse_quote! { _ };
            error_type = parse_quote! { _ };
        }
        ReturnType::Type(_, result_type) => {
            output_type = parse_quote! { <#result_type as ::iex::Outcome>::Output };
            error_type = parse_quote! { <#result_type as ::iex::Outcome>::Error };
        }
    }

    let mut closure_body = input.body;
    let mut replace_try = ReplaceTry {
        errors: darling::Error::accumulator(),
    };
    replace_try.visit_expr_mut(&mut closure_body);
    if let Err(err) = replace_try.errors.finish() {
        return err.write_errors().into();
    }
    // Workaround false positive "useless { .. } around return value" warning.
    let closure_body = match *closure_body {
        Expr::Block(block) if block.attrs.is_empty() && block.label.is_none() => block.block.stmts,
        expr => vec![Stmt::Expr(expr, None)],
    };

    let no_copy: Ident = parse_quote_spanned! { Span::mixed_site() => no_copy };
    let closure_ident: Ident = parse_quote_spanned! { Span::mixed_site() => closure };

    let mut internal_closure: ExprClosure = parse_quote_spanned! {
        Span::mixed_site() => move |marker: ::iex::imp::Marker<#error_type>| {
            let #no_copy = #no_copy; // Force FnOnce inference
            #(#closure_body)*
        }
    };

    internal_closure.attrs = input
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("inline"))
        .cloned()
        .collect();
    internal_closure
        .attrs
        .insert(0, parse_quote! { #[inline(always)] });

    let inline_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("inline"));
    let wrapper_closure = ExprClosure {
        attrs: vec![parse_quote! { #[inline(always)] }],
        output: ReturnType::Default,
        body: Box::new(parse_quote_spanned! {
            // This span is required for dead code diagnostic
            input_span =>
            {
                #[allow(unused_imports)]
                use ::iex::imp::_IexForward;
                let #no_copy = ::iex::imp::NoCopy; // Force FnOnce inference
                // We need { .. } to support the #[inline] attribute on the closure
                #[allow(unused_mut)]
                let mut #closure_ident = { #internal_closure };
                ::iex::imp::IexResult::<#output_type, #error_type, _>(
                    #inline_attr move |marker| {
                        ::iex::Outcome::get_value_or_panic(#closure_ident(marker), marker)
                    },
                    ::core::marker::PhantomData,
                )
            }
        }),
        ..input
    };

    quote! { #wrapper_closure }.into()
}

#[proc_macro_attribute]
pub fn iex(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(args) => args,
        Err(e) => return e.into_compile_error().into(),
    };
    let args = match MacroArgs::from_list(&args) {
        Ok(args) => args,
        Err(e) => return e.write_errors().into(),
    };

    let mut captures = Vec::new();
    for capture in args.captures {
        match parse_str::<Lifetime>(&capture) {
            Ok(lifetime) => captures.push(lifetime),
            Err(e) => return e.into_compile_error().into(),
        }
    }

    if let Ok(input) = parse(input.clone()) {
        transform_item_fn(captures, input)
    } else if let Ok(input) = parse(input.clone()) {
        transform_closure(captures, input)
    } else {
        transform_trait_item_fn(captures, parse_macro_input!(input as TraitItemFn))
    }
}

#[proc_macro]
pub fn try_block(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut body = parse_macro_input!(input with Block::parse_within);

    let mut replace_try = ReplaceTry {
        errors: darling::Error::accumulator(),
    };
    for stmt in &mut body {
        replace_try.visit_stmt_mut(stmt);
    }
    if let Err(err) = replace_try.errors.finish() {
        return err.write_errors().into();
    }

    quote_spanned! {
        Span::mixed_site() => {
            #[allow(unused_imports)]
            use ::iex::imp::_IexForward;
            let no_copy = ::iex::imp::NoCopy; // Force FnOnce inference
            ::iex::imp::IexResult(
                {
                    #[inline(always)]
                    move |marker: ::iex::imp::Marker<_>| {
                        let no_copy = no_copy; // Force FnOnce inference
                        #(#body)*
                    }
                },
                ::core::marker::PhantomData,
            )
        }
    }
    .into()
}
