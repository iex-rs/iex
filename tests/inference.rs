use iex::iex;

#[iex]
fn infer_ok() -> Result<i32, i64> {
    Ok(Default::default())
}

#[iex]
fn infer_err() -> Result<i32, i64> {
    Err(Default::default())
}
