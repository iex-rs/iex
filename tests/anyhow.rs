use anyhow::{Context, Result, anyhow, bail};
use iex::iex;
use std::error::Error;

#[iex]
fn returns_anyhow_error() -> Result<()> {
    Err(anyhow!(r"¯\_(ツ)_/¯"))
    // bail!(r"¯\_(ツ)_/¯");
}

#[iex]
fn adds_context_to_anyhow_error() -> Result<()> {
    returns_anyhow_error().context("In adds_context_to_anyhow_error()")
}

// #[iex]
// fn returns_normal_error() -> Result<u32, impl Error + Send + Sync> {
//     (-1).try_into()
// }

// #[iex]
// fn adds_context_to_normal_error() -> Result<u32> {
//     returns_normal_error().context("In adds_context_to_normal_error()")
// }

#[test]
fn iex_matches_result() {
    let expected: Result<()> =
        Err(anyhow!(r"¯\_(ツ)_/¯")).context("In adds_context_to_anyhow_error()");
    assert_eq!(
        format!("{:?}", adds_context_to_anyhow_error().into_result()),
        format!("{:?}", expected),
    );

    // let expected: Result<u32> = u32::try_from(-1).context("In adds_context_to_normal_error()");
    // assert_eq!(
    //     format!("{:?}", adds_context_to_normal_error().into_result()),
    //     format!("{:?}", expected),
    // );
}
