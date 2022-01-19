> **Note!** This crate is experimental and under development. It may include bugs that alter the behavior of your code
> in unexpected ways. You should review the expanded code from the macro before use, and report any bugs you may find
> to the bug tracker at https://github.com/frxstrem/cain/issues.

The [`cain!`] macro is a macro that rewrites sequential Rust branch statements into nested branches.

This can be useful in cases where you want to have branches with different types
that behave similarly, for example concrete types with a common trait, without
having to use runtime trait objects like `Box<dyn Trait>`.

The disadvantage of this is that the amount of code grows exponentially relative
to the number of sequential branches in the expanded code.

# MSRV

The minimal supported Rust version for `cain` is 1.57.0 (December 2021).

# Example

The code:

```nocompile
fn foo(): Result<i32, bool> { .. }

let a = cain! {
  let value = match foo() {
    Ok(n) => n,
    Err(b) => b,
  };

  value.to_string()
};
```

Will be expanded by the `cain!` macro into the code:

```nocompile
fn foo(): Result<i32, bool> { .. }

let a = {
  match foo() {
    Ok(n) => {
      let value = n;
      n.to_string()
    },
    Err(b) => {
      let value = b;
      b.to_string()
    }
  }
};
```
