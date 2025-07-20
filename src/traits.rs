use crate::trying::TryPhantom;

pub trait Outcome {
    type Output;
    type Error;
    type RethrowHandle: RethrowHandle;

    /// # Safety
    ///
    /// This function throws a Lithium exception of type `Self::Error`.
    unsafe fn unwrap_or_throw(self, phantom: TryPhantom<Self::Error>) -> Self::Output;

    /// # Safety
    ///
    /// Lithium exceptions may not be thrown across the call frame until the rethrow handle is
    /// dropped.
    unsafe fn intercept(self) -> Result<Self::Output, (Self::Error, Self::RethrowHandle)>;
}

pub trait RethrowHandle: Sized {
    /// # Safety
    ///
    /// This function throws a Lithium exception of type `F`.
    unsafe fn rethrow<F>(self, ex: F, _phantom: TryPhantom<F>) -> !;
}
