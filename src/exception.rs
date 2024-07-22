use std::mem::{align_of, size_of, MaybeUninit};

pub(crate) struct Exception {
    data: MaybeUninit<[usize; 8]>,
}

#[repr(C)]
struct Just<T> {
    discriminant: usize,
    value: MaybeUninit<T>,
}

impl Exception {
    pub(crate) const fn new() -> Self {
        Self {
            data: MaybeUninit::zeroed(),
        }
    }

    const fn is_small<T>() -> bool {
        size_of::<Just<T>>() <= size_of::<Exception>()
    }

    unsafe fn write_raw<T>(&mut self, value: T) {
        let ptr = self.data.as_mut_ptr().cast::<T>();
        if align_of::<T>() <= align_of::<usize>() {
            ptr.write(value);
        } else {
            ptr.write_unaligned(value);
        }
    }

    pub(crate) fn write<T>(&mut self, value: T) {
        unsafe {
            if Self::is_small::<T>() {
                self.write_raw(Just {
                    discriminant: 1,
                    value: MaybeUninit::new(value),
                });
            } else {
                self.write_raw(Some(Box::new(value)));
            }
        }
    }

    pub(crate) fn clear(&mut self) {
        unsafe { self.write_raw(0usize) }
    }

    unsafe fn read_raw<T>(&self) -> T {
        let ptr = self.data.as_ptr().cast::<T>();
        if align_of::<T>() <= align_of::<usize>() {
            ptr.read()
        } else {
            ptr.read_unaligned()
        }
    }

    pub(crate) unsafe fn read<T>(&self) -> Option<T> {
        if Self::is_small::<T>() {
            let just = self.read_raw::<Just<T>>();
            if just.discriminant == 0 {
                None
            } else {
                Some(just.value.assume_init())
            }
        } else {
            self.read_raw::<Option<Box<T>>>().map(|b| *b)
        }
    }

    pub(crate) unsafe fn read_unchecked<T>(&self) -> T {
        if Self::is_small::<T>() {
            self.read_raw::<Just<T>>().value.assume_init()
        } else {
            *self.read_raw::<Box<T>>()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn overaligned() {
        let mut exc = Exception::new();
        exc.write(123u128);
        assert_eq!(unsafe { exc.read_unchecked::<u128>() }, 123);
    }
}
