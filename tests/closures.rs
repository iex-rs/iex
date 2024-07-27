#![feature(stmt_expr_attributes, proc_macro_hygiene)]

use iex::{iex, Outcome};

#[iex]
fn example() -> Result<u32, &'static str> {
    let checked_divide = {
        #[iex]
        |a: u32, b: u32| -> Result<u32, &'static str> {
            if b == 0 {
                Err("Cannot divide by zero")
            } else {
                Ok(a / b)
            }
        }
    };

    checked_divide(246, 2)
}

#[test]
fn closure() {
    assert_eq!(example().into_result(), Ok(123));
}
