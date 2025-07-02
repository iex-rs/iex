use core::marker::PhantomData;

pub trait Outcome: Sized {
    type Output;

    type Error;

    type RethrowHandle: RethrowHandle;

    // `phantom` is passed so that there's an easy way to unify type variables with `E`.
    unsafe fn unwrap_or_throw(self, phantom: PhantomData<fn() -> Self::Error>) -> Self::Output;

    unsafe fn intercept(self) -> Result<Self::Output, (Self::Error, Self::RethrowHandle)>;
}

pub trait RethrowHandle: Sized {
    unsafe fn rethrow<F>(self, ex: F, _phantom: PhantomData<fn() -> F>) -> ! {
        unsafe { self.do_rethrow(ex) }
    }

    unsafe fn do_rethrow<F>(self, ex: F) -> !;
}

#[diagnostic::on_unimplemented(
    message = "the `?` operator can only be applied to `Result` or `#[iex] Result`",
    label = "the `?` operator cannot be applied to type `{Self}`"
)]
pub trait Try: Outcome {
    unsafe fn do_try<F: ThrowFromOutcome<Self>>(
        self,
        phantom: PhantomData<fn() -> F>,
    ) -> Self::Output {
        // This comparison will be optimized out.
        if typeid::of::<Self::Error>() == typeid::of::<F>() {
            // SAFETY: If we enter this conditional, `E` and `F` differ only in lifetimes. Lifetimes
            // are erased in runtime, so `impl From<E> for F` has the same implementation as
            // `impl From<T> for T` for some `T`, and that blanket implementation is a no-op.
            // Therefore, no conversion needs to happen.
            unsafe { self.unwrap_or_throw(PhantomData) }
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
    note = "this can't be annotated with `?` because it has type `{R}`",
    note = "the question mark operation (`?`) implicitly performs a conversion on the error value \
            using the `From` trait"
)]
pub trait ThrowFromOutcome<R: Outcome>: From<R::Error> {}

impl<R: Outcome, U: From<R::Error>> ThrowFromOutcome<R> for U {}
