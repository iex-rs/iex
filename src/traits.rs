use crate::trying::TryPhantom;

// The trait hierarchy system is a bit confusing. A type implements `Outcome<Output = T, Error = E>`
// if it's logically equivalent to `Result<T, E>`, and `Propagate<T, E>` if it can be logically
// coerced to `Result<T, E>`. These things are usually equivalent, however, `!` implements
// `Propagate<T, E>` for all `T, E`, but as an `Outcome`, it's equivalent to `Result<!, !>`. This
// approach enables type inference for "normal" types while still letting `!` substitute for any
// `Result<T, E>`.

pub trait Outcome: Propagate<Self::Output, Self::Error> {
    type Output;
    type Error;
}

pub trait Propagate<T, E>: Sized {
    type RethrowHandle: RethrowHandle;
    unsafe fn unwrap_or_throw(self, phantom: TryPhantom<E>) -> T;
    unsafe fn intercept(self) -> Result<T, (E, Self::RethrowHandle)>;
}

pub trait RethrowHandle: Sized {
    unsafe fn rethrow<F>(self, ex: F, _phantom: TryPhantom<F>) -> ! {
        unsafe { self.do_rethrow(ex) }
    }

    unsafe fn do_rethrow<F>(self, ex: F) -> !;
}
