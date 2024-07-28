use crate::{
    imp::{ExceptionMapper, Marker},
    outcome::Sealed,
    IexPanic, Outcome, EXCEPTION,
};
use std::marker::PhantomData;
use std::panic::AssertUnwindSafe;

pub(crate) trait CallWithMarker<T, E> {
    fn call_with_marker(self, marker: Marker<E>) -> T;
}

impl<T, E, Func: FnOnce(Marker<E>) -> T> CallWithMarker<T, E> for Func {
    #[inline(always)]
    fn call_with_marker(self, marker: Marker<E>) -> T {
        self(marker)
    }
}

pub struct IexResult<T, E, Func>(pub Func, pub PhantomData<fn() -> (T, E)>);

impl<T, E, Func> Sealed for IexResult<T, E, Func> {}

impl<T, E, Func: CallWithMarker<T, E>> Outcome for IexResult<T, E, Func> {
    type Output = T;
    type Error = E;

    fn get_value_or_panic(self, marker: Marker<E>) -> T {
        self.0.call_with_marker(marker)
    }

    #[cfg(doc)]
    #[crate::iex]
    fn inspect_err<F>(self, f: F) -> Result<T, E>
    where
        F: FnOnce(&Self::Error),
    {
    }

    #[cfg(not(doc))]
    fn inspect_err<F>(self, f: F) -> impl Outcome<Output = T, Error = E>
    where
        F: FnOnce(&Self::Error),
    {
        // NB: It is impossible to implement inspect_err without writeback that map_err
        // performs. Indeed, if `f` calls an #[iex] function that returns an error, that error
        // is saved to EXCEPTION. It is necessary to override it back with err before returning
        // from `inspect_err(..).get_value_or_panic()`.
        self.map_err(|err| {
            f(&err);
            err
        })
    }

    #[cfg(doc)]
    #[crate::iex]
    fn map_err<F, O>(self, op: O) -> Result<T, F>
    where
        O: FnOnce(E) -> F,
    {
    }

    #[cfg(not(doc))]
    fn map_err<F, O>(self, op: O) -> impl Outcome<Output = Self::Output, Error = F>
    where
        O: FnOnce(E) -> F,
    {
        IexResult(
            |marker| {
                let exception_mapper = ExceptionMapper::new(marker, (), |(), err| op(err));
                let value = self.get_value_or_panic(exception_mapper.get_in_marker());
                exception_mapper.swallow();
                value
            },
            PhantomData,
        )
    }

    fn into_result(self) -> Result<T, E> {
        std::panic::catch_unwind(AssertUnwindSafe(|| {
            self.0.call_with_marker(unsafe { Marker::new() })
        }))
        .map_err(
            #[cold]
            |payload| {
                if !payload.is::<IexPanic>() {
                    std::panic::resume_unwind(payload);
                }
                EXCEPTION.with(|exception| unsafe {
                    let exception = &mut *exception.get();
                    let error = exception.read_unchecked();
                    exception.clear();
                    error
                })
            },
        )
    }
}
