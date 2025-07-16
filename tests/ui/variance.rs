use iex::iex;

#[iex]
fn contravariance_in_ok<'a>(s: &'a str) -> Result<&'static str, ()> {
    Ok(s)
}

#[iex]
fn contravariance_in_return_err<'a>(s: &'a str) -> Result<(), &'static str> {
    Err(s)
}

#[iex]
fn contravariance_in_try_err<'a>(s: &'a str) -> Result<(), &'static str> {
    Err(s)?;
    Ok(())
}

fn main() {}
