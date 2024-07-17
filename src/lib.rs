//! Idiomatic exceptions.
//!
//! Speed up the happy path of your [`Result`]-based functions by seamlessly using exceptions for
//! error propagation.
//!
//! # Crash course
//!
//! Stick [`#[iex]`](macro@iex) on all the functions that return [`Result`] to make them return an
//! efficiently propagatable `#[iex] Result`, apply `?` just like usual, and occasionally call
//! [`.into_result()`](Outcome::into_result) when you need a real [`Result`]. It's that intuitive.
//!
//! Compared to an algebraic [`Result`], `#[iex] Result` is asymmetric: it sacrifices the
//! performance of error handling, and in return:
//! - Gets rid of branching in the happy path,
//! - Reduces memory usage by never explicitly storing the error or the enum discriminant,
//! - Enables the compiler to use registers instead of memory when wrapping small objects in [`Ok`],
//! - Cleanly separates the happy and unhappy paths in the machine code, resulting in better
//!   instruction locality.
//!
//! # Example
//!
//! ```
//! # #![feature(iterator_try_collect)]
//! use iex::{iex, Outcome};
//!
//! #[iex]
//! fn checked_divide(a: u32, b: u32) -> Result<u32, &'static str> {
//!     if b == 0 {
//!         // Actually raises a custom panic
//!         Err("Cannot divide by zero")
//!     } else {
//!         // Actually returns a / b directly
//!         Ok(a / b)
//!     }
//! }
//!
//! #[iex]
//! fn checked_divide_by_many_numbers(a: u32, bs: &[u32]) -> Result<Vec<u32>, &'static str> {
//!     let mut results = Vec::new();
//!     for &b in bs {
//!         // Actually lets the panic bubble
//!         results.push(checked_divide(a, b)?);
//!     }
//!     Ok(results)
//! }
//!
//! fn main() {
//!     // Actually catches the panic
//!     let result = checked_divide_by_many_numbers(5, &[1, 2, 3, 0]).into_result();
//!     assert_eq!(result, Err("Cannot divide by zero"));
//! }
//! ```
//!
//! # All you need to know
//!
//! Functions marked [`#[iex]`](macro@iex) are supposed to return a [`Result<T, E>`] in their
//! definition. The macro rewrites them to return an opaque type `#[iex] Result<T, E>` instead. Upon
//! calling such a function, there are two things you must _immediately_ do with its output:
//! - Either you can propagate it with `?` if it's called from another [`#[iex]`](macro@iex)
//!   function,
//! - Or you must cast it to a [`Result`] via [`.into_result()`](Outcome::into_result).
//!
//! Doing anything else to the return value, e.g. storing it in a variable and reusing later does
//! not cause UB, but will not work the way you think. If you want to swallow the error, use
//! `let _ = func().into_result();` instead.
//!
//! Notably, this list does not include returning from a function with an `#[iex] Result` obtained
//! from a call of another function. You need to use `Ok(..?)`. Sorry.
//!
//! A [`Result`] is only slow when used across function boundaries as a return type. Using it within
//! a function is mostly fine, so don't hesitate to use [`.into_result()`](Outcome::into_result) if
//! you wish to match on the return value, extract the error, or call a combinator like
//! [`Result::or_else`].
//!
//! `?` automatically applies [`Into`] conversion to the error type. If you need a more complicated
//! error conversion, apply [`.map_err(..)?`](Outcome::map_err) to the `#[iex] Result` value.
//!
//! [`#[iex]`](macro@iex) works on methods. If applied to a function in an `impl Trait for Type`
//! block, the corresponding function in the `trait Trait` block should also be marked with
//! [`#[iex]`](macro@iex). Such traits are not object-safe, unless the method is restricted to
//! `where Self: Sized` (open an issue if you want me to spend time developing a workaround). A
//! particular implementation can return an algebraic [`Result`] even if the declaration is marked
//! with [`#[iex]`](macro@iex), but this requires `#[allow(refining_impl_trait)]`.

