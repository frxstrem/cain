use crate::macros::cain;

use pretty_assertions::assert_eq;

macro_rules! test_cain_macro {
    (
        $(
            $test_name:ident: { $($input:tt)* } => { $($output:tt)* }
        ),+ $(,)?
    ) => {
        $(
            #[test]
            fn $test_name() {
                crate::util::in_test(|| {
                    let input = ::quote::quote!{ $($input)* };
                    let expected_output = ::quote::quote!{ { $($output)* } };

                    let actual_output = cain(input).unwrap();

                    assert_eq!(expected_output.to_string(), actual_output.to_string());
                })
            }
        )*
    };
}

test_cain_macro! {
    empty: {} => {},

    literal: { 1 } => { 1 },
    identifier: { a } => { a },

    simple_expr: { a + b } => { a + b },

    sipmle_stmt: { let z = 1; } => { let z = 1; },

    if_simple: { 1 + if x { a } else { b } }
        => { if x { 1 + { a } } else { 1 + { b } } },

    if_double: { 1 + if x { a } else { b } + if y { a } else { b } }
        => {
            if x {
                if y {
                    1 + { a } + { a }
                } else {
                    1 + { a } + { b }
                }
            } else if y {
                1 + { b } + { a }
            } else {
                1 + { b } + { b }
            }
        },

    if_nested_cond: { if if x { a } else { b } { c } else { d } }
        => {
            if x {
                if { a } {
                    { c }
                } else {
                    d
                }
            } else if { b } {
                { c }
            } else {
                d
            }
        },

    if_let_ident: { 1 + if let x = z { a + x } }
        => {
            if let __cain_ident__0 = z {
                if { #[allow(unused_variables, unreachable_patterns)] { matches!(&__cain_ident__0, x) } } {
                    1 + { if let (x,) = (__cain_ident__0,) { { a + x } } else { unreachable!() } }
                }
            }
        },

    match_simple: { 1 + match x { 1 => a, _ => b } }
        => { match x { 1 => 1 + a, _ => 1 + b } },

    match_double: { 1 + match x { 1 => a, _ => b } + match y { 2 => a, _ => b } }
        => {
            match x {
                1 => match y {
                    2 => 1 + a + a,
                    _ => 1 + a + b
                },
                _ => match y {
                    2 => 1 + b + a,
                    _ => 1 + b + b
                }
            }
        },

    match_ident_pat: { 1 + match x { r => a + r } }
        => {
            match x {
                __cain_ident__0
                    if { #[allow(unused_variables, unreachable_patterns)] { matches!(&__cain_ident__0, r) } }
                    => 1 + if let (r,) = (__cain_ident__0,) { a + r } else { unreachable!() },
                #[allow(unreachable_patterns, unused_variables)] r => unreachable!()
            }
        },

    match_ident_pat_2: { 1 + match x { Ok(r) => a + r, Err(r) => b + r } }
        => {
            match x {
                Ok(__cain_ident__0)
                    if { #[allow(unused_variables, unreachable_patterns)] { matches!(&__cain_ident__0, r) } }
                    => 1 + if let (r,) = (__cain_ident__0,) { a + r } else { unreachable!() },
                #[allow(unreachable_patterns, unused_variables)] Ok(r) => unreachable!(),
                Err(__cain_ident__1)
                    if { #[allow(unused_variables, unreachable_patterns)] { matches!(&__cain_ident__1, r) } }
                    => 1 + if let (r,) = (__cain_ident__1,) { b + r } else { unreachable!() },
                #[allow(unreachable_patterns, unused_variables)] Err(r) => unreachable!()
            }
        },

    match_ident_pat_3: { 1 + match x { (r, s) => r + s } }
        => {
            match x {
                (__cain_ident__0, __cain_ident__1)
                    if { #[allow(unused_variables, unreachable_patterns)] { matches!(&__cain_ident__0, r) } }
                        && { #[allow(unused_variables, unreachable_patterns)] { matches!(&__cain_ident__1, s) } }
                    => 1 + if let (r, s,) = (__cain_ident__0, __cain_ident__1,) { r + s } else { unreachable!() },
                #[allow(unreachable_patterns, unused_variables)] (r, s) => unreachable!()
            }
        },

    match_ident_pat_4: { 1 + match x { r @ s => r + s } }
        => {
            match x {
                __cain_ident__0 @ __cain_ident__1
                    if { #[allow(unused_variables, unreachable_patterns)] { matches!(&__cain_ident__0, r) } }
                        && { #[allow(unused_variables, unreachable_patterns)] { matches!(&__cain_ident__1, s) } }
                    => 1 + if let (r, s,) = (__cain_ident__0, __cain_ident__1,) { r + s } else { unreachable!() },
                #[allow(unreachable_patterns, unused_variables)] r @ s => unreachable!()
            }
        },

    match_ident_double: { 1 + match x { r => a + r, _ => c } + match y { r => b + r, _ => c } }
        => {
            match x {
                __cain_ident__1
                    if { #[allow(unused_variables, unreachable_patterns)] { matches!(&__cain_ident__1, r) } }
                    => match y {
                        __cain_ident__0
                            if { #[allow(unused_variables, unreachable_patterns)] { matches!(&__cain_ident__0, r) } }
                            => 1 + if let (r,) = (__cain_ident__1,) { a + r } else { unreachable!() } + if let (r,) = (__cain_ident__0,) { b + r } else { unreachable!() },
                        #[allow(unreachable_patterns, unused_variables)] r => unreachable!(),
                        _ => 1 + if let (r,) = (__cain_ident__1,) { a + r } else { unreachable!() } + c
                    },
                #[allow(unreachable_patterns, unused_variables)] r => unreachable!(),
                _ => match y {
                    __cain_ident__0
                        if { #[allow(unused_variables, unreachable_patterns)] { matches!(&__cain_ident__0, r) } }
                        => 1 + c + if let (r,) = (__cain_ident__0,) { b + r } else { unreachable!() },
                    #[allow(unreachable_patterns, unused_variables)] r => unreachable!(),
                    _ => 1 + c + c
                }
            }
        },

    match_nested_cond: { match match x { 1 => a, _ => b } { 1 => "foo", _ => "bar" } }
        => {
            match x {
                1 => match a {
                    1 => "foo",
                    _ => "bar"
                },
                _ => match b {
                    1 => "foo",
                    _ => "bar"
                }
            }
        },

    match_let_chain: {
        let z = match x {
            1 => 123,
            _ => "def"
        };
        println!("{}", z);
    } => {
        match x {
            1 => {
                let z = 123;
                println!("{}", z);
            },
            _ => {
                let z = "def";
                println!("{}", z);
            }
        }
    },

    closure_inner: { let f = | | 1 + match x { 1 => 123, _ => 0 }; }
        => { let f = | | match x { 1 => 1 + 123, _ => 1 + 0 }; },

    closure_outer_inner:
        {
            let f = match x {
                1 => | | 1 + match y { 1 => 123, _ => 0 },
                _ => g
            };
            f()
        } => {
            match x {
               1 => {
                   let f = | | match y { 1 => 1 + 123, _ => 1 + 0 };
                   f()
               },
               _ => {
                   let f = g;
                   f()
               }
            }
        },

    async_inner: { let f = async { 1 + match x { 1 => 123, _ => 0 } }; }
        => { let f = async { match x { 1 => 1 + 123, _ => 1 + 0 } }; },

    async_outer_inner:
        {
            let f = match x {
                1 => async { 1 + match y { 1 => 123, _ => 0 } },
                _ => g
            };
            f()
        } => {
            match x {
               1 => {
                   let f = async { match y { 1 => 1 + 123, _ => 1 + 0 } };
                   f()
               },
               _ => {
                   let f = g;
                   f()
               }
            }
        },

    try_inner: { let f = try { 1 + match x { 1 => 123, _ => 0 } }; }
        => { let f = try { match x { 1 => 1 + 123, _ => 1 + 0 } }; },

    try_outer_inner:
        {
            let f = match x {
                1 => try { 1 + match y { 1 => 123, _ => 0 } },
                _ => g
            };
            f()
        } => {
            match x {
               1 => {
                   let f = try { match y { 1 => 1 + 123, _ => 1 + 0 } };
                   f()
               },
               _ => {
                   let f = g;
                   f()
               }
            }
        },

    item_stmts: {
        let z = match x {
            1 => 123,
            _ => 0
        };

        fn square(x: i32) -> i32 { x * x }

        square(z)
    } => {
        fn square(x: i32) -> i32 { x * x }

        match x {
            1 => {
                let z = 123;
                square(z)
            },
            _ => {
                let z = 0;
                square(z)
            }
        }
    },

    loop_inner: {
        loop {
            match x {
                1 => f(),
                _ => g(),
            }
        }
    } => {
        loop {
            match x {
                1 => f(),
                _ => g(),
            }
        }
    },

    loop_outer_inner: {
        let f = match x {
            1 => a,
            _ => b
        };
        loop {
            match y {
                1 => f(),
                _ => g(),
            }
        }
    } => {
        match x {
            1 => {
                let f = a;
                loop {
                    match y {
                        1 => f(),
                        _ => g(),
                    }
                }
            },
            _ => {
                let f = b;
                loop {
                    match y {
                        1 => f(),
                        _ => g(),
                    }
                }
            }
        }
    },

    for_inner: {
        for i in 0..10 {
            match x {
                1 => f(),
                _ => g(),
            }
        }
    } => {
        for i in 0..10 {
            match x {
                1 => f(),
                _ => g(),
            }
        }
    },

    for_outer_inner: {
        let f = match x {
            1 => a,
            _ => b
        };
        for i in 0..10  {
            match y {
                1 => f(),
                _ => g(),
            }
        }
    } => {
        match x {
            1 => {
                let f = a;
                for i in 0..10 {
                    match y {
                        1 => f(),
                        _ => g(),
                    }
                }
            },
            _ => {
                let f = b;
                for i in 0..10 {
                    match y {
                        1 => f(),
                        _ => g(),
                    }
                }
            }
        }
    },

    while_inner: {
        while pred() {
            match x {
                1 => f(),
                _ => g(),
            }
        }
    } => {
        while pred() {
            match x {
                1 => f(),
                _ => g(),
            }
        }
    },

    while_inner_outer: {
        let f = match x {
            1 => a,
            _ => b
        };
        while pred() {
            match y {
                1 => f(),
                _ => g(),
            }
        }
    } => {
        match x {
            1 => {
                let f = a;
                while pred() {
                    match y {
                        1 => f(),
                        _ => g(),
                    }
                }
            },
            _ => {
                let f = b;
                while pred() {
                    match y {
                        1 => f(),
                        _ => g(),
                    }
                }
            }
        }
    },
}
