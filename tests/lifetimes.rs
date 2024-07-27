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

#[test]
fn lifetimes() {
    assert_eq!(input_lifetimes(&1, &2).into_result(), Ok(()));
    assert_eq!(output_lifetime().into_result(), Err(()));
    assert_eq!(elided_input_lifetimes(&1, &2).into_result(), Ok(()));
    assert_eq!(
        elided_input_lifetime_struct(A(PhantomData)).into_result(),
        Ok(())
    );
}
