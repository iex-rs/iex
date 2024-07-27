use iex::{iex, Outcome};

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
        self.checked_add(other).ok_or("Integer overflow")
    }
}

#[test]
fn call_via_trait() {
    assert_eq!(FallibleSum::add(1, 2).into_result(), Ok(3));
    assert_eq!(
        FallibleSum::add(1, i32::MAX).into_result(),
        Err("Integer overflow")
    );
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

#[test]
fn refined() {
    assert_eq!(
        "test".to_string().say_hello().into_result().unwrap(),
        "test",
    );
    assert_eq!("test".say_hello().into_result().unwrap(), "test");
}
