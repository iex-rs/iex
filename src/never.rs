use crate::{
    traits::{Outcome, RethrowHandle},
    trying::TryPhantom,
};

pub trait FnOutput {
    type Output;
}

impl<R> FnOutput for fn() -> R {
    type Output = R;
}

// This definition is deliberately better than the one in `never-say-never` and produces much better
// diagnostics.
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
