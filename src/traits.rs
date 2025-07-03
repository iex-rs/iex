use crate::phantoms::TryPhantom;

pub trait Outcome: Sized {
    type Output;

    type Error;

    type RethrowHandle: RethrowHandle;

    unsafe fn unwrap_or_throw(self, phantom: TryPhantom<Self::Error>) -> Self::Output;

    unsafe fn intercept(self) -> Result<Self::Output, (Self::Error, Self::RethrowHandle)>;
}

pub trait RethrowHandle: Sized {
    unsafe fn rethrow<F>(self, ex: F, _phantom: TryPhantom<F>) -> ! {
        unsafe { self.do_rethrow(ex) }
    }

    unsafe fn do_rethrow<F>(self, ex: F) -> !;
}

#[diagnostic::on_unimplemented(
    message = "the `?` operator can only be applied to `Result`",
    label = "the `?` operator cannot be applied to type `{Self}`"
)]
pub trait Try: Outcome {
    unsafe fn do_try<F: ThrowFromOutcome<Self::Output, Self::Error>>(
        self,
        phantom: TryPhantom<F>,
    ) -> Self::Output {
        // This comparison will be optimized out.
        if typeid::of::<Self::Error>() == typeid::of::<F>() {
            // SAFETY: If we enter this conditional, `E` and `F` differ only in lifetimes. Lifetimes
            // are erased in runtime, so `impl From<E> for F` has the same implementation as
            // `impl From<T> for T` for some `T`, and that blanket implementation is a no-op.
            // Therefore, no conversion needs to happen.
            unsafe { self.unwrap_or_throw(TryPhantom::new()) }
        } else {
            match unsafe { self.intercept() } {
                Ok(value) => value,
                Err((err, handle)) => unsafe { handle.rethrow(F::from(err), phantom) },
            }
        }
    }
}

#[diagnostic::on_unimplemented(
    message = "`?` couldn't convert the error to `{Self}`",
    note = "this can't be annotated with `?` because it has type `Result<{T}, {E}>`",
    note = "the question mark operation (`?`) implicitly performs a conversion on the error value \
            using the `From` trait"
)]
pub trait ThrowFromOutcome<T, E>: From<E> {}

impl<T, E, F: From<E>> ThrowFromOutcome<T, E> for F {}

// When returning a mismatching type from a function, we want to show a readable error like
//     expected `Result<{T1}, {E1}>`, found `Result<{T2}, {E2}>`
// If the actual type is `IexResult`, we want the error to spell `Result` anyway. So that means we
// have to read `<R as Outcome>::Output` and `<R as Outcome>::Error`. But if the returned type isn't
// even an `Outcome`, we *still* want to show a readable error like
//     expected `Result<{T1}, {E1}>`, found `R`
// So there's two successive checks. The `Like` trait ensures that if the value isn't an `Outcome`,
// we emit an error of the latter kind. If it is an `Outcome`, or, equivalently, `Like` is
// implemented, we perform the second check with `Return`.
//
// There's some trickiness: if `Like` simply had `Outcome` as a supertrait, rustc would just emit
// an error about `Outcome` not being implemented, and ignore `Like`. So instead, `Like` doesn't
// inherit from anything, but all of its *implementations* are guaranteed to be `Outcome`s.
// Similarly, `Return` does not assert that `T1 = T2, E1 = E2`, but all of its implementations only
// apply when that holds.

// `R` is only used for the diagnostic.
#[diagnostic::on_unimplemented(
    message = "mismatched types",
    label = "expected `{R}`, found `{Self}`"
)]
pub trait Like<R>: Sized {
    type This: From<Self>;
}

#[diagnostic::on_unimplemented(
    message = "mismatched types",
    label = "expected `Result<{T1}, {E1}>`, found `Result<{T2}, {E2}>`"
)]
pub trait Return<T1, E1, T2, E2>: Sized {
    // This acts as a type-level proof of `Self: Outcome<Output = T1, Error = E1>`.
    type This: From<Self> + Outcome<Output = T1, Error = E1>;
}
