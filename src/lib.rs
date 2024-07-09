use std::cell::Cell;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::panic::AssertUnwindSafe;

#[derive(Clone, Copy)]
struct ExceptionState {
    address: *mut (),
}

struct IexPanic;

thread_local! {
    static EXCEPTION: Cell<ExceptionState> = const {
        Cell::new(ExceptionState {
            address: std::ptr::null_mut(),
        })
    };
}

struct InsideCatchExceptionMarker<E>(PhantomData<E>);
impl<E> Clone for InsideCatchExceptionMarker<E> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}
impl<E> Copy for InsideCatchExceptionMarker<E> {}

trait Outcome<Phantom>: Into<Result<Self::Output, Self::Error>> {
    type Output;
    type Error;
    fn get_value_or_panic(self, marker: InsideCatchExceptionMarker<Self::Error>) -> Self::Output;
    fn into_result(self) -> Result<Self::Output, Self::Error>;
}

impl<T, E> Outcome<()> for Result<T, E> {
    type Output = T;
    type Error = E;
    fn get_value_or_panic(self, _marker: InsideCatchExceptionMarker<E>) -> T {
        self.unwrap_or_else(|error| {
            let exception = EXCEPTION.get();
            unsafe {
                exception.address.cast::<E>().write(error);
            }
            std::panic::resume_unwind(Box::new(IexPanic))
        })
    }
    fn into_result(self) -> Self {
        self
    }
}

struct PanickingFunctionWrapper<T, E, F: FnOnce(InsideCatchExceptionMarker<E>) -> T>(
    F,
    PhantomData<fn() -> E>,
);

impl<T, E, F: FnOnce(InsideCatchExceptionMarker<E>) -> T> From<PanickingFunctionWrapper<T, E, F>>
    for Result<T, E>
{
    fn from(wrapper: PanickingFunctionWrapper<T, E, F>) -> Self {
        let mut error = MaybeUninit::<E>::uninit();
        let old_exception = EXCEPTION.replace(ExceptionState {
            address: error.as_mut_ptr().cast(),
        });
        let caught = std::panic::catch_unwind(AssertUnwindSafe(|| {
            wrapper.0(InsideCatchExceptionMarker(PhantomData))
        }));
        EXCEPTION.set(old_exception);
        caught.map_err(|payload| {
            if payload.downcast_ref::<IexPanic>().is_some() {
                unsafe { error.assume_init() }
            } else {
                std::panic::resume_unwind(payload)
            }
        })
    }
}

impl<Phantom, T, E, F: FnOnce(InsideCatchExceptionMarker<E>) -> T> Outcome<Phantom>
    for PanickingFunctionWrapper<T, E, F>
{
    type Output = T;
    type Error = E;
    fn get_value_or_panic(self, marker: InsideCatchExceptionMarker<E>) -> T {
        self.0(marker)
    }
    fn into_result(self) -> Result<T, E> {
        self.into()
    }
}

fn do_failing_operation_result(a: u32, b: u32) -> Result<u32, String> {
    if b == 0 {
        Err(format!("Cannot divide {a} by zero"))
    } else {
        Ok(a / b)
    }
}

// #[iex]
// fn do_failing_operation_iex(a: u32, b: u32) -> Result<u32, String> {
//     if b == 0 {
//         Err(format!("Cannot divide {a} by zero"))
//     } else {
//         Ok(a / b)
//     }
// }

#[inline(always)]
fn do_failing_operation_iex(
    a: u32,
    b: u32,
) -> impl Outcome<(u32, u32), Output = u32, Error = String> {
    #[inline(never)]
    fn implementation(marker: InsideCatchExceptionMarker<String>, a: u32, b: u32) -> u32 {
        {
            if b == 0 {
                Err(format!("Cannot divide {a} by zero"))
            } else {
                Ok(a / b)
            }
        }
        .get_value_or_panic(marker)
    }

    PanickingFunctionWrapper(move |marker| implementation(marker, a, b), PhantomData)
}

// #[iex]
// fn max_length_iex<'a>(a: &'a str, b: &'a str) -> Result<&'a str, String> {
//     if a.len() > b.len() {-
//         Ok(a)
//     } else if a.len() < b.len() {
//         Ok(b)
//     } else {
//         Err(format!("Same length!"))
//     }
// }

fn max_length_iex<'a>(
    a: &'a str,
    b: &'a str,
) -> impl Outcome<(&'a str, &'a str), Output = &'a str, Error = String> {
    PanickingFunctionWrapper(
        move |marker| {
            {
                if a.len() > b.len() {
                    Ok(a)
                } else if a.len() < b.len() {
                    Ok(b)
                } else {
                    Err(format!("Same length!"))
                }
            }
            .get_value_or_panic(marker)
        },
        PhantomData,
    )
}

fn swap_iex<'a, T>(
    a: &'a mut T,
    b: &'a mut T,
) -> impl Outcome<(&'a mut T, &'a mut T), Output = (), Error = ()> {
    fn implementation<'a, T>(marker: InsideCatchExceptionMarker<()>, a: &'a mut T, b: &'a mut T) {
        {
            std::mem::swap(a, b);
            Ok(())
        }
        .get_value_or_panic(marker)
    }

    PanickingFunctionWrapper(move |marker| implementation::<T>(marker, a, b), PhantomData)
}

// #[iex]
// fn invoke_failing_operation() -> Result<u32, String> {
//     do_failing_operation_result(5, 0)?;
//     do_failing_operation_iex(5, 0)?;
//     do_failing_operation_iex(5, 123)
// }

fn invoke_failing_operation() -> impl Outcome<(), Output = u32, Error = String> {
    fn implementation(marker: InsideCatchExceptionMarker<String>) -> u32 {
        {
            // let a = "hello".to_string();
            // let b = "world".to_string();
            // let s = max_length_iex(&a, &b).get_value_or_panic(marker);
            // println!("{s}");
            do_failing_operation_result(5, 0).get_value_or_panic(marker);
            do_failing_operation_iex(5, 0).get_value_or_panic(marker);
            do_failing_operation_iex(123, 5)
        }
        .get_value_or_panic(marker)
    }

    PanickingFunctionWrapper(move |marker| implementation(marker), PhantomData)
}

fn call_from_result() -> Result<u32, String> {
    invoke_failing_operation().into()
}

// #[test]
pub fn main() {
    println!("{:?}", call_from_result());
}
