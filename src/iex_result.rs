use crate::{
    returning::ReturnPhantom,
    traits::{Outcome, RethrowHandle},
    trying::TryPhantom,
};
use core::marker::PhantomData;

// FIXME: This should be covariant over `Func::Output`, but isn't
pub struct IexResult<Func, E> {
    closure: Func,
    _phantom: PhantomData<E>,
}

impl<Func: FnOnce() -> T, T, E> IexResult<Func, E> {
    pub fn new(closure: Func, _phantom: ReturnPhantom<T, E>) -> Self {
        Self {
            closure,
            _phantom: PhantomData,
        }
    }

    /// Cast `#[iex] Result` to [`Result`].
    pub fn into_result(self) -> Result<T, E> {
        lithium::catch(|| unsafe { self.unwrap_or_throw(TryPhantom::new()) })
    }
}

impl<Func: FnOnce() -> T, T, E> Outcome for IexResult<Func, E> {
    type Output = T;
    type Error = E;
    type RethrowHandle = IexResultRethrowHandle<E>;

    unsafe fn unwrap_or_throw(self, _phantom: TryPhantom<E>) -> T {
        (self.closure)()
    }

    unsafe fn intercept(self) -> Result<T, (E, IexResultRethrowHandle<E>)> {
        match lithium::intercept(|| unsafe { self.unwrap_or_throw(TryPhantom::new()) }) {
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
