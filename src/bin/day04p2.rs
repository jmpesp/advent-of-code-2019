#[derive(Copy, Clone, Debug)]
struct Run {
    value: u32,
    length: i32,
}

fn pass(input: usize) -> bool {
    let input_s = format!("{:06}", input);

    if input_s.len() != 6 {
        return false;
    }

    let mut runs: Vec<Run> = Default::default();

    for i in 0..input_s.len() {
        let a = input_s.chars().nth(i + 0).unwrap();
        let an = a.to_digit(10).unwrap();

        if runs.len() == 0 {
            runs.push(Run {
                value: an,
                length: 1,
            });
        } else {
            let last_run_index = runs.len() - 1;
            let mut last_run = &mut runs[last_run_index];

            if an == last_run.value {
                last_run.length = last_run.length + 1;
                runs[last_run_index] = *last_run;
            } else {
                runs.push(Run {
                    value: an,
                    length: 1,
                });
            }
        }
    }

    //println!("{:?}", runs);

    // going from left to right, the digits never decrease
    for i in 0..(runs.len() - 1) {
        if runs[i + 0].value > runs[i + 1].value {
            return false;
        }
    }

    // two adjacent digits are the same
    // at least one set of adjacent digits has to be the same
    // the (single) two adjacent matching digits are not part of a larger group of matching
    // digits:
    // 123444 is bad because there's no single group of 2
    // 688999 is ok because 88
    let mut at_least_one_run_two_adjacent = false;

    for i in 0..runs.len() {
        if runs[i].length == 2 {
            at_least_one_run_two_adjacent = true;
        }
    }

    return at_least_one_run_two_adjacent;
}

#[test]
fn test1() {
    // meets these criteria because the digits never decrease and all repeated digits are exactly two digits long.
    assert!(pass(112233));
}
#[test]
fn test2() {
    // no longer meets the criteria (the repeated 44 is part of a larger group of 444).
    assert!(!pass(123444));
    assert!(pass(122334));
}
#[test]
fn test3() {
    // meets the criteria (even though 1 is repeated more than twice, it still contains a double 22).
    assert!(pass(111122));
}
#[test]
fn test4() {
    assert!(pass(235778));
}
#[test]
fn test5() {
    assert!(!pass(235789)); // doesn't have two adjacent
    assert!(!pass(235790));
    assert!(pass(688999)); // has two adjacent, doesn't matter there's a 999!
}

fn main() {
    let mut count = 0;
    for i in 235741..(706948 + 1) {
        if pass(i) {
            println!("g{:06}", i);
            count = count + 1;
        } else {
            println!("b{:06}", i);
        }
    }
    println!("{}", count);
}
