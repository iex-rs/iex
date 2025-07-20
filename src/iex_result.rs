use crate::{
    covariant_fnonce::{Callable, CovariantFnOnce},
    returning::ReturnPhantom,
    traits::{Outcome, RethrowHandle},
    trying::TryPhantom,
};
use core::marker::PhantomData;

pub struct IexResult<Func, T, E> {
    closure: CovariantFnOnce<Func, T>,
    _phantom: PhantomData<E>,
}

impl<Func: FnOnce() -> T, T, E> IexResult<Func, T, E> {
    pub fn new(closure: Func, _phantom: ReturnPhantom<T, E>) -> Self {
        Self {
            closure: CovariantFnOnce::new(closure),
            _phantom: PhantomData,
        }
    }
}

impl<Func: Callable, T, E> IexResult<Func, T, E> {
    /// Cast `#[iex] Result` to [`Result`].
    pub fn into_result(self) -> Result<T, E> {
        lithium::catch(|| unsafe { self.unwrap_or_throw(TryPhantom::new()) })
    }
}

impl<Func: Callable, T, E> Outcome for IexResult<Func, T, E> {
    type Output = T;
    type Error = E;
    type RethrowHandle = IexResultRethrowHandle<E>;

    unsafe fn unwrap_or_throw(self, _phantom: TryPhantom<E>) -> T {
        self.closure.call()
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
    unsafe fn rethrow<F>(self, ex: F, _phantom: TryPhantom<F>) -> ! {
        unsafe { self.0.rethrow(ex) }
    }
}

// Used to reduce the number of repetitions of `R` in codegen.
pub type IexResultCtor<F, R> = IexResult<F, <R as Outcome>::Output, <R as Outcome>::Error>;
