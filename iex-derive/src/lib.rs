mod rewrite;
mod transform;

use syn::{ExprClosure, TraitItemFn, parse, parse_macro_input};

#[proc_macro_attribute]
pub fn iex(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // We could support closures here as well, but applying attributes to expressions requires
    // unstable features `stmt_expr_attributes` and `proc_macro_hygiene`, so it's too early to care.
    if let Ok(input) = parse(input.clone()) {
        transform::transform_item_fn(input)
    } else {
        transform::transform_trait_item_fn(parse_macro_input!(input as TraitItemFn))
    }
    .into()
}

#[proc_macro]
pub fn closure(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    transform::transform_closure(parse_macro_input!(input as ExprClosure)).into()
}
