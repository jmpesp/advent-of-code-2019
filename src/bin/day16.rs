use std::io::{self, BufRead};

// As input, FFT takes a list of numbers. In the signal you
// received (your puzzle input), each number is a single digit:
// data like 15243 represents the sequence 1, 5, 2, 4, 3.

// FFT operates in repeated phases. In each phase, a new list is
// constructed with the same length as the input list. This new list
// is also used as the input for the next phase.

// Each element in the new list is built by multiplying every value
// in the input list by a value in a repeating pattern and then
// adding up the results.

// if the input list were 9, 8, 7, 6, 5 and the pattern for a given
// element were 1, 2, 3, the result would be:
// 9*1 + 8*2 + 7*3 + 6*1 + 5*2

// Then, only the ones digit is kept.

// While each element in the output array uses all of the same input
// array elements, the actual repeating pattern to use depends on
// which output element is being calculated.

// The base pattern is 0, 1, 0, -1. Then, repeat each value in the
// pattern a number of times equal to the position in the output
// list being considered.

// example: third element of the output list's repeating pattern =
// 0, 0, 0, 1, 1, 1, 0, 0, 0, -1, -1, -1

// When applying the pattern, skip the very first value exactly
// once.

fn get_repeating_pattern(index: u32) -> Vec<i32> {
    let mut output: Vec<i32> = Vec::with_capacity(index as usize * 4);
    let base_pattern: Vec<i32> = vec![0, 1, 0, -1];

    let mut skipped_first: bool = false;

    for i in 0..4 {
        for _j in 0..index {
            if !skipped_first {
                skipped_first = true;
            } else {
                output.push(base_pattern[i]);
            }
        }
    }

    output.push(0);

    return output;
}

#[test]
fn test_get_repeating_pattern() {
    assert_eq!(get_repeating_pattern(1), vec![1, 0, -1, 0]);
    assert_eq!(get_repeating_pattern(2), vec![0, 1, 1, 0, 0, -1, -1, 0]);
    assert_eq!(get_repeating_pattern(3), vec![0, 0, 1, 1, 1, 0, 0, 0, -1, -1, -1, 0]);
}

fn get_repeating_pattern_v2(index: usize, pos: usize) -> i32 {
    let base_pattern: Vec<i32> = vec![0, 1, 0, -1];
    let modulus : usize = (pos + 1) / index;
    return base_pattern[modulus % base_pattern.len()];
}

#[test]
fn test_get_repeating_pattern_v2() {
    for i in 1..3 {
        let pattern = get_repeating_pattern(i);
        for j in 0..pattern.len() {
            assert_eq!(get_repeating_pattern_v2(i as usize, j), pattern[j as usize]);
        }
    }
}

fn apply_repeating_pattern(input: Vec<i32>, pattern: Vec<i32>) -> i32 {
    let mut output: i32 = 0;

    for i in 0..input.len() {
        output += input[i] * pattern[i % pattern.len()];
    }

    return output.abs() % 10;
}

#[test]
fn test_apply_repeating_pattern() {
    assert_eq!(apply_repeating_pattern(vec![1, 2, 3, 4, 5, 6, 7, 8], get_repeating_pattern(1)), 4);
    assert_eq!(apply_repeating_pattern(vec![1, 2, 3, 4, 5, 6, 7, 8], get_repeating_pattern(2)), 8);
    assert_eq!(apply_repeating_pattern(vec![1, 2, 3, 4, 5, 6, 7, 8], get_repeating_pattern(3)), 2);
    assert_eq!(apply_repeating_pattern(vec![1, 2, 3, 4, 5, 6, 7, 8], get_repeating_pattern(4)), 2);
    assert_eq!(apply_repeating_pattern(vec![1, 2, 3, 4, 5, 6, 7, 8], get_repeating_pattern(5)), 6);
    assert_eq!(apply_repeating_pattern(vec![1, 2, 3, 4, 5, 6, 7, 8], get_repeating_pattern(6)), 1);
    assert_eq!(apply_repeating_pattern(vec![1, 2, 3, 4, 5, 6, 7, 8], get_repeating_pattern(7)), 5);
    assert_eq!(apply_repeating_pattern(vec![1, 2, 3, 4, 5, 6, 7, 8], get_repeating_pattern(8)), 8);
}

fn fft(input: Vec<i32>, phases: i32) -> Vec<i32> {
    let mut output: Vec<i32> = input.clone();

    for _phase in 0..phases {
        output = output.iter()
                       .enumerate()
                       .map(|s| apply_repeating_pattern(output.clone(), get_repeating_pattern((s.0 + 1) as u32)))
                       .collect();
    }

    return output;
}

#[test]
fn test_fft() {
    assert_eq!(fft(vec![1, 2, 3, 4, 5, 6, 7, 8], 4), vec![0, 1, 0, 2, 9, 4, 9, 8])
}

fn main() {
    let reader = io::stdin();
    let numbers: Vec<i32> =
        reader.lock()
              .lines().next().unwrap().unwrap() // iterator -> Option<io::Result<String>>
              .char_indices()                   // String -> iterator [character]
              .map(|s| s.1.to_digit(10).expect("could not turn into digit!") as i32)
              .collect();

    //println!("Input is {} characters: {:?}", numbers.len(), numbers);

    let output: Vec<String> = fft(numbers, 100).iter()
                                               .map(|s| s.to_string())
                                               .collect();

    println!("{:?}", output.join(""))
}
