use std::fs;
use std::io::{stdin, stdout, Write};
use std::sync::mpsc;
use std::thread;

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

struct IntcodeComputer {
    InputSender: mpsc::Sender<i32>,
    OutputReceiver: mpsc::Receiver<i32>,
    HaltReceiver: mpsc::Receiver<i32>,
    ThreadHandle: thread::JoinHandle<Vec<i32>>,
}

fn run_intcode_computer(name: String, program: Vec<i32>) -> IntcodeComputer {
    let (isend, irecv) = mpsc::channel();
    let (osend, orecv) = mpsc::channel();
    let (hsend, hrecv) = mpsc::channel();
    return IntcodeComputer {
        InputSender: isend,
        OutputReceiver: orecv,
        HaltReceiver: hrecv,
        ThreadHandle: thread::Builder::new()
            .name(name)
            .spawn(move || {
                return intcode_program(program, 0, irecv, osend, hsend);
            })
            .unwrap(),
    };
}

impl IntcodeComputer {
    fn send(&self, v: i32) {
        self.InputSender.send(v).expect("unable to send input!");
    }

    fn recv(&self) -> i32 {
        return self.OutputReceiver.recv().unwrap();
    }

    fn halted(&self) -> bool {
        // the computer has halted if there's a value here
        return !self.HaltReceiver.try_recv().is_err();
    }
}

fn intcode_program(
    input: Vec<i32>,
    ip: i32,
    computer_input: mpsc::Receiver<i32>,
    computer_output: mpsc::Sender<i32>,
    computer_halted: mpsc::Sender<i32>,
) -> Vec<i32> {
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
                /*
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
                */

                let i = computer_input.recv().expect("Could not receive!");

                let o1 = output[iptr as usize + 1];
                output[o1 as usize] = i;

                step = 2;
            }

            // Opcode 4 outputs the value of its only parameter. For example, the instruction 4,50
            // would output the value at address 50.
            4 => {
                let i1 = get_value(&output, iptr as usize + 1, param_modes[0]);

                // println!("output> {}", i1);
                computer_output.send(i1);

                step = 2;
            }

            // Opcode 5 is jump-if-true: if the first parameter is non-zero, it sets the
            // instruction pointer to the value from the second parameter. Otherwise, it does
            // nothing.
            5 => {
                let i1 = get_value(&output, iptr as usize + 1, param_modes[0]);
                let i2 = get_value(&output, iptr as usize + 2, param_modes[1]);

                if i1 != 0 {
                    iptr = i2;
                    step = 0;
                } else {
                    step = 3;
                }
            }

            // Opcode 6 is jump-if-false: if the first parameter is zero, it sets the instruction
            // pointer to the value from the second parameter. Otherwise, it does nothing.
            6 => {
                let i1 = get_value(&output, iptr as usize + 1, param_modes[0]);
                let i2 = get_value(&output, iptr as usize + 2, param_modes[1]);

                if i1 == 0 {
                    iptr = i2;
                    step = 0;
                } else {
                    step = 3;
                }
            }

            // Opcode 7 is less than: if the first parameter is less than the second parameter, it
            // stores 1 in the position given by the third parameter. Otherwise, it stores 0.
            7 => {
                let i1 = get_value(&output, iptr as usize + 1, param_modes[0]);
                let i2 = get_value(&output, iptr as usize + 2, param_modes[1]);
                let o1 = output[iptr as usize + 3];

                if i1 < i2 {
                    output[o1 as usize] = 1;
                } else {
                    output[o1 as usize] = 0;
                }

                step = 4;
            }

            // Opcode 8 is equals: if the first parameter is equal to the second parameter, it
            // stores 1 in the position given by the third parameter. Otherwise, it stores 0.
            8 => {
                let i1 = get_value(&output, iptr as usize + 1, param_modes[0]);
                let i2 = get_value(&output, iptr as usize + 2, param_modes[1]);
                let o1 = output[iptr as usize + 3];

                if i1 == i2 {
                    output[o1 as usize] = 1;
                } else {
                    output[o1 as usize] = 0;
                }

                step = 4;
            }

            // 99 means that the program is finished
            99 => {
                // halt!
                computer_halted.send(0);
                return output;
            }

            x => {
                panic!("unrecognized opcode {}", x);
            }
        }

        iptr += step;
    }
}

fn run_amplifier_chain(program: Vec<i32>, p1: i32, p2: i32, p3: i32, p4: i32, p5: i32) -> i32 {
    let ic0 = run_intcode_computer("ic0".to_string(), program.clone());
    let ic1 = run_intcode_computer("ic1".to_string(), program.clone());
    let ic2 = run_intcode_computer("ic2".to_string(), program.clone());
    let ic3 = run_intcode_computer("ic3".to_string(), program.clone());
    let ic4 = run_intcode_computer("ic4".to_string(), program.clone());

    ic0.send(p1);
    ic1.send(p2);
    ic2.send(p3);
    ic3.send(p4);
    ic4.send(p5);

    ic0.send(0);
    ic1.send(ic0.recv());
    ic2.send(ic1.recv());
    ic3.send(ic2.recv());
    ic4.send(ic3.recv());

    return ic4.recv();
}

#[test]
fn test_amplifier_programs() {
    assert_eq!(
        run_amplifier_chain(
            vec![3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,],
            4,
            3,
            2,
            1,
            0
        ),
        43210
    );

    assert_eq!(
        run_amplifier_chain(
            vec![
                3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4,
                23, 99, 0, 0
            ],
            0,
            1,
            2,
            3,
            4
        ),
        54321
    );

    assert_eq!(
        run_amplifier_chain(
            vec![
                3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33,
                1, 33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0
            ],
            1,
            0,
            4,
            3,
            2
        ),
        65210
    );
}

