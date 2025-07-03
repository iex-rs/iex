use crate::{
    traits::{Outcome, Propagate, RethrowHandle},
    trying::TryPhantom,
};

impl Outcome for ! {
    type Output = !;
    type Error = !;
}

impl<T, E> Propagate<T, E> for ! {
    type RethrowHandle = !;

    unsafe fn unwrap_or_throw(self, _phantom: TryPhantom<E>) -> T {
        self
    }

    unsafe fn intercept(self) -> Result<T, (E, !)> {
        self
    }
}

impl RethrowHandle for ! {
    unsafe fn do_rethrow<F>(self, _ex: F) -> ! {
        self
    }
}
