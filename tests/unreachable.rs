use iex::iex;

#[iex]
fn panicking() -> Result<i32, i32> {
    panic!();
    Ok(1)
}

#[test]
#[should_panic]
fn test_panicking() {
    let _ = panicking().into_result();
}
