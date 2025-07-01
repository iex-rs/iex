use crate::rewrite::{ErrorType, rewrite_block, rewrite_expr};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    Expr, ExprClosure, Ident, ItemFn, ReturnType, TraitItemFn, parse_quote, parse_quote_spanned,
    spanned::Spanned,
};

pub fn transform_trait_item_fn(input: TraitItemFn) -> TokenStream {
    // If default is Some(..), the input should have already been parsed as an ItemFn.
    assert!(input.default.is_none());

    let mut wrapper_attrs = input.attrs.clone();
    wrapper_attrs.insert(0, parse_quote!(#[cfg(not(doc))]));

    let mut wrapper_sig = input.sig.clone();
    wrapper_sig.output = result_to_outcome(wrapper_sig.output);

    let wrapper_fn = TraitItemFn {
        attrs: wrapper_attrs,
        sig: wrapper_sig,
        default: None,
        semi_token: input.semi_token,
    };

    let doc = format!(
        "
    <span></span>

    <style>
        body.fn .item-decl code::before, #tymethod\\.{} .code-header::before {{
            content: '#[iex] ';
        }}
    </style>",
        input.sig.ident,
    );

    let mut doc_attrs = input.attrs;
    doc_attrs.insert(0, parse_quote!(#[cfg(doc)]));
    doc_attrs.push(parse_quote!(#[doc = #doc]));
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
}

pub fn transform_item_fn(input: ItemFn) -> TokenStream {
    let input_span = input.span();

    if let Some(constness) = input.sig.constness {
        return quote_spanned! { constness.span()=>
            compile_error!("#[iex] does not support const functions");
        };
    }
    if let Some(asyncness) = input.sig.asyncness {
        return quote_spanned! { asyncness.span()=>
            compile_error!("#[iex] does not support async functions");
        };
    }

    let mut wrapper_sig = input.sig.clone();
    wrapper_sig.output = result_to_outcome(wrapper_sig.output);

    let closure_block = rewrite_block(*input.block, Some(ErrorType::ForReturn));

    let mut closure: ExprClosure = parse_quote_spanned! { Span::mixed_site()=>
        move || #closure_block
    };
    closure.attrs = input
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("doc"))
        .cloned()
        .collect();

    // Doc comments must stay in the wrapper even without #[cfg(doc)] because rustc applies the
    // missing_docs lint without cfg(doc).
    let mut wrapper_attrs: Vec<_> = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .cloned()
        .collect();
    wrapper_attrs.extend([
        parse_quote!(#[cfg(not(doc))]),
        parse_quote!(#[inline(always)]),
    ]);

    let body = closure_to_iex_result(input_span, closure);
    let wrapper_fn = ItemFn {
        attrs: wrapper_attrs,
        vis: input.vis.clone(),
        sig: wrapper_sig,
        block: parse_quote!(#body),
    };

    let doc = format!(
        "
    <span></span>

    <style>
        body.fn .item-decl code::before {{
            display: block;
            content: '#[iex]';
        }}
        #method\\.{} .code-header::before {{
            content: '#[iex] ';
        }}
    </style>",
        input.sig.ident,
    );
    let mut doc_attrs = input.attrs;
    doc_attrs.insert(0, parse_quote!(#[cfg(doc)]));
    doc_attrs.push(parse_quote!(#[doc = #doc]));
    let doc_fn = ItemFn {
        attrs: doc_attrs,
        vis: input.vis,
        sig: input.sig,
        block: parse_quote!({}),
    };

    quote! {
        #wrapper_fn
        #doc_fn
    }
}

pub fn transform_closure(input: ExprClosure) -> TokenStream {
    if let Some(constness) = input.constness {
        return quote_spanned! { constness.span()=>
            compile_error!("#[iex] does not support const closures");
        };
    }
    if let Some(asyncness) = input.asyncness {
        return quote_spanned! { asyncness.span()=>
            compile_error!("#[iex] does not support async closures");
        };
    }

    let input_span = input.span();

    let closure_body = rewrite_expr(*input.body, Some(ErrorType::ForReturn));

    let mut closure: ExprClosure = parse_quote_spanned! { Span::mixed_site()=>
        move || #closure_body
    };
    closure.attrs = input.attrs;

    let wrapper_closure = ExprClosure {
        attrs: vec![parse_quote!(#[inline(always)])],
        output: ReturnType::Default,
        body: Box::new(closure_to_iex_result(input_span, closure)),
        ..input
    };

    quote! { #wrapper_closure }
}

fn result_to_outcome(result: ReturnType) -> ReturnType {
    // We have to output this mess:
    //     impl Outcome<
    //         Output = <Result<T, E> as Outcome>::Output,
    //         Error = <Result<T, E> as Outcome>::Error
    //     >
    // ...instead of something like:
    //     impl Outcome<Like = Result<T, E>>
    // because the only way to extract `T` and `E` from such an associated type `Like` is via
    // a trait, and the trait solver cannot see through such shenanigans and emits nonsense like
    // "`<Result<T, E> as ResultTrait>::Output` is not equal to `T`". Yes, this blows up code size.

    // It's also important not to add `#result_type: ::iex::Outcome` to the `where` condition. That
    // would confuse the type checker, leading it to think `#result_type::Output` is an unexpandable
    // associated type even if `#result_type` is as simple as `Result<T, E>`. A simpler example that
    // exhibits this behavior is:
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

    match result {
        ReturnType::Default => ReturnType::Default,
        ReturnType::Type(_, result_type) => {
            parse_quote! {
                -> impl ::iex::Outcome<
                    Output = <#result_type as ::iex::Outcome>::Output,
                    Error = <#result_type as ::iex::Outcome>::Error,
                >
            }
        }
    }
}

fn closure_to_iex_result(input_span: Span, closure: ExprClosure) -> Expr {
    let return_phantom: Ident = parse_quote_spanned!(Span::mixed_site()=> return_phantom);
    let try_phantom: Ident = parse_quote_spanned!(Span::mixed_site()=> try_phantom);

    // This span is required for dead code diagnostic.
    parse_quote_spanned! { input_span=> {
        // Effectively a type variable equivalent to the error type. Used for type inference in `?`
        // codegen.
        let #return_phantom = ::core::marker::PhantomData;
        let #try_phantom = #return_phantom;

        ::iex::IexResult {
            closure: #closure,
            phantom: #return_phantom,
        }
    }}
}
