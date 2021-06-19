use std::fs;
use std::io::{stdin, stdout, Write};
use std::process::exit;

#[derive(PartialEq, Copy, Clone, Debug)]
enum ParameterMode {
    // which causes the parameter to be interpreted as a position - if the parameter is 50, its
    // value is the value stored at address 50 in memory.
    PositionMode = 0,

    // a parameter is interpreted as a value - if the parameter is 50, its value is simply 50.
    ImmediateMode,
}

impl Default for ParameterMode {
    fn default() -> Self {
        ParameterMode::PositionMode
    }
}

fn get_parameter_modes_from_opcode(opcode: i32) -> [ParameterMode; 4] {
    // Parameter modes are stored in the same value as the instruction's opcode.
    //
    // Parameter modes are single digits, one per parameter, read right-to-left from the opcode:
    //
    // - the first parameter's mode is in the hundreds digit,
    // - the second parameter's mode is in the thousands digit,
    // - the third parameter's mode is in the ten-thousands digit,
    // - and so on.
    //
    // Any missing modes are 0 (== PositionMode)

    let mut parameter_mode: [ParameterMode; 4] = Default::default();

    let mut t = opcode;
    let mut i = 0;

    while t > 0 {
        if (t % 10) == 0 {
            parameter_mode[i] = ParameterMode::PositionMode;
        } else if (t % 10) == 1 {
            parameter_mode[i] = ParameterMode::ImmediateMode;
        }

        i += 1;
        t = t / 10;
    }

    return parameter_mode;
}

fn get_value(output: &Vec<i32>, iptr: usize, param_mode: ParameterMode) -> i32 {
    if param_mode == ParameterMode::PositionMode {
        return output[output[iptr] as usize];
    }

    if param_mode == ParameterMode::ImmediateMode {
        return output[iptr];
    }

    panic!();
}

fn intcode_program(input: Vec<i32>, ip: i32) -> Vec<i32> {
    let mut output: Vec<i32> = input.clone();
    let mut iptr = ip;

    // An Intcode program is a list of integers separated by commas.
    loop {
        //println!("{:?}", output);
        //println!("{}", iptr);

        // The opcode is a two-digit number based only on the ones and tens digit of the value
        let opcode = output[iptr as usize + 0] % 100;
        let param_modes = get_parameter_modes_from_opcode(output[iptr as usize + 0] / 100);

        // It is important to remember that the instruction pointer should increase by the number
        // of values in the instruction after the instruction finishes.
        let mut step = 0;

        // Parameters that an instruction writes to will never be in immediate mode.

        match opcode {
            // Opcode 1 adds together numbers read from two positions and stores the result in a
            // third position. The three integers immediately after the opcode tell you these three
            // positions - the first two indicate the positions from which you should read the
            // input values, and the third indicates the position at which the output should be
            // stored.
            1 => {
                let i1 = get_value(&output, iptr as usize + 1, param_modes[0]);
                let i2 = get_value(&output, iptr as usize + 2, param_modes[1]);
                let o1 = output[iptr as usize + 3];

                output[o1 as usize] = i1 + i2;
                step = 4;
            }

            // Opcode 2 works exactly like opcode 1, except it multiplies the two inputs instead of
            // adding them.
            2 => {
                let i1 = get_value(&output, iptr as usize + 1, param_modes[0]);
                let i2 = get_value(&output, iptr as usize + 2, param_modes[1]);
                let o1 = output[iptr as usize + 3];

                output[o1 as usize] = i1 * i2;
                step = 4;
            }

            // Opcode 3 takes a single integer as input and saves it to the position given by its
            // only parameter. For example, the instruction 3,50 would take an input value and
            // store it at address 50.
            3 => {
                let mut s = String::new();

                print!("input> ");
                let _ = stdout().flush();
                stdin()
                    .read_line(&mut s)
                    .expect("Did not enter a correct string");
                if let Some('\n') = s.chars().next_back() {
                    s.pop();
                }
                if let Some('\r') = s.chars().next_back() {
                    s.pop();
                }

                let i = s.parse::<i32>().unwrap();

                let o1 = output[iptr as usize + 1];
                output[o1 as usize] = i;

                step = 2;
            }

            // Opcode 4 outputs the value of its only parameter. For example, the instruction 4,50
            // would output the value at address 50.
            4 => {
                let i1 = get_value(&output, iptr as usize + 1, param_modes[0]);

                println!("output> {}", i1);

                step = 2;
            }

            // 99 means that the program is finished
            99 => {
                // halt!
                return output;
            }

            x => {
                panic!("unrecognized opcode {}", x);
            }
        }

        iptr += step;
    }
}

#[test]
fn test_intcode_program() {
    assert_eq!(
        intcode_program(vec![1, 0, 0, 0, 99], 0),
        vec![2, 0, 0, 0, 99]
    );
    assert_eq!(
        intcode_program(vec![2, 3, 0, 3, 99], 0),
        vec![2, 3, 0, 6, 99]
    );
    assert_eq!(
        intcode_program(vec![2, 4, 4, 5, 99, 0], 0),
        vec![2, 4, 4, 5, 99, 9801]
    );
    assert_eq!(
        intcode_program(vec![1, 1, 1, 4, 99, 5, 6, 0, 99], 0),
        vec![30, 1, 1, 4, 2, 5, 6, 0, 99]
    );

    // from day 5
    let _ = stdout().flush();
    assert_eq!(
        intcode_program(vec![1002, 4, 3, 4, 33], 0),
        vec![1002, 4, 3, 4, 99]
    );
}

fn main() {
    // echo program
    //println!("{:?}", intcode_program(vec![3, 0, 4, 0, 99], 0));

    let contents =
        fs::read_to_string("day5.input").expect("Something went wrong reading the file!");
    let numbers: Vec<i32> = contents
        .split(",")
        .map(|s| s.parse::<i32>().unwrap())
        .collect();

    let output = intcode_program(numbers, 0);
}
