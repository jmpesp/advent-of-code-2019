use std::io::{self, BufRead};
use std::process::exit;

// An Intcode program is a list of integers separated by commas.

// start by looking at the first integer (called position 0).

// you will find an opcode - either 1, 2, or 99.

// 99 means that the program is finished

// Opcode 1 adds together numbers read from two positions and stores
// the result in a third position. The three integers immediately after
// the opcode tell you these three positions - the first two indicate
// the positions from which you should read the input values, and the
// third indicates the position at which the output should be stored.

// Opcode 2 works exactly like opcode 1, except it multiplies the two
// inputs instead of adding them.

// Move to the next one by stepping forward 4 positions.

fn intcode_program(input: Vec<i32>, ip: i32) -> Vec<i32> {
    let mut output: Vec<i32> = input.clone();
    let mut iptr = ip;

    loop {
        let opcode = output[iptr as usize + 0];

        match opcode {
            1 => {
                let i1 = output[output[iptr as usize + 1] as usize];
                let i2 = output[output[iptr as usize + 2] as usize];
                let o1 = output[iptr as usize + 3];

                output[o1 as usize] = i1 + i2;
            },
            2 => {
                let i1 = output[output[iptr as usize + 1] as usize];
                let i2 = output[output[iptr as usize + 2] as usize];
                let o1 = output[iptr as usize + 3];

                output[o1 as usize] = i1 * i2;
            },
            99 => {
                // halt!
                return output
            },
            x => {
                println!("unrecognized opcode {}", x);
                exit(1);
            },
        }

        iptr += 4;
    }
}

#[test]
fn test_intcode_program() {
    assert_eq!(intcode_program(vec![1,0,0,0,99], 0), vec![2,0,0,0,99]);
    assert_eq!(intcode_program(vec![2,3,0,3,99], 0), vec![2,3,0,6,99]);
    assert_eq!(intcode_program(vec![2,4,4,5,99,0], 0), vec![2,4,4,5,99,9801]);
    assert_eq!(intcode_program(vec![1,1,1,4,99,5,6,0,99], 0), vec![30,1,1,4,2,5,6,0,99]);
}

fn main() {
    let reader = io::stdin();
    let mut numbers: Vec<i32> =
        reader.lock()
              .lines().next().unwrap().unwrap()
              .split(",")
              .map(|s| s.parse::<i32>().unwrap())
              .collect();

    // Once you have a working computer, the first step is to restore
    // the gravity assist program (your puzzle input) to the "1202
    // program alarm" state it had just before the last computer caught
    // fire. To do this, before running the program, replace position 1
    // with the value 12 and replace position 2 with the value 2. What
    // value is left at position 0 after the program halts?

    numbers[1] = 12;
    numbers[2] = 2;

    let output = intcode_program(numbers, 0);
    println!("{}", output[0]);
}
