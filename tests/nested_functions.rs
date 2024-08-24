//!
//! Tests that runctions from iex can be returned
//!
//! It currently dosn't work [but rust 2024 might fix it](https://github.com/iex-rs/iex/issues/1#issuecomment-2307934155)

use iex::Outcome;

// #[iex]
fn func1() -> Result<u32, &'static str> {
    Ok(1)
}

// #[iex]
fn func2() -> Result<u32, &'static str> {
    Ok(2)
}

// #[iex]
fn func3(b: u8) -> Result<u32, &'static str> {
    if b == 1 {
        return Ok(3);
    } else if b == 2 {
        return func1();
    }
    func2()
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
