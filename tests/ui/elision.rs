use iex::iex;

struct S;

impl S {
    #[iex]
    fn correct_elision(&self, other: &str) -> Result<&str, ()> {
        Ok(other)
    }
}

fn main() {}
