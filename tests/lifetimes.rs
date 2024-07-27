use iex::{iex, Outcome};
use std::marker::PhantomData;

#[iex]
fn input_lifetimes<'a, 'b>(_x: &'a u32, _y: &'b u32) -> Result<(), ()> {
    Ok(())
}

#[iex]
fn output_lifetime<'a>() -> Result<&'a u32, ()> {
    Err(())
}

#[iex]
fn elided_input_lifetimes(_x: &u32, _y: &u32) -> Result<(), ()> {
    Ok(())
}

struct A<'a>(PhantomData<&'a ()>);

#[iex]
fn elided_input_lifetime_struct(_a: A<'_>) -> Result<(), ()> {
    Ok(())
}

#[iex]
fn max_length<'a>(a: &'a str, b: &'a str) -> Result<&'a str, &'static str> {
    if a.len() > b.len() {
        Ok(a)
    } else if a.len() < b.len() {
        Ok(b)
    } else {
        Err("Same length!")
    }
}

#[iex]
fn swap<'a, T>(a: &'a mut T, b: &'a mut T) -> Result<(), String> {
    std::mem::swap(a, b);
    Ok(())
}

#[test]
fn lifetimes() {
    assert_eq!(input_lifetimes(&1, &2).into_result(), Ok(()));
    assert_eq!(output_lifetime().into_result(), Err(()));
    assert_eq!(elided_input_lifetimes(&1, &2).into_result(), Ok(()));
    assert_eq!(
        elided_input_lifetime_struct(A(PhantomData)).into_result(),
        Ok(())
    );
    assert_eq!(max_length("Hello, ", "world!").into_result(), Ok("Hello, "));
}

#[test]
fn mutability() {
    let mut x = 1;
    let mut y = 2;
    assert_eq!(swap(&mut x, &mut y).into_result(), Ok(()));
    assert_eq!(x, 2);
    assert_eq!(y, 1);
}
