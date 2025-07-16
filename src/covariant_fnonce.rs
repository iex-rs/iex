// This is a weird abstraction.
//
// So `IexResult` wants to be covariant over `T` and `E`. This is mostly necessary because
// `ReturnPhantom` is invariant over `T` for soundness reasons, so the only chance we have to cast
// something like `&'static i32` to `&'a i32` when forwarding the return value of an `#[iex]`
// function to another `#[iex]` function is by letting `IexResult` itself be covariant.
//
// It seems like we could simply define `struct IexResult<Func, E>(Func, PhantomData<E>)`, but
// there's a catch. What *is* `T` in this case? It's `<Func as FnOnce<()>>::Output`, i.e.
// an associated type; but under syntactic variance (see RFC 1214), such types are always considered
// invariant. We *know* the `T` in `impl FnOnce() -> T` should have exactly the same variance as
// `T` in `fn() -> T`, but that's not how the type checker works.
//
// So we force covariantness with this abomination instead. `CovariantFnOnce<F, T>` is *logically*
// just `F` that is also `FnOnce() -> T`, but it's not really quite like that because `F::Output` is
// only guaranteed to be a subtype of `T`, not be equal to `T` itself. This property becomes
// a safety requirement of the type that we enforce by only making it constructible from
// `F: FnOnce() -> T`.

use core::marker::PhantomData;
use core::mem::ManuallyDrop;

// Note that calling this type does not require `F: FnOnce() -> T`, only `F: Callable`! Don't add
// that to trait bounds at the caller site unless you're constructing the object.
pub struct CovariantFnOnce<F, T> {
    func: F,
    _phantom: PhantomData<fn() -> T>,
}

impl<F, T> CovariantFnOnce<F, T> {
    pub fn new(func: F) -> Self
    where
        F: FnOnce() -> T,
    {
        Self {
            func,
            _phantom: PhantomData,
        }
    }

    pub fn call(self) -> T
    where
        F: Callable,
    {
        let value = (self.func)();
        // SAFETY: At construction time, `T = U`. After that, since `T` is covariant and `F` is
        // invariant, the `T` can only become a subtype of `U`. Therefore, casting `U` to `T` is
        // valid.
        unsafe { core::mem::transmute_copy(&ManuallyDrop::new(value)) }
    }
}

pub trait Callable: FnOnce() -> <Self as Callable>::Output {
    type Output;
}

impl<F: FnOnce() -> T, T> Callable for F {
    type Output = T;
}
