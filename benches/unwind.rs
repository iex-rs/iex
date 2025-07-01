use criterion::{Criterion, black_box, criterion_group, criterion_main};
use iex::{Outcome, iex};

#[iex]
fn unwind(n: i32) -> Result<(), &'static str> {
    // let _vec = black_box(vec![1]);
    if n > 0 {
        black_box(Ok(unwind(n - 1)?))
    } else {
        Err("Overflow")
    }
}

fn start_unwind(n: i32) {
    let _ = unwind(n).into_result();
}

fn result(n: i32) -> Result<(), &'static str> {
    let _vec = black_box(vec![1]);
    let _vec = black_box(vec![2]);
    let _vec = black_box(vec![3]);
    let _vec = black_box(vec![4]);
    let _vec = black_box(vec![5]);
    let _vec = black_box(vec![6]);
    let _vec = black_box(vec![7]);
    let _vec = black_box(vec![8]);
    let _vec = black_box(vec![9]);
    let _vec = black_box(vec![10]);
    if n > 0 {
        black_box(Ok(result(n - 1)?))
    } else {
        Err("Overflow")
    }
}

fn start_result(n: i32) {
    let _ = result(n);
}

fn main() {
    for n in 0..100000 {
        start_unwind(black_box(100));
    }
}
