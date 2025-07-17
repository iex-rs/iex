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
pub fn covariance_in_ok<'a>(s: &'static str) -> Result<&'a str, ()> {
    if true {
        Ok::<&'static str, _>(s)
    } else {
        ok::<&'static str, _>(s)
    }
}

#[iex]
pub fn covariance_in_return_err<'a>(s: &'static str) -> Result<(), &'a str> {
    if true {
        Err::<_, &'static str>(s)
    } else {
        err::<_, &'static str>(s)
    }
}

#[iex]
pub fn covariance_in_try_err<'a>(s: &'static str) -> Result<(), &'a str> {
    if true {
        Err::<_, &'static str>(s)
    } else {
        err::<_, &'static str>(s)
    }?;
    Ok(())
}
