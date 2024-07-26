//! Examples of rendered documentation for [`#[iex]`](macro@iex) functions.

use crate::iex;

/// A simple struct containing an [`#[iex]`](macro@iex) method.
pub struct HasIexMethod;

impl HasIexMethod {
    /// Such method. Very wow.
    #[iex]
    pub fn iex_method() -> Result<(), ()> {
        Ok(())
    }
}

/// Fallible talking.
pub trait SayHello {
    /// Say hello.
    #[iex]
    fn provided_method(self) -> Result<String, ()>
    where
        Self: Sized,
    {
        Ok("Default implementation says Hello!".to_string())
    }

    /// Do nothing.
    #[iex]
    fn required_method(&self) -> Result<(), ()>;
}

impl SayHello for String {
    #[iex]
    fn provided_method(self) -> Result<String, ()> {
        Ok(self)
    }

    #[iex]
    fn required_method(&self) -> Result<(), ()> {
        Ok(())
    }
}

/// Add numbers and check for overflow.
///
/// This function tries to compute the sum of the two arguments and returns an error if the sum
/// doesn't fit in the result type. The returned error is the overflowed sum.
#[iex]
pub fn add(a: i32, b: i32) -> Result<i32, i32> {
    a.checked_add(b).ok_or(a.wrapping_add(b))
}
