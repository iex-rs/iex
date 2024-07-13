use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    parse_macro_input, parse_quote, parse_quote_spanned, parse_str,
    spanned::Spanned,
    visit_mut::{visit_expr_mut, visit_expr_path_mut, VisitMut},
    Expr, ExprClosure, ExprPath, ExprTry, FnArg, Generics, ItemFn, Pat, PatType, ReturnType,
    Signature, Token, Type,
};

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

struct ReplaceSelf;
impl VisitMut for ReplaceSelf {
    fn visit_expr_path_mut(&mut self, node: &mut ExprPath) {
        if node.path.is_ident("self") {
            node.path = parse_quote!(_iex_self);
        }
        visit_expr_path_mut(self, node);
    }
    fn visit_item_fn_mut(&mut self, _node: &mut ItemFn) {
        // Don't recurse into other functions
    }
}

#[proc_macro_attribute]
pub fn iex(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let input_span = input.span();

    let output = input.sig.output.clone();
    let result_type = match output {
        ReturnType::Default => parse_quote! { () },
        ReturnType::Type(_, result_type) => result_type,
    };
    let output_type: Type = parse_quote! { <#result_type as ::iex::Outcome>::Output };
    let error_type: Type = parse_quote! { <#result_type as ::iex::Outcome>::Error };
    let to_impl_outcome: ReturnType = parse_quote! {
        -> impl ::iex::Outcome<
            Output = #output_type,
            Error = #error_type,
        >
    };

    let mut where_clause = input
        .sig
        .generics
        .where_clause
        .clone()
        .unwrap_or(parse_quote! { where });
    where_clause
        .predicates
        .push(parse_quote_spanned! { result_type.span() => #result_type: ::iex::Outcome });
    let wrapper_sig = Signature {
        generics: Generics {
            where_clause: Some(where_clause),
            ..input.sig.generics.clone()
        },
        inputs: input
            .sig
            .inputs
            .iter()
            .enumerate()
            .map(|(i, arg)| match arg {
                receiver @ FnArg::Receiver(_) => receiver.clone(),
                FnArg::Typed(pat_type) => FnArg::Typed(PatType {
                    pat: Box::new(Pat::Path(parse_str(&format!("_iex_arg_{i}")).unwrap())),
                    ..pat_type.clone()
                }),
            })
            .collect(),
        output: to_impl_outcome.clone(),
        ..input.sig.clone()
    };
    let call_site_args: Vec<_> = input
        .sig
        .inputs
        .iter()
        .enumerate()
        .map(|(i, arg)| -> Expr {
            match arg {
                FnArg::Receiver(_) => parse_str("self"),
                FnArg::Typed(_) => parse_str(&format!("_iex_arg_{i}")),
            }
            .unwrap()
        })
        .collect();

    let constness = input.sig.constness;
    let asyncness = input.sig.asyncness;

    let mut closure_inputs = input.sig.inputs.clone();
    if let Some(receiver) = input.sig.receiver() {
        closure_inputs[0] = FnArg::Typed(PatType {
            attrs: receiver.attrs.clone(),
            pat: Box::new(Pat::Path(parse_str("_iex_self").unwrap())),
            colon_token: Token![:](Span::call_site()),
            ty: receiver.ty.clone(),
        });
    }
    closure_inputs.insert(
        0,
        parse_quote! {
            #[allow(unused_variables)] _unsafe_iex_marker: ::iex::imp::Marker<#error_type>
        },
    );

    let mut closure_block = input.block;
    ReplaceTry.visit_block_mut(&mut closure_block);
    ReplaceSelf.visit_block_mut(&mut closure_block);

    let mut closure: ExprClosure = parse_quote! {
        #constness #asyncness move |#closure_inputs| -> #result_type #closure_block
    };

    closure.attrs = input.attrs;
    let inline_attr = closure
        .attrs
        .iter()
        .position(|attr| attr.path().is_ident("inline"))
        .map(|index| closure.attrs.remove(index));
    closure.attrs.insert(0, parse_quote! { #[inline(always)] });

    let name = input.sig.ident.clone();

    let wrapper_fn = ItemFn {
        attrs: vec![
            parse_quote! { #[::iex::imp::fix_hidden_lifetime_bug] },
            parse_quote! { #[inline(always)] },
        ],
        vis: input.vis,
        sig: wrapper_sig,
        block: parse_quote_spanned! {
            // This span is required for dead code diagnostic
            input_span =>
            {
                // We need { .. } to support the #[inline] attribute on the closure
                let #name = { #closure };
                ::iex::imp::IexResult::new(
                    #inline_attr move |_unsafe_iex_marker| {
                        ::iex::Outcome::get_value_or_panic(
                            #name(_unsafe_iex_marker, #(#call_site_args,)*),
                            _unsafe_iex_marker,
                        )
                    },
                )
            }
        },
    };

    wrapper_fn.into_token_stream().into()
}
