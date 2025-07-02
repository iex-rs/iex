use iex::iex;

#[iex]
fn phantoms(try_phantom: i32, return_phantom: i32) -> Result<i32, ()> {
    Ok(try_phantom + return_phantom)
}

#[test]
fn hygiene() {
    assert_eq!(phantoms(5, 7).into_result(), Ok(12));
}
