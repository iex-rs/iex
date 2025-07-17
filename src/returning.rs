use crate::{never::Never, traits::Outcome, trying::TryPhantom};
use core::marker::PhantomData;

// When returning a mismatching type from a function, we want to show a readable error like
//     expected `Result<{T1}, {E1}>`, found `Result<{T2}, {E2}>`
// for *any* instances of `Outcome`, not just `REsult` itself. So that means we have to read
// `<R as Outcome>::Output` and `<R as Outcome>::Error`. But if the returned type isn't even
// an `Outcome`, we *still* want to show a readable error like
//     expected `Result<{T1}, {E1}>`, found `R`
// So there's two successive checks: first, `AnyReturn` checks that the value is an `Outcome`, and
// then `Return` validates the `Output` and `Error` types. There's also another issue: since `!`
// cannot automatically coerce to `impl Outcome` like it can to `Result`, we have to explicitly
// implement helper traits for `!` here.

// `Expected` is only used for diagnostics.
#[diagnostic::on_unimplemented(
    message = "mismatched types",
    label = "expected `{Expected}`, found `{Self}`"
)]
pub trait AnyReturn<Expected> {
    // `Result<Self::Output, Self::Error>`.
    type AsResult;
}

#[diagnostic::do_not_recommend]
impl<Expected, R: Outcome> AnyReturn<Expected> for R {
    type AsResult = Result<R::Output, R::Error>;
}

#[diagnostic::do_not_recommend]
impl<Expected> AnyReturn<Expected> for Never {
    type AsResult = Never;
}

#[diagnostic::on_unimplemented(
    message = "mismatched types",
    label = "expected `Result<{T}, {E}>`, found `{Self}`"
)]
pub trait Return<T, E, R>: Sized {
    fn map_outcome(outcome: R) -> impl Outcome<Output = T, Error = E>;
}

#[diagnostic::do_not_recommend]
impl<R: Outcome<Output = T, Error = E>, T, E> Return<T, E, R> for Result<T, E> {
    fn map_outcome(outcome: R) -> impl Outcome<Output = T, Error = E> {
        outcome
    }
}

#[diagnostic::do_not_recommend]
impl<T, E> Return<T, E, Never> for Never {
    #[allow(unreachable_code)]
    fn map_outcome(outcome: Never) -> impl Outcome<Output = T, Error = E> {
        outcome as Result<T, E>
    }
}

// The variance here is... tricky. `ReturnPhantom` is mainly lifetime-coerced in two contexts:
//
// - When returning values via `do_return`, the phantom is cast to the type of the returned
//   expression.
// - When initializing the result with `IexResult::new`, the phantom is cast to the signature return
//   type.
//
// Pay very close attention to the fact that instead of a sensible pipeline like
// "returned expression -> phantom -> signature return type", we cast the phantom in *both* the
// would-be covariant direction (to signature return type) and the would-be contravariant direction
// (to returned expression type). This forces us to declare `ReturnPhantom` as invariant to prevent
// unsoundness.
//
// Note that this doesn't prevent functions like this from compiling:
//     #[iex]
//     fn f<'a>() -> Result<(), &'a str> {
//         Err("s" as &'static str)
//     }
// ...since the outcome remains covariant, and that's enough: its lifetimes are correctly adjusted
// to the invariant lifetimes in the phantom.
pub struct ReturnPhantom<T, E>(PhantomData<*mut (T, E)>);

pub fn make_return_phantom<T, E>() -> ReturnPhantom<T, E> {
    ReturnPhantom(PhantomData)
}

impl<T, E> ReturnPhantom<T, E> {
    pub fn to_try_phantom(self) -> TryPhantom<E> {
        TryPhantom::new()
    }

    pub unsafe fn do_return<R: AnyReturn<Result<T, E>, AsResult: Return<T, E, R>>>(
        self,
        outcome: R,
    ) -> T {
        unsafe { R::AsResult::map_outcome(outcome).unwrap_or_throw(self.to_try_phantom()) }
    }

    pub unsafe fn do_return_divergent(self, _outcome: Result<T, E>) -> ! {
        unsafe {
            core::hint::unreachable_unchecked();
        }
    }
}

impl<T, E> Clone for ReturnPhantom<T, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, E> Copy for ReturnPhantom<T, E> {}
