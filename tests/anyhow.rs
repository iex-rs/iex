use anyhow::{anyhow, bail, Result};
use iex::{iex, Context, Outcome};

#[iex]
fn returns_anyhow_error() -> Result<()> {
    bail!(r"¯\_(ツ)_/¯");
}

#[iex]
fn adds_context_to_anyhow_error() -> Result<()> {
    returns_anyhow_error().context("In adds_context_to_anyhow_error()")
}

#[test]
fn iex_matches_result() {
    let expected: Result<()> =
        Err(anyhow!(r"¯\_(ツ)_/¯")).context("In adds_context_to_anyhow_error()");

    assert_eq!(
        format!("{:?}", adds_context_to_anyhow_error().into_result()),
        format!("{:?}", expected),
    );
}

#[test]
fn option_works() {
    let _: Result<()> = None.context("Meow");
}
