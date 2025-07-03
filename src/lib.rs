//! Unwinding-powered [`Result`]s.
//!
//! Speed up your [`Result`]-based control flow in the `Ok` path by seamlessly using exceptions for
//! error propagation, while retaining the monadic syntax beloved by Rust users.
//!
//!
//! # Example
//!
//! ```
//! use iex::iex;
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
//!
//! # Usage
//!
//! Applying [`#[iex]`](macro@iex) to functions that return [`Result`]s makes them return
//! the efficiently propagatable type `#[iex] Result` instead.
//!
//! This type is magical. **Immediately** after invoking the `#[iex]` function, you need to do one
//! of the following:
//!
//! - Cast its return value to a normal [`Result`] by calling its `into_result` method, e.g.
//!   `f().into_result()`.
//! - Propagate the error with `?`, if within another `#[iex]` function, e.g. `f()?`. You can choose
//!   to insert calls to the following methods between the function call and `?`:
//!   - [`map_err`](Result::map_err) and [`inspect_err`](Result::inspect_err),
//!   - [`context`](anyhow::Context::context) and [`with_context`](anyhow::Context::with_context)
//!     from [`anyhow`],
//!   - [`wrap_err`](eyre::WrapErr::wrap_err) and [`wrap_err_with`](eyre::WrapErr::wrap_err_with)
//!     from [`eyre`].
//! - Return the result, either implicitly or with `return`, if within another `#[iex]` function,
//!   e.g. `return f()`. `map_err` and alike can also be used in this context.
//!
//! Note the word "immediately": `#[iex] Result` should not be stored in a variable or ignored.
//! Behind the scenes, `#[iex] Result` contains a closure, and it's the act of applying `?` or
//! calling `into_result` that triggers the actual call to the function. As such, `let _ = f();`
//! will not invoke the body of `f` at all, and `let x = f(); g(); x` will invoke `g` first and `f`
//! second.
//!
//! The sample snippet above shows the vision: `#[iex]` functions typically call other `#[iex]`
//! functions, optionally add context to the error, and then propagate it with `?`. Complex error
//! handling happens rarely in comparison and uses the somewhat slower `into_result` mechanism.
//! `into_result` is also used when bridging between `#[iex]` and non-`#[iex]` functions -- this
//! might be useful, for example, if you want to keep your public API simpler, but use `#[iex]`
//! internally.
//!
//! [`#[iex]`](macro@iex) can be applied to methods. When working with traits, it needs to be
//! applied both to declaration in `trait` and the definition in `impl` blocks. Traits with `#[iex]`
//! methods are not object-safe, unless the method is restricted to `where Self: Sized` (open
//! an issue if you want me to spend time developing a workaround).
//!
//!
//! # Implementation and performance
//!
//! `#[iex]` uses [Lithium](lithium) to throw and catch exceptions. It's faster and generates less
//! code than panics, especially with a nightly compiler.
//!
//! The invoked methods and the use of `?` vs `into_result` directly correspond to the lowering and
//! its performance characteristics:
//!
//! - `f()?` and `return f()` don't catch the exception at all, implicitly letting it through.
//! - `f().map_err(...)?` and alike catch and rethrow the exception, reusing its EH context.
//! - `f().into_result()` catches the exception and destroys its EH context. Throwing further
//!   exceptions, e.g. by `return` in  `return f().into_result();`, will need to allocate a new EH
//!   context.
//!
//! Note that just blindly slapping `#[iex]` onto every single function might not improve your
//! performance at best and will decrease it at worst. Like with every other optimization, it is
//! critical to profile code and measure performance on realistic data.
//!
//!
//! # Documentation
//!
//! `#[iex]` functions are documented (by rustdoc) to return an algebraic [`Result`], just like in
//! source code, but they also have an `#[iex]` macro attached to their signature. This is a
//! sufficient indicator for those who know what `#[iex]` is, but if you use `#[iex]` in the public
//! API of a library, you probably want to write that down in prose.
//!
//! For a rendered example, see [`example`].
//!
//!
//! # Benchmark
//!
//! As a demonstration, we have rewritten [serde](https://serde.rs) and
//! [serde_json](https://crates.io/crates/serde_json) to use `#[iex]` in the deserialization path
//! and used the [Rust JSON Benchmark](https://github.com/serde-rs/json-benchmark) to compare
//! performance. These are the results:
//!
//! <table width="100%">
//!     <thead>
//!         <tr>
//!             <td rowspan="2">Speed (MB/s)</td>
//!             <th colspan="2"><code>canada</code></th>
//!             <th colspan="2"><code>citm_catalog</code></th>
//!             <th colspan="2"><code>twitter</code></th>
//!         </tr>
//!         <tr>
//!             <th>DOM</th>
//!             <th>struct</th>
//!             <th>DOM</th>
//!             <th>struct</th>
//!             <th>DOM</th>
//!             <th>struct</th>
//!         </tr>
//!     </thead>
//!     <tbody>
//!         <tr>
//!             <td><a href="https://doc.rust-lang.org/nightly/core/result/enum.Result.html"><code>Result</code></a></td>
//!             <td align="center">282.4</td>
//!             <td align="center">404.2</td>
//!             <td align="center">363.8</td>
//!             <td align="center">907.8</td>
//!             <td align="center">301.2</td>
//!             <td align="center">612.4</td>
//!         </tr>
//!         <tr>
//!             <td><code>#[iex] Result</code></td>
//!             <td align="center">282.4</td>
//!             <td align="center">565.0</td>
//!             <td align="center">439.4</td>
//!             <td align="center">1025.4</td>
//!             <td align="center">317.6</td>
//!             <td align="center">657.8</td>
//!         </tr>
//!         <tr>
//!             <td>Performance increase</td>
//!             <td align="center">0%</td>
//!             <td align="center">+40%</td>
//!             <td align="center">+21%</td>
//!             <td align="center">+13%</td>
//!             <td align="center">+5%</td>
//!             <td align="center">+7%</td>
//!         </tr>
//!     </tbody>
//! </table>
//!
//! The data is averaged between 5 runs. The repositories for data reproduction are published
//! [on GitHub](https://github.com/orgs/iex-rs/repositories).
//!
//! This benchmark only measures the happy path. When triggered, exceptions are significantly slower
//! than algebraic [`Result`]s. However, it is important to recognize that realistic programs
//! perform actions other than throwing errors, and the slowness of the error path is offset by the
//! increased speed of the happy path. For JSON parsing in particular, the break-even point is 1
//! error per 30-100k bytes parsed, depending on the data.

#![cfg_attr(doc, feature(doc_auto_cfg))]

pub use iex_derive::*;

mod outcome;
#[doc(hidden)]
pub use outcome::{Outcome, RethrowHandle};

mod iex_result;
#[doc(hidden)]
pub use iex_result::IexResult;

mod result;

pub mod example;

mod returning;
pub use returning::make_return_phantom;

mod trying;

extern crate self as iex;
