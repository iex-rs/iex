use crate::{outcome::RethrowHandle, Outcome};
use core::marker::PhantomData;

pub struct IexResult<Func, E> {
    pub closure: Func,
    pub phantom: PhantomData<fn() -> E>,
}

impl<Func: FnOnce() -> T, T, E> IexResult<Func, E> {
    /// Cast a generic result to a [`Result`].
    ///
    /// The [`Result`] can then be matched on, returned from a function that doesn't use
    /// [`#[iex]`](macro@crate::iex), etc.
    ///
    /// This method is typically slow on complex code. Avoid it in the hot path if you can. For
    /// example,
    ///
    /// ```rust
    /// # use iex::{iex, Outcome};
    /// # #[iex] fn f() -> Result<(), ()> { Ok(()) }
    /// # #[iex] fn g() -> Result<(), ()> { Ok(()) }
    /// # #[iex] fn fg() -> Result<(), ()> {
    /// let result = f().into_result();
    /// g()?;
    /// result
    /// # }
    /// ```
    ///
    /// is perhaps better written as
    ///
    /// ```rust
    /// # use iex::{iex, Outcome};
    /// # #[iex] fn f() -> Result<(), ()> { Ok(()) }
    /// # #[iex] fn g() -> Result<(), ()> { Ok(()) }
    /// # #[iex] fn fg() -> Result<(), ()> {
    /// let value = f().inspect_err(|_| drop(g().into_result()))?;
    /// g()?;
    /// Ok(value)
    /// # }
    /// ```
    ///
    /// despite repetitions.
    pub fn into_result(self) -> Result<T, E> {
        lithium::catch(|| unsafe { self.unwrap_or_throw(PhantomData) })
    }
}

#[diagnostic::do_not_recommend]
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
}
