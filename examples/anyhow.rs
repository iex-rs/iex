use anyhow::{bail, Result};
use iex::{iex, Outcome};

#[iex]
fn returns_anyhow_error() -> Result<()> {
    bail!(r"¯\_(ツ)_/¯");
}

fn main() {
    returns_anyhow_error().into_result().unwrap();
}
