pub mod util;

pub fn bad_fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => bad_fibonacci(n - 1) + bad_fibonacci(n - 2),
    }
}

pub fn better_fibonacci(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;

    match n {
        0 => b,
        _ => {
            for _ in 0..n {
                let c = a + b;
                a = b;
                b = c;
            }
            b
        }
    }
}

pub fn bad_add(a: u64, b: u64) -> u64 {
    match a {
        0 => b,
        _ => bad_add(a - 1, b) + 1,
    }
}

pub fn better_add(a: u64, b: u64) -> u64 {
    a + b
}
