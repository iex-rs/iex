// As we want to implement traits for `Never` without angrying the coherence checker, we need to
// copy the definition from `never-say-never` here.
pub trait FnOutput {
    type Output;
}

impl<R> FnOutput for fn() -> R {
    type Output = R;
}

pub type Never = <fn() -> ! as FnOutput>::Output;