fn run_amplifier_chain_feedback(
    program: Vec<i32>,
    p1: i32,
    p2: i32,
    p3: i32,
    p4: i32,
    p5: i32,
) -> i32 {
    let ic0 = run_intcode_computer("ic0".to_string(), program.clone());
    let ic1 = run_intcode_computer("ic1".to_string(), program.clone());
    let ic2 = run_intcode_computer("ic2".to_string(), program.clone());
    let ic3 = run_intcode_computer("ic3".to_string(), program.clone());
    let ic4 = run_intcode_computer("ic4".to_string(), program.clone());

    ic0.send(p1);
    ic1.send(p2);
    ic2.send(p3);
    ic3.send(p4);
    ic4.send(p5);

    ic0.send(0);

    // connect amplifier E to amplifier A's input, run in feedback loop
    // computers will produce multiple values before halting
    // Each one should continue receiving and sending signals until it halts
    let mut last_output_from_last_amplifier: Option<i32> = None;

    loop {
        if ic1.halted() {
            return last_output_from_last_amplifier.unwrap();
        }
        ic1.send(ic0.recv());

        if ic2.halted() {
            return last_output_from_last_amplifier.unwrap();
        }
        ic2.send(ic1.recv());

        if ic3.halted() {
            return last_output_from_last_amplifier.unwrap();
        }
        ic3.send(ic2.recv());

        if ic4.halted() {
            return last_output_from_last_amplifier.unwrap();
        }
        ic4.send(ic3.recv());

        last_output_from_last_amplifier = Some(ic4.recv());

        if ic0.halted() {
            return last_output_from_last_amplifier.unwrap();
        }
        ic0.send(last_output_from_last_amplifier.unwrap());
    }

    panic!("end of function!");
}

#[test]
fn test_amplifier_with_feedback_programs() {
    assert_eq!(
        run_amplifier_chain_feedback(
            vec![
                3, 26, 1001, 26, -4, 26, 3, 27, 1002, 27, 2, 27, 1, 27, 26, 27, 4, 27, 1001, 28,
                -1, 28, 1005, 28, 6, 99, 0, 0, 5
            ],
            9,
            8,
            7,
            6,
            5
        ),
        139629729
    );

    assert_eq!(
        run_amplifier_chain_feedback(
            vec![
                3, 52, 1001, 52, -5, 52, 3, 53, 1, 52, 56, 54, 1007, 54, 5, 55, 1005, 55, 26, 1001,
                54, -5, 54, 1105, 1, 12, 1, 53, 54, 53, 1008, 54, 0, 55, 1001, 55, 1, 55, 2, 53,
                55, 53, 4, 53, 1001, 56, -1, 56, 1005, 56, 6, 99, 0, 0, 0, 0, 10
            ],
            9,
            7,
            8,
            5,
            6
        ),
        18216
    );
}

fn main() {
    // echo program
    //println!("{:?}", intcode_program(vec![3, 0, 4, 0, 99], 0));

    let contents =
        fs::read_to_string("day7.input").expect("Something went wrong reading the file!");
    let numbers: Vec<i32> = contents
        .split(",")
        .map(|s| s.parse::<i32>().unwrap())
        .collect();

    let mut max_output = 0;

    for p1 in 0..5 {
        for p2 in 0..5 {
            for p3 in 0..5 {
                for p4 in 0..5 {
                    for p5 in 0..5 {
                        // each phase setting is only used once
                        let mut bool_array: [bool; 5] = Default::default();
                        bool_array[p1] = true;
                        if bool_array[p2] {
                            continue;
                        }
                        bool_array[p2] = true;
                        if bool_array[p3] {
                            continue;
                        }
                        bool_array[p3] = true;
                        if bool_array[p4] {
                            continue;
                        }
                        bool_array[p4] = true;
                        if bool_array[p5] {
                            continue;
                        }
                        bool_array[p5] = true;
                        for i in 0..5 {
                            assert!(bool_array[i]);
                        }

                        let output = run_amplifier_chain(
                            numbers.clone(),
                            p1 as i32,
                            p2 as i32,
                            p3 as i32,
                            p4 as i32,
                            p5 as i32,
                        );
                        if output > max_output {
                            println!(
                                "update from {} to {} at {} {} {} {} {}",
                                max_output, output, p1, p2, p3, p4, p5,
                            );
                            max_output = output;
                        }
                    }
                }
            }
        }
    }

    println!("max output is {}", max_output);

    max_output = 0;

    for p1 in 5..10 {
        for p2 in 5..10 {
            for p3 in 5..10 {
                for p4 in 5..10 {
                    for p5 in 5..10 {
                        // each phase setting is only used once
                        let mut bool_array: [bool; 5] = Default::default();
                        bool_array[p1 - 5] = true;
                        if bool_array[p2 - 5] {
                            continue;
                        }
                        bool_array[p2 - 5] = true;
                        if bool_array[p3 - 5] {
                            continue;
                        }
                        bool_array[p3 - 5] = true;
                        if bool_array[p4 - 5] {
                            continue;
                        }
                        bool_array[p4 - 5] = true;
                        if bool_array[p5 - 5] {
                            continue;
                        }
                        bool_array[p5 - 5] = true;
                        for i in 0..5 {
                            assert!(bool_array[i]);
                        }

                        let output = run_amplifier_chain_feedback(
                            numbers.clone(),
                            p1 as i32,
                            p2 as i32,
                            p3 as i32,
                            p4 as i32,
                            p5 as i32,
                        );
                        if output > max_output {
                            println!(
                                "update from {} to {} at {} {} {} {} {}",
                                max_output, output, p1, p2, p3, p4, p5,
                            );
                            max_output = output;
                        }
                    }
                }
            }
        }
    }

    println!("max output is {}", max_output);
}
