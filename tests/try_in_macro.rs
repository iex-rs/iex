//!
//! That that we can use `?` in a macro
//!

use iex::Outcome;

#[iex::iex]
fn func1() -> Result<u32, &'static str> {
    Ok(1)
}

macro_rules! test {
    () => {
        func1()?
    };
}
#[iex::iex]
fn func3(b: u8) -> Result<u32, &'static str> {
    if b == 1 {
        return Ok(3);
    }
    Ok(test!())
}

#[test]
fn test_nested_functions() {
    let res = func3(1).into_result();
    assert_eq!(res, Ok(3));

    let res = func3(2).into_result();
    assert_eq!(res, Ok(1));

    let res = func3(3).into_result();
    assert_eq!(res, Ok(2));
}
