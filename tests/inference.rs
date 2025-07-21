use iex::iex;

#[iex]
pub fn infer_ok() -> Result<i32, i64> {
    if true {
        Ok(Default::default())
    } else {
        Ok(Default::default()).map_err(|e| e)
    }
}

#[iex]
pub fn infer_err() -> Result<i32, i64> {
    if true {
        Err(Default::default())
    } else {
        Err(Default::default()).map_err(|e| e)
    }
}

#[iex]
pub fn infer_question_mark_ok() -> Result<(), i64> {
    // In both cases, `T` needs to fallback to `!` / `()`
    Err(1)?;
    Err(1).map_err(|e| e)?;
    Ok(())
}

// There's exactly one type `T` satisfying `S: From<T>`, and that's `S` itself. Rust's type
// inference manages to infer that the generic parameter of `conjure()` is `S` from this. This test
// verifies that black magic in `do_try` bounds doesn't accidentally break this.
#[iex]
pub fn infer_single_impl() -> Result<(), S> {
    Err(conjure())?;
    Err(conjure()).map_err(|e| e)?;
    Ok(())
}

pub struct S;

pub fn conjure<T>() -> T {
    #[allow(clippy::empty_loop)]
    loop {}
}
