use crate::{
    Outcome,
    outcome::{RethrowHandle, Sealed},
};
use core::marker::PhantomData;

pub struct IexResult<Func, E> {
    pub closure: Func,
    pub phantom: PhantomData<fn() -> E>,
}

impl<Func, E> Sealed for IexResult<Func, E> {}

impl<Func: FnOnce() -> T, T, E> Outcome for IexResult<Func, E> {
    type Output = T;
    type Error = E;

    unsafe fn unwrap_or_throw(self, _phantom: PhantomData<fn() -> E>) -> T {
        (self.closure)()
    }

    unsafe fn intercept(self) -> Result<T, (E, RethrowHandle<E>)> {
        match lithium::intercept(|| unsafe { self.unwrap_or_throw(PhantomData) }) {
            Ok(value) => Ok(value),
            Err((err, handle)) => Err((
                err,
                RethrowHandle {
                    in_flight_exception: Some(handle),
                },
            )),
        }
    }

    fn into_result(self) -> Result<T, E> {
        lithium::catch(|| unsafe { self.unwrap_or_throw(PhantomData) })
    }
}
