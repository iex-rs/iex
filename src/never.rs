use crate::{
    traits::{Outcome, RethrowHandle},
    trying::TryPhantom,
};

// As we want to implement traits for `Never` without angrying the coherence checker, we need to
// copy the definition from `never-say-never` here.
pub trait FnOutput {
    type Output;
}

impl<R> FnOutput for fn() -> R {
    type Output = R;
}

pub type Never = <fn() -> ! as FnOutput>::Output;

impl Outcome for Never {
    type Output = Never;
    type Error = Never;
    type RethrowHandle = Never;

    unsafe fn unwrap_or_throw(self, _phantom: TryPhantom<Never>) -> Never {
        self
    }

    unsafe fn intercept(self) -> Result<Never, (Never, Never)> {
        self
    }
}

impl RethrowHandle for Never {
    unsafe fn do_rethrow<F>(self, _ex: F) -> Never {
        self
    }
}
