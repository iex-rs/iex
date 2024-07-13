#![feature(never_type)]

use iex::{iex, Outcome};

#[inline(never)]
fn result_divide(a: u32, b: u32) -> Result<u32, &'static str> {
    if b == 0 {
        Err("Cannot divide by zero")
    } else {
        Ok(a / b)
    }
}

#[iex]
#[inline(never)]
fn checked_divide(a: u32, b: u32) -> Result<u32, &'static str> {
    if b == 0 {
        Err("Cannot divide by zero")
    } else {
        Ok(a / b)
    }
}

#[iex]
fn max_length<'a>(a: &'a str, b: &'a str) -> Result<&'a str, String> {
    if a.len() > b.len() {
        Ok(a)
    } else if a.len() < b.len() {
        Ok(b)
    } else {
        Err(format!("Same length!"))
    }
}

#[iex]
#[inline(never)]
fn swap<'a, T>(a: &'a mut T, b: &'a mut T) -> Result<(), String> {
    std::mem::swap(a, b);
    Ok(())
}

struct A;

impl A {
    #[iex]
    fn static_method() -> Result<(), &'static str> {
        Ok(())
    }

    #[iex]
    fn value_method(self) -> Result<Self, &'static str> {
        Ok(self)
    }

    #[iex]
    fn ref_method<'a>(&'a self) -> Result<&'a Self, &'static str> {
        Ok(self)
    }

    #[iex]
    fn mut_method<'a>(&'a mut self) -> Result<&'a mut Self, &'static str> {
        Ok(self)
    }
}

#[iex]
#[inline(never)]
fn invoke_failing_operation() -> Result<u32, String> {
    result_divide(6, 0)?;
    result_divide(5, 123)?;
    checked_divide(7, 0)?;
    let mut x = 1;
    let mut y = 2;
    swap(&mut x, &mut y)?;
    A::static_method()?;
    Ok(checked_divide(5, 123)?)
}

fn main() {
    eprintln!("{:?}", invoke_failing_operation().into_result());
}
