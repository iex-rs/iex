use iex::iex;

#[iex]
fn returns_result() -> Result<i32, ()> {
    Ok(1)
}

#[iex]
fn takes_fn(f: impl iex::FnOnce0<fn() -> Result<i32, ()>>) -> Result<i32, ()> {
    f()
}

#[test]
fn test_takes_fn() {
    assert_eq!(takes_fn(returns_result).into_result(), Ok(1));
}

pub fn upcast_fn_to_fnmut(
    f: impl iex::Fn1<fn(String) -> Result<i32, ()>>,
) -> impl iex::FnMut1<fn(String) -> Result<i32, ()>> {
    f
}

pub fn upcast_fnmut_to_fnonce(
    f: impl iex::FnMut1<fn(String) -> Result<i32, ()>>,
) -> impl iex::FnOnce1<fn(String) -> Result<i32, ()>> {
    f
}

pub fn upcast_fn_to_fnonce(
    f: impl iex::Fn1<fn(String) -> Result<i32, ()>>,
) -> impl iex::FnOnce1<fn(String) -> Result<i32, ()>> {
    f
}
