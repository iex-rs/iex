use iex::iex;

trait FallibleSum {
    type Error;

    #[iex]
    fn add(self, other: Self) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl FallibleSum for i32 {
    type Error = &'static str;

    #[iex]
    fn add(self, other: Self) -> Result<Self, Self::Error> {
        self.checked_add(other).ok_or("Stack Overflow")
    }
}

trait SayHello {
    #[iex]
    fn say_hello(self) -> Result<String, ()>
    where
        Self: Sized,
    {
        Ok(format!("Default implementation says Hello!"))
    }
}

impl SayHello for String {
    #[iex]
    fn say_hello(self) -> Result<String, ()> {
        Ok(self)
    }
}

impl SayHello for &str {
    #[allow(refining_impl_trait)]
    fn say_hello(self) -> Result<String, ()> {
        Ok(self.to_string())
    }
}

fn main() {}
