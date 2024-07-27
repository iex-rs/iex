use iex::{iex, Outcome};

fn result_divide(a: u32, b: u32) -> Result<u32, &'static str> {
    if b == 0 {
        Err("Cannot divide by zero")
    } else {
        Ok(a / b)
    }
}

#[iex]
fn checked_divide(a: u32, b: u32) -> Result<u32, &'static str> {
    if b == 0 {
        Err("Cannot divide by zero")
    } else {
        Ok(a / b)
    }
}

#[iex]
fn invoke_failing_operation() -> Result<u32, String> {
    result_divide(6, 0)?;
    result_divide(5, 123)?;
    checked_divide(7, 0)?;
    Ok(checked_divide(5, 123)?)
}

#[test]
fn simple_propagation() {
    assert_eq!(
        invoke_failing_operation().into_result().unwrap_err(),
        "Cannot divide by zero",
    );
}
