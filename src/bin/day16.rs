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

fn fft(input: Vec<i32>, phases: i32, message_offset: usize) -> Vec<i32> {
    let mut output: Vec<i32> = input.clone();

    /*
    // part 2:
    //
    // for high index, last column doesn't change!
    //
    // if sufficiently high in list, can approximate:
    // n1 + n2 + n3 + n4 = v4 = v3 + n1
    //  0 + n2 + n3 + n4 = v3 = v2 + n2
    //  0 +  0 + n3 + n4 = v2 = v1 + n3
    //  0 +  0 +  0 + n4 = v1 = n4

    // this should work up to the half way point + 1
    // reason is that length = N, index (N / 2) + 1 is
    // 000000000000000000000011111111111111111111111111
    // and shifts over to 1 at half way point

    // so throw out the pattern stuff and simply add (abs mod 10)

    assert!(message_offset > (output.len() / 2));

    let min = message_offset;
    let max = output.len();

    for _phase in 0..phases {
        println!("{}", _phase);
        for index in (min..(max - 1)).rev() {
            output[index] = (output[index] + output[index + 1]).abs() % 10;
        }
    }

    return output;
    */

    // part 1:
    for _phase in 0..phases {
        let mut inner: Vec<i32> = Vec::with_capacity(output.len());
        inner.resize(output.len(), 0);

        let mut min = 0;
        let max = output.len();

        if message_offset != 0 {
            min = message_offset;
        }

        for index in (min..max).rev() {
            let mut v = 0;

            for mut pos in index..output.len() {
                let p = get_repeating_pattern_v2(index + 1, pos);
                if p == 0 {
                    pos += index;
                    continue;
                }
                v += output[pos] * p;
            }

            inner[index] = v.abs() % 10;
        }

        output = inner;
    }

    return output;
}

#[test]
fn test_fft() {
    assert_eq!(fft(vec![1, 2, 3, 4, 5, 6, 7, 8], 4, 0), vec![0, 1, 0, 2, 9, 4, 9, 8]);

    assert_eq!(fft(vec![8, 0, 8, 7, 1, 2, 2, 4, 5, 8, 5, 9, 1, 4, 5, 4, 6, 6, 1, 9, 0, 8, 3, 2, 1, 8, 6, 4, 5, 5, 9, 5], 100, 0),
                   vec![2, 4, 1, 7, 6, 1, 7, 6, 4, 8, 0, 9, 1, 9, 0, 4, 6, 1, 1, 4, 0, 3, 8, 7, 6, 3, 1, 9, 5, 5, 9, 5]);

    assert_eq!(fft(vec![1, 9, 6, 1, 7, 8, 0, 4, 2, 0, 7, 2, 0, 2, 2, 0, 9, 1, 4, 4, 9, 1, 6, 0, 4, 4, 1, 8, 9, 9, 1, 7], 100, 0),
                   vec![7, 3, 7, 4, 5, 4, 1, 8, 5, 5, 7, 2, 5, 7, 2, 5, 9, 1, 4, 9, 4, 6, 6, 5, 9, 9, 6, 3, 9, 9, 1, 7]);

    assert_eq!(fft(vec![6, 9, 3, 1, 7, 1, 6, 3, 4, 9, 2, 9, 4, 8, 6, 0, 6, 3, 3, 5, 9, 9, 5, 9, 2, 4, 3, 1, 9, 8, 7, 3], 100, 0),
                   vec![5, 2, 4, 3, 2, 1, 3, 3, 2, 9, 2, 9, 9, 8, 6, 0, 6, 8, 8, 0, 4, 9, 5, 9, 7, 4, 8, 6, 9, 8, 7, 3]);

    // test message stuff
    assert_eq!(fft(vec![1, 2, 3, 4, 5, 6, 7, 8], 4, 1), vec![0, 1, 0, 2, 9, 4, 9, 8]);
    assert_eq!(fft(vec![1, 2, 3, 4, 5, 6, 7, 8], 4, 2), vec![0, 0, 0, 2, 9, 4, 9, 8]);
    assert_eq!(fft(vec![1, 2, 3, 4, 5, 6, 7, 8], 4, 3), vec![0, 0, 0, 2, 9, 4, 9, 8]);
    assert_eq!(fft(vec![1, 2, 3, 4, 5, 6, 7, 8], 4, 4), vec![0, 0, 0, 0, 9, 4, 9, 8]);
    assert_eq!(fft(vec![1, 2, 3, 4, 5, 6, 7, 8], 4, 5), vec![0, 0, 0, 0, 0, 4, 9, 8]);
    assert_eq!(fft(vec![1, 2, 3, 4, 5, 6, 7, 8], 4, 6), vec![0, 0, 0, 0, 0, 0, 9, 8]);
    assert_eq!(fft(vec![1, 2, 3, 4, 5, 6, 7, 8], 4, 7), vec![0, 0, 0, 0, 0, 0, 0, 8]);

    let val = fft(vec![1, 2, 3, 4, 5, 6, 7, 8], 4, 3);
    assert_eq!(val[3..5].to_vec(), vec![2, 9]);
}

fn main() {
    let reader = io::stdin();
    let numbers: Vec<i32> =
        reader.lock()
              .lines().next().unwrap().unwrap() // iterator -> Option<io::Result<String>>
              .char_indices()                   // String -> iterator [character]
              .map(|s| s.1.to_digit(10).expect("could not turn into digit!") as i32)
              .collect();

    let mut part_2_numbers: Vec<i32> = vec![];
    for _i in 0..10000 {
        part_2_numbers.append(&mut numbers.clone());
    }
    let numbers = part_2_numbers;

    // The first seven digits of your initial input signal also
    // represent the message offset. The message offset is the location
    // of the eight-digit message in the final output list.
    // Specifically, the message offset indicates the number of digits
    // to skip before reading the eight-digit message.

    let message_offset : usize = (numbers[0] as usize) * 1000000 +
                                 (numbers[1] as usize) * 100000 +
                                 (numbers[2] as usize) * 10000 +
                                 (numbers[3] as usize) * 1000 +
                                 (numbers[4] as usize) * 100 +
                                 (numbers[5] as usize) * 10 +
                                 (numbers[6] as usize) * 1;

    let output: Vec<String> = numbers.iter().map(|s| s.to_string()).collect();
    //println!("Input is {} characters: {:?}", numbers.len(), output.join(""));

    let output: Vec<String> = fft(numbers, 100, message_offset).iter()
                                                               .map(|s| s.to_string())
                                                               .collect();

    println!("{:?}", output[message_offset..message_offset+8].join(""))
}
