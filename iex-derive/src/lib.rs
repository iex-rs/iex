use darling::{ast::NestedMeta, FromMeta};
use quote::quote;
use syn::{
    parse, parse_macro_input, parse_quote, parse_quote_spanned, parse_str,
    spanned::Spanned,
    visit_mut::{visit_expr_mut, VisitMut},
    Expr, ExprClosure, ExprTry, ItemFn, Lifetime, ReturnType, Signature, TraitItemFn, Type,
};

#[derive(FromMeta)]
struct MacroArgs {
    #[darling(multiple)]
    captures: Vec<String>,
}

struct ReplaceTry;
impl VisitMut for ReplaceTry {
    fn visit_expr_mut(&mut self, node: &mut Expr) {
        if let Expr::Try(ExprTry { expr, .. }) = node {
            *node = parse_quote!(::iex::Outcome::get_value_or_panic(#expr, _unsafe_iex_marker));
        }
        visit_expr_mut(self, node);
    }
    fn visit_item_fn_mut(&mut self, _node: &mut ItemFn) {
        // Don't recurse into other functions or closures
    }
    fn visit_expr_closure_mut(&mut self, _node: &mut ExprClosure) {
        // Don't recurse into other functions or closures
    }
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

    let wrapper_fn = TraitItemFn {
        attrs: vec![parse_quote! { #[cfg(not(doc))] }],
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

    let constness = input.sig.constness;
    let asyncness = input.sig.asyncness;

    let mut closure_block = input.block;
    ReplaceTry.visit_block_mut(&mut closure_block);

    let mut closure: ExprClosure = parse_quote! {
        #constness #asyncness move |_unsafe_iex_marker| -> #result_type {
            let _iex_no_copy = _iex_no_copy; // Force FnOnce inference
            #closure_block
        }
    };

    closure.attrs = input
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("doc") && !attr.path().is_ident("inline"))
        .cloned()
        .collect();
    let inline_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("inline"));
    closure.attrs.insert(0, parse_quote! { #[inline(always)] });

    let name = input.sig.ident.clone();

    let wrapper_fn = ItemFn {
        attrs: vec![
            parse_quote! { #[cfg(not(doc))] },
            parse_quote! {
                #[::iex::imp::fix_hidden_lifetime_bug::fix_hidden_lifetime_bug(
                    crate = ::iex::imp::fix_hidden_lifetime_bug
                )]
            },
            parse_quote! { #[inline(always)] },
        ],
        vis: input.vis.clone(),
        sig: wrapper_sig,
        block: parse_quote_spanned! {
            // This span is required for dead code diagnostic
            input_span =>
            {
                let _iex_no_copy = ::iex::imp::NoCopy; // Force FnOnce inference
                // We need { .. } to support the #[inline] attribute on the closure
                #[allow(unused_mut)]
                let mut #name = { #closure };
                ::iex::imp::IexResult::new(
                    #inline_attr move |_unsafe_iex_marker| {
                        ::iex::Outcome::get_value_or_panic(
                            #name(_unsafe_iex_marker),
                            _unsafe_iex_marker,
                        )
                    },
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
    } else {
        transform_trait_item_fn(captures, parse_macro_input!(input as TraitItemFn))
    }
}
