use iex::iex;

#[iex]
pub fn ok<T, E>(x: T) -> Result<T, E> {
    Ok(x)
}

#[iex]
pub fn err<T, E>(x: E) -> Result<T, E> {
    Err(x)
}

#[iex]
pub fn covariance_in_ok<'a>(n: i32, s: &'static str) -> Result<&'a str, ()> {
    match n {
        1 => Ok::<&'static str, _>(s),
        2 => ok::<&'static str, _>(s),
        3 => Ok::<&'static str, _>(s).map_err(|e| e),
        _ => ok::<&'static str, _>(s).map_err(|e| e),
    }
}

#[iex]
pub fn covariance_in_return_err<'a>(n: i32, s: &'static str) -> Result<(), &'a str> {
    match n {
        1 => Err::<_, &'static str>(s),
        2 => err::<_, &'static str>(s),
        3 => Err::<_, &'static str>(s).map_err(|e| e),
        _ => err::<_, &'static str>(s).map_err(|e| e),
    }
}

#[iex]
pub fn covariance_in_try_err<'a>(n: i32, s: &'static str) -> Result<(), &'a str> {
    match n {
        1 => Err::<_, &'static str>(s),
        2 => err::<_, &'static str>(s),
        3 => Err::<_, &'static str>(s).map_err(|e| e),
        _ => err::<_, &'static str>(s).map_err(|e| e),
    }?;
    Ok(())
}
