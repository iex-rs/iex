use core::marker::PhantomData;

#[allow(private_bounds)]
#[diagnostic::on_unimplemented(
    message = "the `?` operator can only be applied to `Result` or `#[iex] Result`",
    label = "the `?` operator cannot be applied to type `{Self}`"
)]
pub trait Outcome {
    type Output;

    type Error;

    // `phantom` is passed so that there's an easy way to unify type variables with `E`.
    unsafe fn unwrap_or_throw(self, phantom: PhantomData<fn() -> Self::Error>) -> Self::Output;

    unsafe fn unwrap_or_throw_with_conversion<F>(
        self,
        phantom: PhantomData<fn() -> F>,
    ) -> Self::Output
    where
        Self: Sized,
        Self::Error: Into<F>,
    {
        // This comparison will be optimized out.
        if typeid::of::<Self::Error>() == typeid::of::<F>() {
            // SAFETY: If we enter this conditional, `E` and `F` differ only in lifetimes. Lifetimes
            // are erased in runtime, so `impl Into<F> for E` has the same implementation as
            // `impl Into<T> for T` for some `T`, and that blanket implementation is a no-op.
            // Therefore, no conversion needs to happen.
            unsafe { self.unwrap_or_throw(PhantomData) }
        } else {
            match unsafe { self.intercept() } {
                Ok(value) => value,
                Err((err, handle)) => unsafe { handle.rethrow(err.into(), phantom) },
            }
        }
    }

    // This, unfortunately, needs to return a specific type for the handle, because `impl Trait`
    // would capture `Self`, and for `IexResult`, this means that values borrowed by the closure
    // would be considered borrowed even after `map_err` returns.
    unsafe fn intercept(self) -> Result<Self::Output, (Self::Error, RethrowHandle<Self::Error>)>;
}

pub struct RethrowHandle<E> {
    pub(crate) in_flight_exception: Option<lithium::InFlightException<E>>,
}

impl<E> RethrowHandle<E> {
    pub unsafe fn rethrow<F>(self, ex: F, _phantom: PhantomData<fn() -> F>) -> ! {
        match self.in_flight_exception {
            Some(handle) => unsafe { handle.rethrow(ex) },
            None => unsafe { lithium::throw(ex) },
        }
    }
}
