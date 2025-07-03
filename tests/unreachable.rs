use iex::iex;

#[iex]
fn panicking_expr() -> Result<i32, i32> {
    panic!()
}

#[iex]
fn panicking_stmt() -> Result<i32, i32> {
    panic!();
}

#[test]
#[should_panic]
fn test_panicking_expr() {
    let _ = panicking_expr().into_result();
}

#[test]
#[should_panic]
fn test_panicking_stmt() {
    let _ = panicking_stmt().into_result();
}
