use crate::traits::Outcome;

/// # Safety
///
/// The rethrow handles must be nested correctly.
pub unsafe fn intercept<R: Outcome>(outcome: R) -> Result<R::Output, (R::Error, R::RethrowHandle)> {
    unsafe { outcome.intercept() }
}
