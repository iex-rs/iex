use iex::{iex, Outcome};

#[iex]
fn marker_and_no_copy(marker: i32, no_copy: i32) -> Result<i32, ()> {
    Ok(marker + no_copy)
}

#[test]
fn hygiene() {
    assert_eq!(marker_and_no_copy(5, 7).into_result(), Ok(12));
}
