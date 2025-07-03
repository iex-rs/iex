use crate::{phantoms::TryPhantom, Like, Outcome, RethrowHandle, Return, Try};

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

#[diagnostic::do_not_recommend]
impl<T, E> Try for Result<T, E> {}

#[diagnostic::do_not_recommend]
impl<T1, E1, T2, E2> Like<Result<T1, E1>> for Result<T2, E2> {
    type This = Self;
}

#[diagnostic::do_not_recommend]
impl<T, E> Return<T, E, T, E> for Result<T, E> {
    type This = Self;
}

pub struct ResultRethrowHandle;

impl RethrowHandle for ResultRethrowHandle {
    unsafe fn do_rethrow<F>(self, ex: F) -> ! {
        unsafe { lithium::throw(ex) }
    }
}
