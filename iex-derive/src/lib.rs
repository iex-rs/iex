mod rewrite;
mod transform;

use syn::{TraitItemFn, parse, parse_macro_input};

#[proc_macro_attribute]
pub fn iex(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if let Ok(input) = parse(input.clone()) {
        transform::transform_item_fn(input)
    } else if let Ok(input) = parse(input.clone()) {
        transform::transform_closure(input)
    } else {
        transform::transform_trait_item_fn(parse_macro_input!(input as TraitItemFn))
    }
    .into()
}
