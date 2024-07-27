use crate::{imp::Marker, EXCEPTION};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;

pub struct ExceptionMapper<S, T, U, F: FnOnce(S, T) -> U> {
    state: ManuallyDrop<S>,
    f: ManuallyDrop<F>,
    phantom: PhantomData<fn(S, T) -> U>,
}

impl<S, T, U, F: FnOnce(S, T) -> U> ExceptionMapper<S, T, U, F> {
    pub fn new(_marker: Marker<U>, state: S, f: F) -> Self {
        Self {
            state: ManuallyDrop::new(state),
            f: ManuallyDrop::new(f),
            phantom: PhantomData,
        }
    }

    pub fn get_in_marker(&self) -> Marker<T> {
        unsafe { Marker::new() }
    }

    pub fn get_state(&mut self) -> &mut S {
        &mut self.state
    }

    pub fn swallow(self) {
        let mut exception_mapper = ManuallyDrop::new(self);
        // take instead of drop so that if the destructor of 'state' panics, 'f' is still dropped
        let _state = unsafe { ManuallyDrop::take(&mut exception_mapper.state) };
        let _f = unsafe { ManuallyDrop::take(&mut exception_mapper.f) };
    }
}

impl<S, T, U, F: FnOnce(S, T) -> U> Drop for ExceptionMapper<S, T, U, F> {
    fn drop(&mut self) {
        // Resolve TLS just once
        EXCEPTION.with(|exception| unsafe {
            let exception = exception.get();
            // Dereference twice instead of keeping a &mut around, because self.0() may call a
            // function that uses 'exception'.
            if let Some(error) = (*exception).read::<T>() {
                let state = ManuallyDrop::take(&mut self.state);
                let f = ManuallyDrop::take(&mut self.f);
                (*exception).write::<U>(f(state, error));
            }
        })
    }
}
