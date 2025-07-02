use crate::{outcome::RethrowHandle, Outcome};
use core::marker::PhantomData;

pub struct IexResult<Func, E> {
    pub closure: Func,
    pub phantom: PhantomData<fn() -> E>,
}

impl<Func: FnOnce() -> T, T, E> IexResult<Func, E> {
    /// Cast `#[iex] Result` to [`Result`].
    pub fn into_result(self) -> Result<T, E> {
        lithium::catch(|| unsafe { self.unwrap_or_throw(PhantomData) })
    }
}

#[diagnostic::do_not_recommend]
impl<Func: FnOnce() -> T, T, E> Outcome for IexResult<Func, E> {
    type Output = T;
    type Error = E;
    type RethrowHandle = IexResultRethrowHandle<E>;

    unsafe fn unwrap_or_throw(self, _phantom: PhantomData<fn() -> E>) -> T {
        (self.closure)()
    }

    unsafe fn intercept(self) -> Result<T, (E, IexResultRethrowHandle<E>)> {
        match lithium::intercept(|| unsafe { self.unwrap_or_throw(PhantomData) }) {
            Ok(value) => Ok(value),
            Err((err, handle)) => Err((err, IexResultRethrowHandle(handle))),
        }
    }
}

pub struct IexResultRethrowHandle<E>(lithium::InFlightException<E>);

impl<E> RethrowHandle for IexResultRethrowHandle<E> {
    unsafe fn do_rethrow<F>(self, ex: F) -> ! {
        unsafe { self.0.rethrow(ex) }
    }
}
