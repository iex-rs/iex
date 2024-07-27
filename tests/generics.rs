use iex::{iex, Outcome};

#[iex]
fn identity<T>(a: T) -> Result<T, ()> {
    Ok(a)
}

#[iex]
fn default<T: Default>() -> Result<T, ()> {
    Ok(Default::default())
}

#[iex]
fn drop<T>(_x: T) -> Result<(), ()> {
    Ok(())
}

#[iex]
fn drop_apit(_x: impl Send) -> Result<(), ()> {
    Ok(())
}

#[test]
fn generics() {
    assert_eq!(identity(123).into_result(), Ok(123));
    assert_eq!(default().into_result(), Ok(0));
    assert_eq!(drop(123).into_result(), Ok(()));
    assert_eq!(drop_apit(123).into_result(), Ok(()));
}
