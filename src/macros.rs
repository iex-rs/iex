/// Use unwinding for error propagation.
///
/// This attribute can be applied to functions and closures.
///
/// Applying this attribute to a function or a closure that returns [`Result<T, E>`] turns it into a
/// function/closure that returns `#[iex] Result<T, E>`. This is an opaque type, but it implements
/// the [`Outcome`](crate::Outcome) trait, so you can use
/// [`.into_result()`](crate::Outcome::into_result) to turn it into [`Result<T, E>`].
///
/// Additionally, `expr?` inside `#[iex]`-wrapped code is interpreted as a custom operator (as
/// opposed to the built-in try operator) that propagates the error from a [`Result<T, E>`] or an
/// `#[iex] Result<T, E>` and returns a `T`.
///
/// # Pitfalls
///
/// The lifetimes may be a bit difficult to get right.
///
/// ## Functions and lifetimes
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
/// ## Closures and lifetimes
///
/// `#[iex]` closures can't take arguments whose types contain non-`'static` lifetimes. Sorry.
///
/// ## `?` in macros
///
/// `#[iex]` needs to replace the `?` operator with a custom implementation in the function body.
/// This notably fails if the `?` is generated by a macro:
///
/// ```compile_fail
/// use iex::iex;
///
/// #[iex]
/// fn returns_result(x: i32) -> Result<i32, i32> { Ok(x) }
///
/// macro_rules! ok {
///     ($e:expr) => {
///         // the `?` operator cannot be applied to type `..`
///         returns_result($e)?
///     };
/// }
///
/// #[iex]
/// fn test() -> Result<i32, i32> {
///     Err(ok!(123))
/// }
/// ```
///
/// To handle this usecase, the macro `iex_try!(<expr>)`, semanticaly identical to `<expr>?`, works
/// in `#[iex]` functions even from within macros:
///
/// ```
/// use iex::iex;
///
/// #[iex]
/// fn returns_result(x: i32) -> Result<i32, i32> { Ok(x) }
///
/// macro_rules! ok {
///     ($e:expr) => {
///         iex_try!(returns_result($e))
///     };
/// }
///
/// #[iex]
/// fn test() -> Result<i32, i32> {
///     Err(ok!(123))
/// }
/// ```
///
/// # Attributes
///
/// Rust evaluates attribute macros from top to bottom, so if `#[iex]` is not the only attribute
/// macro applied to the function/closure, the macros listed above it will be applied to the
/// original definition, and the macros listed below it will be applied to an internal closure
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
/// For a rendered example, see [`example`](crate::example).
///
/// # `#[iex(shares = ..)]`
///
/// This use is specific for `map_err` and `inspect_err`. See the documentation for
/// [`Outcome`](crate::Outcome::map_err) for more information.
///
/// # Example
///
/// ```
/// #![feature(stmt_expr_attributes, proc_macro_hygiene)]
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
///     // Closures work too
///     let closure = #[iex] || Ok(1);
///     closure()?;
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

pub use iex_derive::try_block;
