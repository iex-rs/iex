use iex::iex;

#[iex]
fn temporary_across_try() -> Result<String, ()> {
    Ok(identity(&"hi".to_string())?.clone())
}

#[iex]
fn temporary_across_try_with_map_err() -> Result<String, ()> {
    Ok(identity(&"hi".to_string()).map_err(|e| e)?.clone())
}

fn identity<T>(x: T) -> Result<T, ()> {
    Ok(x)
}

#[test]
fn test() {
    assert_eq!(temporary_across_try().into_result(), Ok("hi".to_string()));
    assert_eq!(
        temporary_across_try_with_map_err().into_result(),
        Ok("hi".to_string()),
    );
}
