use anyhow::{bail, Result};
use iex::{iex, AnyhowContext, Outcome};

#[iex]
fn returns_anyhow_error() -> Result<()> {
    bail!(r"¯\_(ツ)_/¯");
}

#[iex]
fn adds_context_to_anyhow_error() -> Result<()> {
    returns_anyhow_error().context("In adds_context_to_anyhow_error()")
}

fn main() {
    adds_context_to_anyhow_error().into_result().unwrap();
}
