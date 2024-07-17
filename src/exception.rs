use std::mem::{size_of, MaybeUninit};

const EXCEPTION_INLINE_SIZE: usize = {
    let size = std::mem::size_of::<Option<std::ptr::NonNull<()>>>();
    if size > 64 {
        size
    } else {
        64
    }
};

pub(crate) struct Exception {
    data: MaybeUninit<[u8; EXCEPTION_INLINE_SIZE]>,
}

impl Exception {
    pub(crate) const fn new() -> Self {
        Self {
            data: MaybeUninit::uninit(),
        }
    }

    const fn is_boxed<T>() -> bool {
        size_of::<Option<T>>() > EXCEPTION_INLINE_SIZE
    }

    unsafe fn write_inline<T>(&mut self, value: Option<T>) {
        self.data.as_mut_ptr().cast::<Option<T>>().write(value);
    }

    pub(crate) fn write<T>(&mut self, value: Option<T>) {
        if Self::is_boxed::<T>() {
            unsafe { self.write_inline(value.map(Box::new)) }
        } else {
            unsafe { self.write_inline(value) }
        }
    }

    unsafe fn read_inline<T>(&self) -> Option<T> {
        self.data.as_ptr().cast::<Option<T>>().read()
    }

    pub(crate) unsafe fn read<T>(&self) -> Option<T> {
        if Self::is_boxed::<T>() {
            unsafe { self.read_inline().map(|b: Box<T>| *b) }
        } else {
            unsafe { self.read_inline() }
        }
    }
}
