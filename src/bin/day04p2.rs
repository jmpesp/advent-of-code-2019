fn pass(input: usize) -> bool {
    let input_s = format!("{}", input);

    // Two adjacent digits are the same
    let mut adjacent_the_same = false;
    for i in 0..(input_s.len() - 1) {
        let a = input_s.chars().nth(i + 0).unwrap();
        let b = input_s.chars().nth(i + 1).unwrap();

        if a == b {
            adjacent_the_same = true;
        }

        // Going from left to right, the digits never decrease
        let an = a.to_digit(10).unwrap();
        let bn = b.to_digit(10).unwrap();

        if an > bn {
            return false;
        }
    }

    if !adjacent_the_same {
        return false;
    }

    return true;
}

#[test]
fn test1() {
    assert!(pass(111111));
}
#[test]
fn test2() {
    assert!(!pass(223450));
}
#[test]
fn test3() {
    assert!(!pass(123789));
}

fn main() {
    let mut count = 0;
    for i in 235741..(706948 + 1) {
        if pass(i) {
            count = count + 1;
        }
    }
    println!("{}", count);
}
