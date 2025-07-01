use iex::{Outcome, iex};

#[iex]
fn try_phantom_and_no_copy(try_phantom: i32, no_copy: i32) -> Result<i32, ()> {
    Ok(try_phantom + no_copy)
}

#[test]
fn hygiene() {
    assert_eq!(try_phantom_and_no_copy(5, 7).into_result(), Ok(12));
}