/// Use unwinding for error propagation from a function.
///
/// Applying this attribute to a function that returns [`Result<T, E>`] turns it into a function
/// that returns `#[iex] Result<T, E>`. This is an opaque type, but it implements the [`Outcome`]
/// trait, so you can use [`.into_result()`](Outcome::into_result) to turn it into [`Result<T, E>`].
///
/// Additionally, `expr?` inside an `#[iex]` function is interpreted as a custom operator (as
/// opposed to the built-in try operator) that propagates the error from a [`Result<T, E>`] or an
/// `#[iex] Result<T, E>` and returns a `T`.
///
/// # Pitfalls
///
/// The lifetimes may be a bit difficult to get right.
///
/// If a function takes an argument whose *type* has an elided lifetime *parameter*, this parameter
/// must be specified explicitly:
///
/// ```
/// use iex::iex;
/// use std::marker::PhantomData;
///
/// struct A<'a>(PhantomData<&'a ()>);
///
/// #[iex]
/// fn good(a: A<'_>) -> Result<(), ()> { Ok(()) }
///
/// // #[iex]
/// // fn bad(a: A) -> Result<(), ()> { Ok(()) }
/// ```
///
/// This is the conventional way to specify elided lifetimes on structs, so it shouldn't be a
/// nuisance.
///
/// Additionally, if an associated function captures the lifetime from the `impl` block that is not
/// mentioned in its signature, this lifetime must be specified explicitly:
///
/// ```
/// use iex::iex;
/// use std::marker::PhantomData;
///
/// struct Ref<'a, T>(Option<&'a T>);
///
/// impl<'a, T: Clone> Ref<'a, T> {
///     // If there were more lifetimes to list, you'd use #[iex(captures = "'a", captures = "'b")]
///     #[iex(captures = "'a")]
///     fn get(self) -> Result<T, ()> {
///         self.0.cloned().ok_or(())
///     }
/// }
/// ```
///
/// Don't waste time adding the capture clause everywhere, just look out for errors like this one:
///
/// ```text
/// error[E0700]: hidden type for `impl Outcome` captures lifetime that does not appear in bounds
///   --> src/lib.rs:130:5
///    |
/// 10 |   impl<'a, T: Clone> Ref<'a, T> {
///    |        -- hidden type `IexResult<..>` captures the lifetime `'a` as defined here
/// 11 |       #[iex]
///    |       ------ opaque type defined here
/// 12 | /     fn get(self) -> Result<T, ()> {
/// 13 | |         self.0.cloned().ok_or(())
/// 14 | |     }
///    | |_____^
/// ```
///
/// Finally, make sure to use the same lifetimes in `trait` and `impl`:
///
/// ```compile_fail
/// use iex::iex;
///
/// trait Trait {
///     #[iex]
///     fn takes_str(s: &'static str) -> Result<(), ()>;
/// }
///
/// impl Trait for () {
///     // error[E0308]: method not compatible with trait
///     // Use 's: &'static str' instead
///     #[iex]
///     fn takes_str(s: &str) -> Result<(), ()> {
///         Ok(())
///     }
/// }
/// ```
///
/// # Attributes
///
/// Rust evaluates attribute macros from top to bottom, so if `#[iex]` is not the only attribute
/// macro applied to the function, the macros listed above it will be applied to the original
/// function definition, and the macros listed below it will be applied to an internal closure
/// generated by `#[iex]`.
///
/// Note that this only applies to attribute *macros*; normal attributes, such as `#[inline]` and
/// `#[cfg]`, do the right thing independently from their location.
///
/// # Documentation
///
/// `#[iex]` functions are documented (by rustdoc) to return an algebraic [`Result`], just like in
/// source code, but they also have an `#[iex]` macro attached to their signature. This is a
/// sufficient indicator for those who know what `#[iex]` is, but if you use `#[iex]` in the public
/// API of a library, you probably want to write that down in prose.
///
/// For a rendered example, see [`example`].
///
/// # Example
///
/// ```
/// // The Outcome trait is required for .into_result()
/// use iex::{iex, Outcome};
///
/// fn returning_regular_result<E>(err: E) -> Result<(), E> { Err(err) }
///
/// #[iex]
/// fn returning_iex_result<E>(err: E) -> Result<(), E> { Err(err) }
///
/// #[iex]
/// fn test() -> Result<i32, String> {
///     // ? can be applied to a Result<_, String>
///     returning_regular_result("Some error happened!".to_string())?;
///
///     // ? can be applied to a Result<_, impl Into<String>> too
///     returning_regular_result("Some error happened!")?;
///
///     // The same applies to #[iex] Result
///     returning_iex_result("Some error happened!".to_string())?;
///     returning_iex_result("Some error happened!")?;
///
///     // You can also directly return a Result
///     Ok(123)
/// }
///
/// fn main() {
///     // Using an #[iex] function from a regular function requires a cast
///     let _result: Result<i32, String> = test().into_result();
/// }
/// ```
///
/// This attribute can only be applied to functions that return a [`Result`]:
///
/// ```compile_fail
/// # use iex::iex;
/// // the trait `Outcome` is not implemented for `Option<()>`
/// #[iex]
/// fn invalid_example() -> Option<()> {
///     None
/// }
/// ```
///
/// ```compile_fail
/// # use iex::iex;
/// // the trait `Outcome` is not implemented for `()`
/// #[iex]
/// fn invalid_example() {}
/// ```
pub use iex_derive::iex;

