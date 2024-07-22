# Idiomatic exceptions for Rust

![Crates.io Version](https://img.shields.io/crates/v/iex)
[![docs.rs](https://img.shields.io/docsrs/iex)](https://docs.rs/iex/latest/iex/)

Speed up the happy path of your `Result`-based functions by seamlessly using exceptions for error
propagation.

# Crash course

Stick `#[iex]` on all the functions that return `Result` to make them return an efficiently
propagatable `#[iex] Result`, apply `?` just like usual, and occasionally call `.into_result()`
when you need a real `Result`. It's that intuitive.

Compared to an algebraic `Result`, `#[iex] Result` is asymmetric: it sacrifices the performance of
error handling, and in return:
- Gets rid of branching in the happy path,
- Reduces memory usage by never explicitly storing the error or the enum discriminant,
- Enables the compiler to use registers instead of memory when wrapping small objects in `Ok`,
- Cleanly separates the happy and unhappy paths in the machine code, resulting in better instruction
  locality.

# Benchmark

As a demonstration, we have rewritten [serde](https://serde.rs) and [serde_json](https://crates.io/crates/serde_json) to use `#[iex]` in the
deserialization path and used the [Rust JSON Benchmark](https://github.com/serde-rs/json-benchmark) to compare performance. These are the
results:

<table width="100%">
    <thead>
        <tr>
            <td rowspan="2">Speed (MB/s)</td>
            <th colspan="2"><code>canada</code></th>
            <th colspan="2"><code>citm_catalog</code></th>
            <th colspan="2"><code>twitter</code></th>
        </tr>
        <tr>
            <th>DOM</th>
            <th>struct</th>
            <th>DOM</th>
            <th>struct</th>
            <th>DOM</th>
            <th>struct</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><code>Result</code></td>
            <td align="center">296.2</td>
            <td align="center">439.0</td>
            <td align="center">392.4</td>
            <td align="center">876.8</td>
            <td align="center">274.8</td>
            <td align="center">536.4</td>
        </tr>
        <tr>
            <td><code>#[iex] Result</code></td>
            <td align="center">294.8</td>
            <td align="center">537.0</td>
            <td align="center">400.6</td>
            <td align="center">940.6</td>
            <td align="center">303.8</td>
            <td align="center">568.8</td>
        </tr>
        <tr>
            <td>Performance increase</td>
            <td align="center">-0.5%</td>
            <td align="center">+22%</td>
            <td align="center">+2%</td>
            <td align="center">+7%</td>
            <td align="center">+11%</td>
            <td align="center">+6%</td>
        </tr>
    </tbody>
</table>

The data is averaged between 5 runs. The repositories for data reproduction are published
[on GitHub](https://github.com/orgs/iex-rs/repositories).

# Example

```rust
use iex::{iex, Outcome};

#[iex]
fn checked_divide(a: u32, b: u32) -> Result<u32, &'static str> {
    if b == 0 {
        // Actually raises a custom panic
        Err("Cannot divide by zero")
    } else {
        // Actually returns a / b directly
        Ok(a / b)
    }
}

#[iex]
fn checked_divide_by_many_numbers(a: u32, bs: &[u32]) -> Result<Vec<u32>, &'static str> {
    let mut results = Vec::new();
    for &b in bs {
        // Actually lets the panic bubble
        results.push(checked_divide(a, b)?);
    }
    Ok(results)
}

fn main() {
    // Actually catches the panic
    let result = checked_divide_by_many_numbers(5, &[1, 2, 3, 0]).into_result();
    assert_eq!(result, Err("Cannot divide by zero"));
}
```
