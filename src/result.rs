use crate::{Outcome, RethrowHandle, Try};
use core::marker::PhantomData;

impl<T, E> Outcome for Result<T, E> {
    type Output = T;
    type Error = E;
    type RethrowHandle = ResultRethrowHandle;

    unsafe fn unwrap_or_throw(self, _phantom: PhantomData<fn() -> E>) -> T {
        match self {
            Ok(value) => value,
            Err(error) => unsafe { lithium::throw(error) },
        }
    }

    unsafe fn intercept(self) -> Result<T, (E, ResultRethrowHandle)> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err((err, ResultRethrowHandle)),
        }
    }
}

#[diagnostic::do_not_recommend]
impl<T, E> Try for Result<T, E> {}

pub struct ResultRethrowHandle;

impl RethrowHandle for ResultRethrowHandle {
    unsafe fn do_rethrow<F>(self, ex: F) -> ! {
        unsafe { lithium::throw(ex) }
    }
}
