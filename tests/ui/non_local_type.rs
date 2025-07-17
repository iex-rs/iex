#![deny(private_interfaces)]

use iex::iex;

struct S;

#[iex]
pub fn f() -> Result<S, S> {
    Ok(S)
}

fn main() {}
