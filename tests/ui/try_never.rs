use iex::iex;

#[iex]
fn try_divergent_expr() -> Result<i32, u32> {
    panic!()?;
}

#[iex]
fn try_divergent_block() -> Result<i32, u32> {
    {
        panic!();
    }?;
}

fn main() {}
