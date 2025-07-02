use iex::iex;

#[iex]
fn wrong_return_type(cond: bool) -> Result<i32, i32> {
    if cond {
        return 42;
    }

    "meow"
}

fn main() {}
