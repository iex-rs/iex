use iex::iex;

#[iex]
fn temporary_across_try() -> Result<String, ()> {
    Ok(identity(&"hi".to_string())?.clone())
}

fn identity<T>(x: T) -> Result<T, ()> {
    Ok(x)
}

#[test]
fn test() {
    assert_eq!(temporary_across_try().into_result(), Ok("hi".to_string()));
}
