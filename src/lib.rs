#![doc = include_str!("../README.md")]

mod macros;
mod placeholder;
mod util;

#[cfg(test)]
mod codegen_tests;

/// Rewrite branching statements to be nested.
///
/// See the [module documentation][self] for more details.
#[proc_macro]
pub fn cain(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input);

    macros::cain(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
