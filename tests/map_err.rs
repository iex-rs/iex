use iex::iex;

#[iex]
fn produces_err() -> Result<(), String> {
    Err("Hello,".to_string())
}

#[iex]
fn maps_err() -> Result<(), String> {
    produces_err().map_err(|e| format!("{e} world!"))
}

#[iex]
fn inspects_err() -> Result<(), String> {
    produces_err().inspect_err(|_e: &String| {})
}

#[test]
fn simple() {
    assert_eq!(maps_err().into_result().unwrap_err(), "Hello, world!");
    assert_eq!(inspects_err().into_result().unwrap_err(), "Hello,");
}

#[iex]
fn produces_err2(s: &str) -> Result<i32, String> {
    Err(s.to_string())
}

#[iex]
fn maps_err_owned() -> Result<i32, String> {
    let mut s1 = "Hello,".to_string();
    Ok(produces_err2(&mut s1).map_err(|e| {
        let _s1: String = s1;
        format!("{e} world!")
    })?)
}

#[test]
fn shares() {
    assert_eq!(maps_err_owned().into_result().unwrap_err(), "Hello, world!");
}

struct A;

impl A {
    #[iex]
    fn produces_err(&mut self) -> Result<(), ()> {
        Err(())
    }

    #[iex]
    fn maps_err_owned(mut self) -> Result<(), ()> {
        Ok(self.produces_err().map_err(|_| {
            let _self: A = self;
        })?)
    }
}

#[iex]
fn maps_err_mut_ref(mut a: A) -> Result<(), ()> {
    let ar = &mut a;
    ar.produces_err().map_err(|_| {
        let _ar: &mut A = ar;
    })?;
    drop(a);
    Ok(())
}

#[test]
fn owned_method() {
    assert_eq!(A.maps_err_owned().into_result(), Err(()));
}

#[test]
fn mut_ref() {
    assert_eq!(maps_err_mut_ref(A).into_result(), Err(()));
}

// Ensures that error type before `map_err` can capture locals if they're not returned in the end
#[iex]
fn not_leaking_local_ref() -> Result<(), ()> {
    let mut local = 1;
    err(&mut local).map_err(|_| ())?;
    err(&mut local).map_err(|_| ())
}

#[iex]
fn err<T>(x: T) -> Result<(), T> {
    Err(x)
}

#[test]
fn test_not_leaking_local_ref() {
    assert_eq!(not_leaking_local_ref().into_result(), Err(()));
}

// Ensures a panicking callback works correctly
#[iex]
fn panicking_closure() -> Result<(), ()> {
    Err(()).map_err(|_| panic!())
}

#[test]
#[should_panic]
fn test_panicking_closure() {
    let _ = panicking_closure().into_result();
}

// Ensures a panic-on-drop callback works correctly
#[iex]
fn closure_with_panic_on_drop() -> Result<(), ()> {
    struct Bomb;

    impl Drop for Bomb {
        fn drop(&mut self) {
            panic!();
        }
    }

    let bomb = Bomb;
    let closure = move |_: ()| {
        let _bomb = bomb;
    };

    Ok(()).map_err(closure)?;

    Ok(())
}

#[test]
#[should_panic]
fn test_closure_with_panic_on_drop() {
    let _ = closure_with_panic_on_drop().into_result();
}

#[iex]
fn from_conversion() -> Result<(), String> {
    Err("Hello, world!").map_err(|e| e)?;
    Ok(())
}

#[test]
fn test_from_conversion() {
    assert_eq!(
        from_conversion().into_result(),
        Err("Hello, world!".to_string()),
    );
}
