use crate::{
    imp::{ExceptionMapper, Marker},
    Outcome,
};
use std::mem::ManuallyDrop;

pub trait _IexForward {
    type Output;
    fn _iex_forward(self) -> Self::Output;
}

impl<E, R: Outcome> _IexForward for &mut (Marker<E>, ManuallyDrop<R>)
where
    R::Error: Into<E>,
{
    type Output = R::Output;
    fn _iex_forward(self) -> R::Output {
        let outcome = unsafe { ManuallyDrop::take(&mut self.1) };
        if typeid::of::<E>() == typeid::of::<R::Error>() {
            // SAFETY: If we enter this conditional, E and R::Error differ only in lifetimes.
            // Lifetimes are erased in runtime, so `impl Into<E> for R::Error` has the same
            // implementation as `impl Into<T> for T` for some `T`, and that blanket
            // implementation is a no-op. Therefore, no conversion needs to happen.
            outcome.get_value_or_panic(unsafe { Marker::new() })
        } else {
            let exception_mapper = ExceptionMapper::new(self.0, (), |_, err| Into::<E>::into(err));
            let output = outcome.get_value_or_panic(exception_mapper.get_in_marker());
            exception_mapper.swallow();
            output
        }
    }
}

// Autoref specialization for conversion-less forwarding. This *must* be callable without taking
// a (mutable) reference in user code, so that the LLVM optimizer has less work to do. This
// actually matters for serde.
impl<R: Outcome> _IexForward for (Marker<R::Error>, ManuallyDrop<R>) {
    type Output = R::Output;
    fn _iex_forward(self) -> R::Output {
        ManuallyDrop::into_inner(self.1).get_value_or_panic(self.0)
    }
}
