use iex::{Outcome, iex};

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
fn checked_divide_by_many_numbers(a: u32, bs: &[u32]) -> Result<Vec<u32>, &'static str> {
    let mut results = Vec::new();
    for &b in bs {
        // Actually lets the panic bubble
        results.push(checked_divide(a, b)?);
    }
    Ok(results)
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
    assert_eq!(
        checked_divide_by_many_numbers(5, &[2, 3]).into_result(),
        Ok(vec![2, 1]),
    );
}
