use crate::imp::Marker;

pub trait Sealed {}

/// Properties of a generalized result type.
///
/// This unifies [`Result`] and `#[iex] Result`.
#[must_use]
pub trait Outcome: Sealed {
    /// The type of the success value.
    type Output;

    /// The type of the error value.
    type Error;

    #[doc(hidden)]
    fn get_value_or_panic(self, marker: Marker<Self::Error>) -> Self::Output;

    /// Apply a function to the `Err` value, leaving `Ok` untouched.
    ///
    /// This is a generalized and more efficient version of [`Result::map_err`].
    ///
    /// # Ownership
    ///
    /// The semantics of ownership and capturing for `#[iex] Result` complicates the use of
    /// `map_err` in some cases. Notably, using `f(...).map_err(|e| ...)` requires that `f(...)`
    /// and `|e| ...` don't capture variables in incompatible ways:
    ///
    /// ```compile_fail
    /// use iex::{iex, Outcome};
    ///
    /// struct Struct;
    ///
    /// impl Struct {
    ///     #[iex]
    ///     fn errors(&mut self) -> Result<(), i32> {
    ///         Err(123)
    ///     }
    ///     fn error_mapper(&mut self, err: i32) -> i32 {
    ///         err + 1
    ///     }
    ///     #[iex]
    ///     fn calls(&mut self) -> Result<(), i32> {
    ///         // closure requires unique access to `*self` but it is already borrowed
    ///         self.errors().map_err(|err| self.error_mapper(err))
    ///     }
    /// }
    /// ```
    ///
    /// `#[iex]` provides a workaround for this particular usecase. The pattern
    /// `(..).map_err(#[iex(shares = ..)] ..)?` (and only this pattern, the `?` is required) allows
    /// you to share variables between the fallible function and the error handler. A *mutable
    /// reference* to the variable will be visible to the fallible function, and the *value* of the
    /// variable will be visible to the error handler. This applies to `self` too:
    ///
    /// ```
    /// use iex::{iex, Outcome};
    ///
    /// struct Struct;
    ///
    /// impl Struct {
    ///     #[iex]
    ///     fn errors(&mut self) -> Result<(), i32> {
    ///         Err(123)
    ///     }
    ///     fn error_mapper(&mut self, err: i32) -> i32 {
    ///         err + 1
    ///     }
    ///     #[iex]
    ///     fn calls(&mut self) -> Result<(), i32> {
    ///         Ok(self.errors().map_err(#[iex(shares = self)] |err| self.error_mapper(err))?)
    ///     }
    /// }
    /// ```
    ///
    /// In a more complicated case, you would have to resort to the less efficient
    /// [`into_result`](Self::into_result):
    ///
    /// ```
    /// use iex::{iex, Outcome};
    ///
    /// struct Struct;
    ///
    /// impl Struct {
    ///     #[iex]
    ///     fn errors(&mut self) -> Result<(), i32> {
    ///         Err(123)
    ///     }
    ///     fn error_mapper(&mut self, err: i32) -> i32 {
    ///         err + 1
    ///     }
    ///     #[iex]
    ///     fn calls(&mut self) -> Result<(), i32> {
    ///         self.errors().into_result().map_err(|err| self.error_mapper(err))
    ///     }
    /// }
    /// ```
    ///
    /// # Example
    ///
    /// ```
    /// use iex::{iex, Outcome};
    ///
    /// enum MyError {
    ///     IO(std::io::Error),
    ///     Custom(String),
    /// }
    ///
    /// #[iex]
    /// fn producing_io_error() -> Result<(), std::io::Error> {
    ///     Ok(())
    /// }
    ///
    /// #[iex]
    /// fn producing_string<T: std::fmt::Debug>(arg: T) -> Result<(), String> {
    ///     Err(format!("Could not handle {:?}", arg))
    /// }
    ///
    /// #[iex]
    /// fn producing_my_error() -> Result<(), MyError> {
    ///     producing_io_error().map_err(MyError::IO)?;
    ///     producing_string(123).map_err(MyError::Custom)?;
    ///     Ok(())
    /// }
    ///
    /// assert!(matches!(
    ///     producing_my_error().into_result(),
    ///     Err(MyError::Custom(s)) if s == "Could not handle 123",
    /// ));
    /// ```
    fn map_err<F, Map: FnOnce(Self::Error) -> F>(
        self,
        map: Map,
    ) -> impl Outcome<Output = Self::Output, Error = F>;

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
    /// let value = f().map_err(|err| {
    ///     let _ = g().into_result();
    ///     err
    /// })?;
    /// g()?;
    /// Ok(value)
    /// # }
    /// ```
    ///
    /// despite repetitions.
    fn into_result(self) -> Result<Self::Output, Self::Error>;
}
