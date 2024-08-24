//!
//! Tests tha errors can be returned from macros
//!
//!

use core::str;
use iex::Outcome;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Error(String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.0)
    }
}
impl std::error::Error for Error {}

macro_rules! err {
    ($e:expr) => {
        return Err(Error(format!("oh no:{}", $e)));
    };
}

macro_rules! test {
    ($e:expr) => {
        if $e == 1 {
            return Ok(2);
        } else if $e == 2 {
            Ok(1)
        } else {
            err!($e);
        }
    };
}

#[iex::iex]
fn func3(b: u8) -> Result<u32, Error> {
    test!(b)
}
#[test]
fn test_nested_functions() {
    let res = func3(1).into_result();
    assert_eq!(res, Ok(2));

    let res = func3(2).into_result();
    assert_eq!(res, Ok(1));

    let res = func3(3).into_result();
    assert!(res.is_err());
}
