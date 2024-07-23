use crate::{iex, Outcome};
use anyhow::Error;
use std::fmt::Display;

/// [`anyhow`](https://docs.rs/anyhow/latest/anyhow/) compatibility layer.
///
/// [`anyhow::Context`] does not work with `#[iex] Result`, but this does.
///
/// # Example
///
/// ```rust
/// use anyhow::{bail, Result};
/// use iex::{iex, AnyhowContext};
///
/// #[iex]
/// fn returns_anyhow_error() -> Result<()> {
///     bail!(r"¯\_(ツ)_/¯");
/// }
///
/// #[iex]
/// fn adds_context_to_anyhow_error() -> Result<()> {
///     returns_anyhow_error().context("In adds_context_to_anyhow_error()")
/// }
/// ```
pub trait AnyhowContext: Outcome<Error = Error> {
    /// Wrap the error value with additional context.
    #[iex]
    fn context<C>(self, context: C) -> Result<Self::Output, Error>
    where
        C: Display + Send + Sync + 'static;

    /// Wrap the error value with additional context that is evaluated lazily only once an error
    /// does occur.
    #[iex]
    fn with_context<C, F>(self, f: F) -> Result<Self::Output, Error>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<R: Outcome<Error = Error>> AnyhowContext for R {
    #[iex]
    fn context<C>(self, context: C) -> Result<R::Output, Error>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|e| e.context(context))
    }

    #[iex]
    fn with_context<C, F>(self, f: F) -> Result<R::Output, Error>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| e.context(f()))
    }
}
