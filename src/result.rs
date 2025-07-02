use crate::{outcome::RethrowHandle, Outcome};
use core::marker::PhantomData;

#[diagnostic::do_not_recommend]
impl<T, E> Outcome for Result<T, E> {
    type Output = T;
    type Error = E;

    unsafe fn unwrap_or_throw(self, _phantom: PhantomData<fn() -> E>) -> T {
        match self {
            Ok(value) => value,
            Err(error) => unsafe { lithium::throw(error) },
        }
    }

    unsafe fn intercept(self) -> Result<T, (E, RethrowHandle<E>)> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err((
                err,
                RethrowHandle {
                    in_flight_exception: None,
                },
            )),
        }
    }
}
