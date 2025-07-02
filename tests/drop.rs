use iex::iex;

struct Bomb;

impl Drop for Bomb {
    fn drop(&mut self) {
        assert_eq!(g().into_result().unwrap_err(), 2);
    }
}

#[iex]
fn g() -> Result<(), i32> {
    Err(2)
}

#[iex]
fn f() -> Result<(), i32> {
    let _bomb = Bomb;
    Err::<(), i32>(1)?;
    Ok(())
}

#[test]
fn in_drop() {
    assert_eq!(f().into_result().unwrap_err(), 1);
}
