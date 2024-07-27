use crate::{iex, Outcome};
use anyhow::Error;
use std::convert::Infallible;
use std::fmt::Display;

/// [`anyhow`](https://docs.rs/anyhow/latest/anyhow/) compatibility layer.
///
/// [`anyhow::Context`] does not work with `#[iex] Result`, but this does.
///
/// # Example
///
/// ```rust
/// use anyhow::{bail, Result};
/// use iex::{iex, Context};
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
pub trait Context<T, E> {
    /// Wrap the error value with additional context.
    #[iex]
    fn context<C>(self, context: C) -> Result<T, Error>
    where
        C: Display + Send + Sync + 'static;

    /// Wrap the error value with additional context that is evaluated lazily only once an error
    /// does occur.
    #[iex]
    fn with_context<C, F>(self, f: F) -> Result<T, Error>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<R: Outcome> Context<R::Output, R::Error> for R
where
    Result<(), R::Error>: anyhow::Context<(), R::Error>,
{
    #[iex]
    fn context<C>(self, context: C) -> Result<R::Output, Error>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|e| anyhow::Context::context(Err(e), context).unwrap_err())
    }

    #[iex]
    fn with_context<C, F>(self, f: F) -> Result<R::Output, Error>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| anyhow::Context::with_context(Err(e), f).unwrap_err())
    }
}

impl<T> Context<T, Infallible> for Option<T> {
    #[iex]
    fn context<C>(self, context: C) -> Result<T, Error>
    where
        C: Display + Send + Sync + 'static,
    {
        anyhow::Context::context(self, context)
    }

    #[iex]
    fn with_context<C, F>(self, f: F) -> Result<T, Error>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        anyhow::Context::with_context(self, f)
    }
}
