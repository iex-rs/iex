use crate::{
    traits::{Outcome, Propagate, RethrowHandle},
    trying::TryPhantom,
};

impl<T, E> Outcome for Result<T, E> {
    type Output = T;
    type Error = E;
}

impl<T, E> Propagate<T, E> for Result<T, E> {
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
    unsafe fn do_rethrow<F>(self, ex: F) -> ! {
        unsafe { lithium::throw(ex) }
    }
}
