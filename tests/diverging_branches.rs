use cain::cain;

macro_rules! test_cain {
    (
        $(
            $(#[$attr:meta])*
            $test_name:ident : { $($input:tt)* } => $expected_output:expr
        ),+ $(,)?
    ) => {
        $(
            $(#[$attr])*
            #[test]
            fn $test_name() {
                let result = { $($input)* };
                assert_eq!($expected_output, result);
            }
        )*
    };
}

test_cain! {
    simple_diverging_branches: {
        [0, 1, 2].into_iter()
            .map(|n| {
                cain!{
                    let x = match n {
                        0 => true,
                        1 => 1,
                        2 => "abc",

                        _ => panic!() as &str,
                    };

                    x.to_string()
                }
            })
            .collect::<Vec<_>>()
    } => { vec!["true", "1", "abc"] },

    double_diverging_branches: {
        [0, 1, 2, 3, 4, 5].into_iter()
            .map(|n| {
                cain!{
                    let x = match n % 3 {
                        0 => true,
                        1 => 1,
                        2 => "abc",

                        _ => panic!() as &str,
                    };

                    let y = match n % 2 {
                        0 => 0,
                        1 => "!",

                        _ => panic!() as &str,
                    };

                    format!("{}{}", x, y)
                }
            })
            .collect::<Vec<_>>()
    } => { vec!["true0", "1!", "abc0", "true!", "10", "abc!"] },
}
