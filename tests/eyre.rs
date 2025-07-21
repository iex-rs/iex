use eyre::{Result, WrapErr, bail, eyre};
use iex::iex;

#[iex]
fn returns_eyre_error() -> Result<()> {
    Err(eyre!(r"¯\_(ツ)_/¯"))
    // bail!(r"¯\_(ツ)_/¯");
}

#[iex]
fn wraps_eyre_error() -> Result<()> {
    returns_eyre_error().wrap_err("In wraps_eyre_error()")
}

#[test]
fn iex_matches_result() {
    let expected: Result<()> = Err(eyre!(r"¯\_(ツ)_/¯")).context("In wraps_eyre_error()");

    assert_eq!(
        format!("{:#}", wraps_eyre_error().into_result().unwrap_err()),
        format!("{:#}", expected.unwrap_err()),
    );
}
