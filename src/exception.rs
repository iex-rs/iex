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

#[repr(C)]
struct Just<T> {
    discriminant: usize,
    value: MaybeUninit<T>,
}

impl Exception {
    pub(crate) const fn new() -> Self {
        Self {
            data: MaybeUninit::uninit(),
        }
    }

    const fn is_small<T>() -> bool {
        size_of::<Just<T>>() <= EXCEPTION_INLINE_SIZE
    }

    pub(crate) fn write<T>(&mut self, value: T) {
        unsafe {
            let ptr = self.data.as_mut_ptr();
            if Self::is_small::<T>() {
                ptr.cast::<Just<T>>().write(Just {
                    discriminant: 1,
                    value: MaybeUninit::new(value),
                });
            } else {
                ptr.cast::<Option<Box<T>>>().write(Some(Box::new(value)));
            }
        }
    }

    pub(crate) fn clear(&mut self) {
        unsafe {
            self.data.as_mut_ptr().cast::<usize>().write(0);
        }
    }

    pub(crate) unsafe fn read<T>(&self) -> Option<T> {
        let ptr = self.data.as_ptr();
        if Self::is_small::<T>() {
            let just = ptr.cast::<Just<T>>().read();
            if just.discriminant == 0 {
                None
            } else {
                Some(just.value.assume_init())
            }
        } else {
            ptr.cast::<Option<Box<T>>>().read().map(|b| *b)
        }
    }

    pub(crate) unsafe fn read_unchecked<T>(&self) -> T {
        let ptr = self.data.as_ptr();
        if Self::is_small::<T>() {
            ptr.cast::<Just<T>>().read().value.assume_init()
        } else {
            *ptr.cast::<Box<T>>().read()
        }
    }
}