use std::cell::Cell;
use std::marker::PhantomData;
use std::panic::AssertUnwindSafe;

struct IexPanic;

thread_local! {
    static EXCEPTION: Cell<*mut ()> = const { Cell::new(std::ptr::null_mut()) };
}

mod sealed {
    pub trait Sealed {}
}

/// Properties of a generalized result type.
///
/// This unifies [`Result`] and `#[iex] Result`.
#[must_use]
pub trait Outcome: sealed::Sealed {
    /// The type of the success value.
    type Output;

    /// The type of the error value.
    type Error;

    #[doc(hidden)]
    fn get_value_or_panic<F>(self, marker: imp::Marker<F>) -> Self::Output
    where
        Self::Error: Into<F>;

    /// Apply a function to the `Err` value, leaving `Ok` untouched.
    ///
    /// This is a generalized and more efficient version of [`Result::map_err`].
    ///
    /// # Example
    ///
    /// ```
    /// use iex::{iex, Outcome};
    ///
    /// enum MyError {
    ///     IO(std::io::Error),
    ///     Custom(String),
    /// }
    ///
    /// #[iex]
    /// fn producing_io_error() -> Result<(), std::io::Error> {
    ///     Ok(())
    /// }
    ///
    /// #[iex]
    /// fn producing_string<T: std::fmt::Debug>(arg: T) -> Result<(), String> {
    ///     Err(format!("Could not handle {:?}", arg))
    /// }
    ///
    /// #[iex]
    /// fn producing_my_error() -> Result<(), MyError> {
    ///     producing_io_error().map_err(MyError::IO)?;
    ///     producing_string(123).map_err(MyError::Custom)?;
    ///     Ok(())
    /// }
    ///
    /// assert!(matches!(
    ///     producing_my_error().into_result(),
    ///     Err(MyError::Custom(s)) if s == "Could not handle 123",
    /// ));
    /// ```
    fn map_err<F, Map: FnOnce(Self::Error) -> F>(
        self,
        map: Map,
    ) -> impl Outcome<Output = Self::Output, Error = F>;

    /// Cast a generic result to a [`Result`].
    ///
    /// The [`Result`] can then be matched on, returned from a function that doesn't use
    /// [`#[iex]`](macro@iex), etc.
    fn into_result(self) -> Result<Self::Output, Self::Error>;
}

impl<T, E> sealed::Sealed for Result<T, E> {}
impl<T, E> Outcome for Result<T, E> {
    type Output = T;

    type Error = E;

