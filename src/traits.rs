use crate::trying::TryPhantom;

pub trait Outcome {
    type Output;
    type Error;
    type RethrowHandle: RethrowHandle;
    unsafe fn unwrap_or_throw(self, phantom: TryPhantom<Self::Error>) -> Self::Output;
    unsafe fn intercept(self) -> Result<Self::Output, (Self::Error, Self::RethrowHandle)>;
}

pub trait RethrowHandle: Sized {
    unsafe fn rethrow<F>(self, ex: F, _phantom: TryPhantom<F>) -> ! {
        unsafe { self.do_rethrow(ex) }
    }

    unsafe fn do_rethrow<F>(self, ex: F) -> !;
}
