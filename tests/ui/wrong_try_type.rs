use iex::iex;

#[iex]
fn wrong_try_type() -> Result<i32, i32> {
    10?;
    None?;
    Err("")?;
    Ok(0)
}

fn main() {}
