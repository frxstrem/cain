/// A collection of tests that ensure that the `cain!` macro does not change the
/// behavior of already valid Rust code.
use cain::cain;

#[macro_export]
macro_rules! identity {
    ($($input:tt)*) => { { $($input)* } };
}

/// Generate tests based on input.
///
/// The code for each test is run twice: once where the `cain!` macro just emits its
/// input tokens again, and once where the `cain!` macro actually performs the macro
/// transformation.
///
/// In each test, the output from both must be identical to success. In other words,
/// the `cain!` macro transformation must not change the behavior of code that would
/// otherwise have worked.
macro_rules! test_no_behavior_change {
    (
        $cain_macro:ident!;

        $(
            $(#[$attr:meta])*
            $test_name:ident : { $($input:tt)* }
        ),+ $(,)?
    ) => {
        $(
            $(#[$attr])*
            #[test]
            fn $test_name() {
                let without_cain = {
                    use self::identity as $cain_macro;
                    $($input)*
                };
                println!("without cain: {:?}", without_cain);

                let with_cain = {
                    use ::cain::cain as $cain_macro;
                    $($input)*
                };
                println!("with cain: {:?}", with_cain);

                assert_eq!(without_cain, with_cain);
            }
        )*
    };
}

fn inc(n: &mut i32) {
    *n += 1;
}

test_no_behavior_change! {cain!;

    // https://github.com/frxstrem/cain/issues/1
    issue_1: {
        let val = Some(vec![0_usize]);

        let mut v = vec![1];

        cain! {
            let _ = match val {
                Some(mut v) => { v.pop(); },
                None => {}
            };

            v.pop().unwrap();
        }
    },

    behavior_1_a: {
        let mut a = Some(10);
        let mut b = 20;

        cain! {
            let _ = match a.as_mut() {
                Some(mut b) => inc(&mut b),
                _ => {}
            };

            inc(&mut b);
        }

        (a, b)
    },
    behavior_1_b: {
        let mut a = Some(10);
        let mut b = 20;

        cain! {
            let _ = if let Some(mut b) = a.as_mut() {
                inc(&mut b)
            } else {
            };

            inc(&mut b);
        }

        (a, b)
    },

    behavior_2_a: {
        let mut a = Some(10);
        let mut b = 20;

        cain! {
            let _ = match &mut a {
                Some(mut b) => inc(&mut b),
                _ => {}
            };

            inc(&mut b);
        }

        (a, b)
    },
    behavior_2_b: {
        let mut a = Some(10);
        let mut b = 20;

        cain! {
            let _ = if let Some(mut b) = &mut a {
                inc(&mut b)
            } else {
            };

            inc(&mut b);
        }

        (a, b)
    },

    behavior_3_a: {
        let mut a = Some(10);
        let mut b = 20;

        cain! {
            let _ = match &mut a {
                Some(ref mut b) => inc(b),
                _ => {}
            };

            inc(&mut b);
        }

        (a, b)
    },
    behavior_3_b: {
        let mut a = Some(10);
        let mut b = 20;

        cain! {
            let _ = if let Some(ref mut b) = &mut a {
                inc(b)
            } else {
            };

            inc(&mut b);
        }

        (a, b)
    },

    behavior_4_a: {
        let a = Some(10);

        cain! {
            match a {
                None => 1,
                Some(_) => 2,
            }
        }
    },
    behavior_4_b: {
        let a = Some(10);

        cain! {
            if let None = a {
                1
            } else {
                2
            }
        }
    },
}
