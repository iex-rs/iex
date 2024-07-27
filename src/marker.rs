use std::marker::PhantomData;

pub struct Marker<E>(PhantomData<E>);

impl<E> Marker<E> {
    pub(crate) unsafe fn new() -> Self {
        Self(PhantomData)
    }
}

impl<E> Clone for Marker<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E> Copy for Marker<E> {}
