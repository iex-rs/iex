use iex::iex;

#[iex]
fn f() -> Result<i32, u32> {
    panic!()?;
}

fn main() {}
