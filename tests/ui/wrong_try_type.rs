use iex::iex;

#[iex]
fn f() -> Result<(), &'static str> {
    Err("")
}

#[iex]
fn wrong_try_type() -> Result<i32, i32> {
    10?;
    None?;
    Err("")?;
    f()?;
    Ok(0)
}

fn main() {}
