use iex::{iex, Outcome};

#[iex]
fn produces_err() -> Result<(), String> {
    Err("Hello,".to_string())
}

#[iex]
fn maps_err() -> Result<(), String> {
    produces_err().map_err(|e| format!("{e} world!"))
}

#[iex]
fn produces_err2(s: &str) -> Result<i32, &'static str> {
    Err(&*s.to_string().leak())
}

#[iex]
fn maps_err_owned() -> Result<i32, String> {
    let s1 = "Hello,".to_string();
    Ok(produces_err2(&mut s1).map_err(
        #[iex(shares = s1)]
        |e| {
            let _s1: String = s1;
            format!("{e} world!")
        },
    )?)
}

struct A;

impl A {
    #[iex]
    fn produces_err(&mut self) -> Result<(), ()> {
        Err(())
    }

    #[iex]
    fn maps_err_owned(self) -> Result<(), ()> {
        Ok(self.produces_err().map_err(
            #[iex(shares = self)]
            |_| {
                let _self: A = self;
            },
        )?)
    }
}

#[iex]
fn maps_err_mut_ref(mut a: A) -> Result<(), ()> {
    let ar = &mut a;
    ar.produces_err().map_err(
        #[iex(shares = ar)]
        |_| {
            let _ar: &mut A = ar;
        },
    )?;
    drop(a);
    Ok(())
}

fn main() {
    let s = maps_err().into_result().unwrap_err();
    assert_eq!(s, "Hello, world!");

    let s = maps_err_owned().into_result().unwrap_err();
    assert_eq!(s, "Hello, world!");

    let _ = A.maps_err_owned().into_result();

    let _ = maps_err_mut_ref(A).into_result();
}
