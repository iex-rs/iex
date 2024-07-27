use crate::{imp::Marker, outcome::Sealed, IexPanic, Outcome, EXCEPTION};

impl<T, E> Sealed for Result<T, E> {}

impl<T, E> Outcome for Result<T, E> {
    type Output = T;

    type Error = E;

    fn get_value_or_panic(self, _marker: Marker<E>) -> T {
        self.unwrap_or_else(|error| {
            EXCEPTION.with(|exception| unsafe { &mut *exception.get() }.write(error));
            // This does not allocate, because IexPanic is a ZST.
            std::panic::resume_unwind(Box::new(IexPanic))
        })
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
        Result::inspect_err(self, f)
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
        Result::map_err(self, op)
    }

    fn into_result(self) -> Self {
        self
    }
}
