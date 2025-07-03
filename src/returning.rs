use crate::{trying::TryPhantom, IexResult, Outcome};
use core::marker::PhantomData;

// When returning a mismatching type from a function, we want to show a readable error like
//     expected `Result<{T1}, {E1}>`, found `Result<{T2}, {E2}>`
// If the actual type is `IexResult`, we want the error to spell `Result` anyway. So that means we
// have to read `<R as Outcome>::Output` and `<R as Outcome>::Error`. But if the returned type isn't
// even an `Outcome`, we *still* want to show a readable error like
//     expected `Result<{T1}, {E1}>`, found `R`
// So there's two successive checks: first, `AnyReturn` checks that the value is an `Outcome`, and
// then `Return` validates the `Output` and `Error` types.

// `Expected` is only used for diagnostics.
#[diagnostic::on_unimplemented(
    message = "mismatched types",
    label = "expected `{Expected}`, found `{Self}`"
)]
pub trait AnyReturn<Expected>: Outcome {}

#[diagnostic::do_not_recommend]
impl<Expected, T, E> AnyReturn<Expected> for Result<T, E> {}

#[diagnostic::do_not_recommend]
impl<Expected, Func: FnOnce() -> T, T, E> AnyReturn<Expected> for IexResult<Func, E> {}

#[diagnostic::on_unimplemented(
    message = "mismatched types",
    label = "expected `Result<{T}, {E}>`, found `{AsResult}`"
)]
pub trait Return<T, E, AsResult>: Outcome<Output = T, Error = E> {}

#[diagnostic::do_not_recommend]
impl<T, E, R: Outcome<Output = T, Error = E>> Return<T, E, Result<T, E>> for R {}

pub struct ReturnPhantom<T, E>(PhantomData<(T, E)>);

pub fn make_return_phantom<T, E>() -> ReturnPhantom<T, E> {
    ReturnPhantom(PhantomData)
}

impl<T, E> ReturnPhantom<T, E> {
    pub fn to_try_phantom(self) -> TryPhantom<E> {
        TryPhantom::new()
    }

    pub unsafe fn do_return<R: AnyReturn<Result<T, E>>>(self, outcome: R) -> T
    where
        R: Return<T, E, Result<<R as Outcome>::Output, <R as Outcome>::Error>>,
    {
        unsafe { outcome.unwrap_or_throw(self.to_try_phantom()) }
    }
}

impl<T, E> Clone for ReturnPhantom<T, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, E> Copy for ReturnPhantom<T, E> {}
