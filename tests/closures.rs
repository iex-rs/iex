use iex::iex;

#[iex]
fn example<'a>(error_str: &'a str) -> Result<u32, &'a str> {
    // Check that receiving arguments with non-`'static` lifetimes works.
    let checked_divide = iex::closure! {
        |a: u32, b: u32, error_str: &'a str| /* -> Result<u32, &'static str> */ {
            if b == 0 {
                Err(error_str)
            } else {
                Ok(a / b)
            }
        }
    };

    checked_divide(246, 2, error_str)
}

#[test]
fn closure() {
    assert_eq!(example("Cannot divide by zero").into_result(), Ok(123));
}
