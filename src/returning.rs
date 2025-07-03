use crate::{never::Never, traits::Outcome, trying::TryPhantom, IexResult};
use core::marker::PhantomData;

// When returning a mismatching type from a function, we want to show a readable error like
//     expected `Result<{T1}, {E1}>`, found `Result<{T2}, {E2}>`
// If the actual type is `IexResult`, we want the error to spell `Result` anyway. So that means we
// have to read `<R as Outcome>::Output` and `<R as Outcome>::Error`. But if the returned type isn't
// even an `Outcome`, we *still* want to show a readable error like
//     expected `Result<{T1}, {E1}>`, found `R`
// So there's two successive checks: first, `AnyReturn` checks that the value is an `Outcome`, and
// then `Return` validates the `Output` and `Error` types for compatibility wrt. `Propagate`.

// `Expected` is only used for diagnostics.
#[diagnostic::on_unimplemented(
    message = "mismatched types",
    label = "expected `{Expected}`, found `{Self}`"
)]
pub trait AnyReturn<Expected>: Outcome {
    // `Result<Self::Output, Self::Error>`.
    type AsResult;
}

#[diagnostic::do_not_recommend]
impl<Expected, T, E> AnyReturn<Expected> for Result<T, E> {
    type AsResult = Result<T, E>;
}

#[diagnostic::do_not_recommend]
impl<Expected, Func: FnOnce() -> T, T, E> AnyReturn<Expected> for IexResult<Func, E> {
    type AsResult = Result<T, E>;
}

#[diagnostic::do_not_recommend]
impl<Expected> AnyReturn<Expected> for Never {
    type AsResult = Result<Never, Never>;
}

#[diagnostic::on_unimplemented(
    message = "mismatched types",
    label = "expected `Result<{T}, {E}>`, found `{Self}`"
)]
pub trait Return<T, E, R>: Sized {
    fn map_outcome(outcome: R) -> impl Outcome<Output = T, Error = E>;
}

#[diagnostic::do_not_recommend]
impl<T, E> Return<T, E, Result<T, E>> for Result<T, E> {
    fn map_outcome(outcome: Result<T, E>) -> impl Outcome<Output = T, Error = E> {
        outcome
    }
}

#[diagnostic::do_not_recommend]
impl<Func: FnOnce() -> T, T, E> Return<T, E, IexResult<Func, E>> for Result<T, E> {
    fn map_outcome(outcome: IexResult<Func, E>) -> impl Outcome<Output = T, Error = E> {
        outcome
    }
}

#[diagnostic::do_not_recommend]
impl<T, E> Return<T, E, Never> for Result<Never, Never> {
    #[allow(unreachable_code)]
    fn map_outcome(outcome: Never) -> impl Outcome<Output = T, Error = E> {
        outcome as Result<T, E>
    }
}

pub struct ReturnPhantom<T, E>(PhantomData<(T, E)>);

pub fn make_return_phantom<T, E>() -> ReturnPhantom<T, E> {
    ReturnPhantom(PhantomData)
}

impl<T, E> ReturnPhantom<T, E> {
    pub fn to_try_phantom(self) -> TryPhantom<E> {
        TryPhantom::new()
    }

    pub unsafe fn do_return<R: AnyReturn<Result<T, E>, AsResult: Return<T, E, R>>>(
        self,
        outcome: R,
    ) -> T {
        unsafe { R::AsResult::map_outcome(outcome).unwrap_or_throw(self.to_try_phantom()) }
    }
}

impl<T, E> Clone for ReturnPhantom<T, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, E> Copy for ReturnPhantom<T, E> {}
