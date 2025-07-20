use crate::{
    traits::{Outcome, RethrowHandle},
    trying::TryPhantom,
};

impl<T, E> Outcome for Result<T, E> {
    type Output = T;
    type Error = E;
    type RethrowHandle = ResultRethrowHandle;

    unsafe fn unwrap_or_throw(self, _phantom: TryPhantom<E>) -> T {
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

pub struct ResultRethrowHandle;

impl RethrowHandle for ResultRethrowHandle {
    unsafe fn rethrow<F>(self, ex: F, _phantom: TryPhantom<F>) -> ! {
        unsafe { lithium::throw(ex) }
    }
}
