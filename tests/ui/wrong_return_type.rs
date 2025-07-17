use iex::iex;

#[iex]
fn non_result_returns(cond: bool) -> Result<i32, i32> {
    if cond {
        return 42;
    }

    "meow"
}

#[iex]
fn wrong_ok_type() -> Result<i32, i32> {
    Ok("meow")
}

#[iex]
fn wrong_incompatible_ok_types(cond: bool) -> Result<i32, i32> {
    if cond { Ok("meow") } else { Ok(false) }
}

#[iex]
fn wrong_err_type() -> Result<i32, i32> {
    Err("meow")
}

#[iex]
fn wrong_incompatible_err_types(cond: bool) -> Result<i32, i32> {
    if cond { Err("meow") } else { Err(false) }
}

#[iex]
fn f() -> Result<(), &'static str> {
    Err("meow")
}

#[iex]
fn wrong_err_type_iex() -> Result<i32, i32> {
    f()
}

fn main() {}