    fn get_value_or_panic<F>(self, _marker: imp::Marker<F>) -> T
    where
        E: Into<F>,
    {
        self.unwrap_or_else(|error| {
            EXCEPTION.set(Box::into_raw(Box::new(error.into())).cast());
            std::panic::resume_unwind(Box::new(IexPanic))
        })
    }

    fn map_err<F, Map: FnOnce(Self::Error) -> F>(
        self,
        map: Map,
    ) -> impl Outcome<Output = Self::Output, Error = F> {
        Result::map_err(self, map)
    }

    fn into_result(self) -> Self {
        self
    }
}

struct ExceptionMapper<T, U, F: FnOnce(T) -> U>(Option<F>, PhantomData<fn(T) -> U>);

impl<T, U, F: FnOnce(T) -> U> ExceptionMapper<T, U, F> {
    fn swallow(mut self) {
        self.0.take();
        std::mem::forget(self);
    }
}

impl<T, U, F: FnOnce(T) -> U> Drop for ExceptionMapper<T, U, F> {
    fn drop(&mut self) {
        let Some(f) = self.0.take() else {
            return;
        };

        // Resolve TLS just once
        EXCEPTION.with(|exception| {
            let error_ptr: *mut T = exception.get().cast();
            if error_ptr.is_null() {
                return;
            }
            let error: T = *unsafe { Box::from_raw(error_ptr) };
            let error: U = f(error);
            let error_ptr: *mut U = Box::into_raw(Box::new(error));
            exception.set(error_ptr.cast());
        })
    }
}

#[doc(hidden)]
pub mod imp {
    use super::*;

    pub use fix_hidden_lifetime_bug;

    pub struct Marker<F>(PhantomData<F>);

    impl<F> Clone for Marker<F> {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<F> Copy for Marker<F> {}

    pub struct IexResult<T, E, Func>(Func, PhantomData<fn() -> (T, E)>);

    impl<T, E, Func> IexResult<T, E, Func> {
        pub fn new(f: Func) -> Self {
            Self(f, PhantomData)
        }
    }

    impl<T, E, Func> sealed::Sealed for IexResult<T, E, Func> {}
    impl<T, E, Func: FnOnce(Marker<E>) -> T> Outcome for IexResult<T, E, Func> {
        type Output = T;
        type Error = E;

        fn get_value_or_panic<F>(self, _marker: Marker<F>) -> T
        where
            E: Into<F>,
        {
            let exception_mapper = ExceptionMapper(
                if typeid::of::<E>() == typeid::of::<F>() {
                    // SAFETY: If we enter this conditional, E and F differ only in lifetimes.
                    // Lifetimes are erased in runtime, so `impl Into<F> for E` has the same
                    // implementation as `impl Into<T> for T` for some `T`, and that blanket
                    // implementation is a no-op. Therefore, no conversion needs to happen.
                    None
                } else {
                    Some(<E as Into<F>>::into)
                },
                PhantomData,
            );
            let output = self.0(Marker(PhantomData));
            exception_mapper.swallow();
            output
        }

        fn map_err<F, Map: FnOnce(Self::Error) -> F>(
            self,
            map: Map,
        ) -> impl Outcome<Output = Self::Output, Error = F> {
            IexResult(
                |_marker| {
                    let exception_mapper = ExceptionMapper(Some(map), PhantomData);
                    let value = self.get_value_or_panic::<E>(Marker(PhantomData));
                    exception_mapper.swallow();
                    value
                },
                PhantomData,
            )
        }

        fn into_result(self) -> Result<T, E> {
            std::panic::catch_unwind(AssertUnwindSafe(|| self.0(Marker(PhantomData)))).map_err(
                |payload| {
                    if payload.downcast_ref::<IexPanic>().is_some() {
                        let error_ptr = EXCEPTION.replace(std::ptr::null_mut()).cast();
                        *unsafe { Box::from_raw(error_ptr) }
                    } else {
                        std::panic::resume_unwind(payload)
                    }
                },
            )
        }
    }

    pub struct NoCopy;
}

extern crate self as iex;

/// Examples of rendered documentation for [`#[iex]`](macro@iex) functions.
pub mod example {
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
}
