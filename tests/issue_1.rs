use cain::cain;

// https://github.com/frxstrem/cain/issues/1
#[test]
fn issue_1() {
    let val = vec![0_usize];
    let mut v = vec![1];

    cain! {
        let _ = match val {
            mut v => { v.pop(); }
        };

        v.pop().unwrap();
    }
}
