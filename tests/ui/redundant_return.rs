use iex::iex;

#[iex]
fn redundant_return(cond: bool) -> Result<i32, i32> {
    if cond {
        return 42;
    } else {
        "meow"
    }
}

fn main() {}
