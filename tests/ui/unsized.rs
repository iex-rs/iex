use iex::iex;

#[iex]
fn return_unsized_untyped() -> Result<(), ()> {
    match 1 {
        1 => *"meow",
        2 => Ok(*"meow"),
        _ => Err("*meow"),
    }
}

#[iex]
fn return_unsized_typed() -> Result<str, str> {
    Err(*"meow")?;
    if true { Ok(*"meow") } else { Err(*"meow") }
}

#[iex]
fn forward_unsized_typed() -> Result<str, str> {
    return_unsized_typed()
}

#[iex]
fn try_unsized_err() -> Result<(), ()> {
    Err(*"meow")?;
    Ok(())
}

fn main() {}

// FIXME: good diagnostics for this are blocked on https://github.com/rust-lang/rust/issues/144074
