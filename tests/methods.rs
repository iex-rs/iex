use iex::{iex, Outcome};

#[derive(Debug, PartialEq)]
struct A;

impl A {
    #[iex]
    fn static_method() -> Result<(), &'static str> {
        Ok(())
    }

    #[iex]
    fn value_method(self) -> Result<Self, &'static str> {
        Ok(self)
    }

    #[iex]
    fn ref_method<'a>(&'a self) -> Result<&'a Self, &'static str> {
        Ok(self)
    }

    #[iex]
    fn mut_method<'a>(&'a mut self) -> Result<&'a mut Self, &'static str> {
        Ok(self)
    }
}

#[test]
fn methods() {
    assert_eq!(A::static_method().into_result(), Ok(()));
    assert_eq!(A.value_method().into_result(), Ok(A));
    assert_eq!(A.ref_method().into_result(), Ok(&A));
    assert_eq!(A.mut_method().into_result(), Ok(&mut A));
}
