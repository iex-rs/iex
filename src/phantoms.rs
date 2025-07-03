use crate::{Like, Outcome, Return};
use std::marker::PhantomData;

pub struct ReturnPhantom<T, E>(PhantomData<(T, E)>);

impl<T, E> ReturnPhantom<T, E> {
    pub fn new() -> Self {
        Self(PhantomData)
    }

    pub fn to_try_phantom(self) -> TryPhantom<E> {
        TryPhantom::new()
    }

    // These methods can't be on the `Return` trait because if the associated types of `outcome` and
    // our `T, E` disagree, we want to trust `T, E` more.
    pub fn assert_is_outcome<R: Like<Result<T, E>>>(self, outcome: R) -> R::This {
        // This needs to return `R::This` rather than `R`, because if `R` doesn't implement `Like`,
        // we don't want the trait solver to even try to prove preconditions of `do_return` (which
        // would yield errors about `R` not implementing `Outcome` and `Return`). Returning
        // `R::This` guarantees that the return type of `assert_is_outcome` is a non-existent type
        // in this case, which the trait solver doesn't try to operate on.
        outcome.into()
    }

    pub unsafe fn do_return<R: Outcome + Return<T, E, R::Output, R::Error>>(self, outcome: R) -> T {
        unsafe { R::This::from(outcome).unwrap_or_throw(self.to_try_phantom()) }
    }
}

impl<T, E> Clone for ReturnPhantom<T, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, E> Copy for ReturnPhantom<T, E> {}

pub struct TryPhantom<E>(PhantomData<E>);

impl<E> TryPhantom<E> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<E> Clone for TryPhantom<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E> Copy for TryPhantom<E> {}
