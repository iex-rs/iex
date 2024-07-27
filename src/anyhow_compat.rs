use crate::{
    iex,
    iex_result::CallWithMarker,
    imp::{IexResult, Marker},
    Outcome,
};
use anyhow::{Error, Result};
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
    type ContextOutcome<C>: Outcome<Output = T, Error = Error>
    where
        Result<(), E>: anyhow::Context<(), E>,
        C: Display + Send + Sync + 'static;

    type WithContextOutcome<C, F>: Outcome<Output = T, Error = Error>
    where
        Result<(), E>: anyhow::Context<(), E>,
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;

    /// Wrap the error value with additional context.
    fn context<C>(self, context: C) -> Self::ContextOutcome<C>
    where
        Result<(), E>: anyhow::Context<(), E>,
        C: Display + Send + Sync + 'static;

    /// Wrap the error value with additional context that is evaluated lazily only once an error
    /// does occur.
    fn with_context<C, F>(self, f: F) -> Self::WithContextOutcome<C, F>
    where
        Result<(), E>: anyhow::Context<(), E>,
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T, E> Context<T, E> for Result<T, E> {
    type ContextOutcome<C> = Result<T>
    where
        Result<(), E>: anyhow::Context<(), E>,
        C: Display + Send + Sync + 'static;

    type WithContextOutcome<C, F> = Result<T>
    where
        Result<(), E>: anyhow::Context<(), E>,
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;

    fn context<C>(self, context: C) -> Result<T>
    where
        Result<(), E>: anyhow::Context<(), E>,
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|e| anyhow::Context::context(Err(e), context).unwrap_err())
    }

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        Result<(), E>: anyhow::Context<(), E>,
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| anyhow::Context::with_context(Err(e), f).unwrap_err())
    }
}

impl<T, E, Func: CallWithMarker<T, E>> Context<T, E> for IexResult<T, E, Func> {
    type ContextOutcome<C> = IexResult<T, Error, GenericContext<Self, C>>
    where
        Result<(), E>: anyhow::Context<(), E>,
        C: Display + Send + Sync + 'static;

    type WithContextOutcome<C, F> = IexResult<T, Error, GenericWithContext<Self, C, F>>
    where
        Result<(), E>: anyhow::Context<(), E>,
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;

    fn context<C>(self, context: C) -> Self::ContextOutcome<C>
    where
        Result<(), E>: anyhow::Context<(), E>,
        C: Display + Send + Sync + 'static,
    {
        IexResult::new(GenericContext {
            outcome: self,
            context,
        })
    }

    fn with_context<C, F>(self, f: F) -> Self::WithContextOutcome<C, F>
    where
        Result<(), E>: anyhow::Context<(), E>,
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        IexResult::new(GenericWithContext { outcome: self, f })
    }
}

pub struct GenericContext<R, C> {
    outcome: R,
    context: C,
}

impl<R: Outcome, C> CallWithMarker<R::Output, Error> for GenericContext<R, C>
where
    Result<(), R::Error>: anyhow::Context<(), R::Error>,
    C: Display + Send + Sync + 'static,
{
    fn call_with_marker(self, marker: Marker<Error>) -> R::Output {
        self.outcome
            .map_err(|e| anyhow::Context::context(Err(e), self.context).unwrap_err())
            .get_value_or_panic(marker)
    }
}

pub struct GenericWithContext<R, C, F: FnOnce() -> C> {
    outcome: R,
    f: F,
}

impl<R: Outcome, C, F: FnOnce() -> C> CallWithMarker<R::Output, Error>
    for GenericWithContext<R, C, F>
where
    Result<(), R::Error>: anyhow::Context<(), R::Error>,
    C: Display + Send + Sync + 'static,
{
    fn call_with_marker(self, marker: Marker<Error>) -> R::Output {
        self.outcome
            .map_err(|e| anyhow::Context::context(Err(e), (self.f)()).unwrap_err())
            .get_value_or_panic(marker)
    }
}

impl<T> Context<T, Infallible> for Option<T> {
    type ContextOutcome<C> = Result<T>
    where
        C: Display + Send + Sync + 'static;

    type WithContextOutcome<C, F> = Result<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;

    fn context<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
    {
        anyhow::Context::context(self, context)
    }

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        anyhow::Context::with_context(self, f)
    }
}
