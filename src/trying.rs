use crate::traits::{Outcome, RethrowHandle};
use core::marker::PhantomData;

#[diagnostic::on_unimplemented(
    message = "the `?` operator can only be applied to `Result`",
    label = "the `?` operator cannot be applied to type `{Self}`"
)]
pub trait Try: Sized {
    // Type-level proof that `Self: Outcome`. We can't just make `Outcome` a supertrait of `Try`
    // because that causes rustc to emit an unsatisfied obligation `Self: !Outcome` rather than
    // `Self: !Try`, resulting in worse diagnostics.
    type This: From<Self> + Outcome;
}

#[diagnostic::do_not_recommend]
impl<R: Outcome> Try for R {
    type This = Self;
}

#[diagnostic::on_unimplemented(
    message = "`?` couldn't convert the error to `{Self}`",
    note = "this can't be annotated with `?` because it has type `Result<{T}, {E}>`",
    note = "the question mark operation (`?`) implicitly performs a conversion on the error value \
            using the `From` trait"
)]
pub trait ThrowFromOutcome<T, E>: From<E> {}

impl<T, E, F: From<E>> ThrowFromOutcome<T, E> for F {}

// See the comment on `ReturnPhantom` for why this is invariant.
pub struct TryPhantom<E>(PhantomData<*mut E>);

impl<E> TryPhantom<E> {
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }

    pub unsafe fn do_try<R: Try>(self, outcome: R) -> <R::This as Outcome>::Output
    where
        E: ThrowFromOutcome<<R::This as Outcome>::Output, <R::This as Outcome>::Error>,
    {
        let outcome = R::This::from(outcome);
        // This comparison will be optimized out.
        if typeid::of::<E>() == typeid::of::<<R::This as Outcome>::Error>() {
            // SAFETY: If we enter this conditional, `E` and `F` differ only in lifetimes. Lifetimes
            // are erased in runtime, so `impl From<E> for F` has the same implementation as
            // `impl From<T> for T` for some `T`, and that blanket implementation is a no-op.
            // Therefore, no conversion needs to happen.
            unsafe { outcome.unwrap_or_throw(TryPhantom::new()) }
        } else {
            match unsafe { outcome.intercept() } {
                Ok(value) => value,
                Err((err, handle)) => unsafe { handle.rethrow(E::from(err), self) },
            }
        }
    }
}

impl<E> Clone for TryPhantom<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E> Copy for TryPhantom<E> {}
