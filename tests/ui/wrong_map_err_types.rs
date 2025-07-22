use iex::iex;

#[iex]
fn wrong_ok_type() -> Result<i32, i32> {
    Ok("meow").map_err(|e| e)
}

#[iex]
fn wrong_new_err_type() -> Result<i32, i32> {
    Err(1).map_err(|_| "meow")
}

#[iex]
fn wrong_passed_err_type() -> Result<i32, i32> {
    Err("meow").map_err(|e| e)
}

#[iex]
fn wrong_try_type() -> Result<(), i32> {
    Err(1).map_err(|_| "meow")?;
    Err("meow").map_err(|e| e)?;
    Ok(())
}

fn main() {}
