use std::cmp;
use std::collections::HashMap;
use std::fs;
use std::ops::{Index, IndexMut};
use std::sync::mpsc;
use std::thread;

#[derive(PartialEq, Copy, Clone, Debug)]
enum ParameterMode {
    // which causes the parameter to be interpreted as a position - if the parameter is 50, its
    // value is the value stored at address 50 in memory.
    PositionMode = 0,

    // a parameter is interpreted as a value - if the parameter is 50, its value is simply 50.
    ImmediateMode,

    // the parameter is interpreted as a position like PositionMode
    // except relative mode parameters don't count from address 0. Instead, they count from a value called the relative base.
    // The address a relative mode parameter refers to is itself plus the current relative base.
    RelativeMode,
}

impl Default for ParameterMode {
    fn default() -> Self {
        ParameterMode::PositionMode
    }
}

fn get_parameter_modes_from_opcode(opcode: i64) -> [ParameterMode; 4] {
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
        } else if (t % 10) == 2 {
            parameter_mode[i] = ParameterMode::RelativeMode;
        }

        i += 1;
        t = t / 10;
    }

    return parameter_mode;
}

struct Memory {
    memory: HashMap<i64, i64>,
}

impl Index<i64> for Memory {
    type Output = i64;

    fn index(&self, index: i64) -> &Self::Output {
        if index < 0 {
            panic!("index {} < 0!", index);
        }
        if self.memory.contains_key(&index) {
            return self.memory.get(&index).unwrap();
        } else {
            return &0;
        }
    }
}

impl IndexMut<i64> for Memory {
    fn index_mut(&mut self, index: i64) -> &mut Self::Output {
        if index < 0 {
            panic!("index < 0!");
        }
        return &mut *self.memory.entry(index).or_insert(0);
    }
}

#[test]
fn test_memory() {
    let mut memory = Memory {
        memory: Default::default(),
    };

    assert_eq!(memory[0], 0);
    assert_eq!(memory[1000], 0);

    memory[1000] = 123;

    assert_eq!(memory[0], 0);
    assert_eq!(memory[1000], 123);

    // only 1000 written in
    assert_eq!(memory.memory.keys().len(), 1);

    memory[0] = 23874612876341;

    assert_eq!(memory[0], 23874612876341);
    assert_eq!(memory[1000], 123);

    assert_eq!(memory.memory.keys().len(), 2);

    memory[985237621] = 72346571;

    assert_eq!(memory[0], 23874612876341);
    assert_eq!(memory[1000], 123);
    assert_eq!(memory[985237621], 72346571);

    assert_eq!(memory.memory.keys().len(), 3);
}

fn get_value(output: &Memory, iptr: i64, param_mode: ParameterMode, rbase: i64) -> i64 {
    let param = output[iptr];

    if param_mode == ParameterMode::PositionMode {
        //println!(
        //    "iptr {} param {} position mode == {}",
        //    iptr, param, output[param]
        //);
        return output[param];
    }

    if param_mode == ParameterMode::ImmediateMode {
        //println!("iptr {} param {} immediate mode == {}", iptr, param, param);
        return param;
    }

    if param_mode == ParameterMode::RelativeMode {
        //println!(
        //    "iptr {} rbase {} param {} relative mode == {}",
        //    iptr,
        //    rbase,
        //    param,
        //    output[param + rbase]
        //);
        return output[param + rbase];
    }

    panic!();
}

fn set_value(output: &mut Memory, iptr: i64, param_mode: ParameterMode, rbase: i64, v: i64) {
    let param = output[iptr];

    if param_mode == ParameterMode::PositionMode {
        //println!("set iptr {} param {} position mode == {}", iptr, param, v);
        output[param] = v;
        return;
    }

    if param_mode == ParameterMode::RelativeMode {
        //println!(
        //    "set iptr {} rbase {} param {} relative mode == {}",
        //    iptr, rbase, param, v,
        //);
        output[param + rbase] = v;
        return;
    }

    panic!();
}

#[test]
fn test_relative_mode() {
    // Parameters in mode 2, relative mode, behave very similarly to parameters in position mode:
    // the parameter is interpreted as a position. Like position mode, parameters in relative mode
    // can be read from or written to.
    //
    // The address a relative mode parameter refers to is itself plus the current relative base.
    //
    // When the relative base is 0, relative mode parameters and position mode parameters with the
    // same value refer to the same address. For example, given a relative base of 50, a relative
    // mode parameter of -7 refers to memory address 50 + -7 = 43.

    let mut memory = Memory {
        memory: Default::default(),
    };

    memory[43] = 873645927183645;

    let rbase = 50;
    let parameter = -7;

    memory[0] = parameter;

    assert_eq!(
        873645927183645,
        get_value(&memory, 0, ParameterMode::RelativeMode, rbase)
    );
}

struct IntcodeComputer {
    InputSender: mpsc::Sender<i64>,
    OutputReceiver: mpsc::Receiver<i64>,
    HaltReceiver: mpsc::Receiver<i64>,
    ThreadHandle: thread::JoinHandle<Memory>,
}

fn run_intcode_computer(name: String, program: Vec<i64>) -> IntcodeComputer {
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
                let memory_output = intcode_program(program, 0, irecv, osend, hsend);
                /*
                loop {
                    // wait until all output is drained?
                    if orecv.try_recv().is_err() {
                        break;
                    }
                }
                */
                return memory_output;
            })
            .unwrap(),
    };
}

impl IntcodeComputer {
    fn send(&self, v: i64) {
        self.InputSender.send(v).expect("unable to send input!");
    }

    fn recv(&self) -> i64 {
        return self.OutputReceiver.recv().unwrap();
    }

    fn recv2(&self) -> Result<i64, mpsc::RecvError> {
        return self.OutputReceiver.recv();
    }

    fn try_recv(&self) -> Option<i64> {
        let result: Result<i64, mpsc::TryRecvError> = self.OutputReceiver.try_recv();

        if result.is_err() {
            return None;
        } else {
            return Some(result.unwrap());
        }
    }

    fn halted(&self) -> bool {
        // the computer has halted if there's a value here
        return !self.HaltReceiver.try_recv().is_err();
    }
}

fn intcode_program(
    input: Vec<i64>,
    ip: i64,
    computer_input: mpsc::Receiver<i64>,
    computer_output: mpsc::Sender<i64>,
    computer_halted: mpsc::Sender<i64>,
) -> Memory {
    let mut iptr = ip;
    let mut rbase: i64 = 0;
    let mut memory: Memory = Memory {
        memory: Default::default(),
    };

    for i in 0..input.len() {
        memory[i as i64] = input[i];
        //print!("{}:{},", i, memory[i as i64]);
    }
    //println!("");

    // An Intcode program is a list of integers separated by commas.
    loop {
        /*
        // store previous state
        println!("--------");
        let old_memory: Memory = Memory {
            memory: memory.memory.clone(),
        };
        let old_rbase = rbase;
        */

        // The opcode is a two-digit number based only on the ones and tens digit of the value
        let opcode = memory[iptr] % 100;
        let param_modes = get_parameter_modes_from_opcode(memory[iptr] / 100);

        // It is important to remember that the instruction pointer should increase by the number
        // of values in the instruction after the instruction finishes.
        let mut step = 0;
        let mut op: String = "".to_string();

        // Parameters that an instruction writes to will never be in immediate mode.

        match opcode {
            // Opcode 1 adds together numbers read from two positions and stores the result in a
            // third position. The three integers immediately after the opcode tell you these three
            // positions - the first two indicate the positions from which you should read the
            // input values, and the third indicates the position at which the output should be
            // stored.
            1 => {
                let i1 = get_value(&memory, iptr + 1, param_modes[0], rbase);
                let i2 = get_value(&memory, iptr + 2, param_modes[1], rbase);
                set_value(&mut memory, iptr + 3, param_modes[2], rbase, i1 + i2);

                step = 4;
                op = "ADD".to_string();
            }

            // Opcode 2 works exactly like opcode 1, except it multiplies the two inputs instead of
            // adding them.
            2 => {
                let i1 = get_value(&memory, iptr + 1, param_modes[0], rbase);
                let i2 = get_value(&memory, iptr + 2, param_modes[1], rbase);
                set_value(&mut memory, iptr + 3, param_modes[2], rbase, i1 * i2);

                step = 4;
                op = "MUL".to_string();
            }

            // Opcode 3 takes a single integer as input and saves it to the position given by its
            // only parameter. For example, the instruction 3,50 would take an input value and
            // store it at address 50.
            3 => {
                let i = computer_input.recv().expect("Could not receive!");

                set_value(&mut memory, iptr + 1, param_modes[0], rbase, i);

                step = 2;
                op = "IN".to_string();
            }

            // Opcode 4 outputs the value of its only parameter. For example, the instruction 4,50
            // would output the value at address 50.
            4 => {
                let i1 = get_value(&memory, iptr + 1, param_modes[0], rbase);

                computer_output.send(i1);

                step = 2;
                op = "OUT".to_string();
            }

            // Opcode 5 is jump-if-true: if the first parameter is non-zero, it sets the
            // instruction pointer to the value from the second parameter. Otherwise, it does
            // nothing.
            5 => {
                let i1 = get_value(&memory, iptr + 1, param_modes[0], rbase);
                let i2 = get_value(&memory, iptr + 2, param_modes[1], rbase);

                if i1 != 0 {
                    iptr = i2;
                    step = 0;
                } else {
                    step = 3;
                }
                op = "JT".to_string();
            }

            // Opcode 6 is jump-if-false: if the first parameter is zero, it sets the instruction
            // pointer to the value from the second parameter. Otherwise, it does nothing.
            6 => {
                let i1 = get_value(&memory, iptr + 1, param_modes[0], rbase);
                let i2 = get_value(&memory, iptr + 2, param_modes[1], rbase);

                if i1 == 0 {
                    iptr = i2;
                    step = 0;
                } else {
                    step = 3;
                }
                op = "JF".to_string();
            }

            // Opcode 7 is less than: if the first parameter is less than the second parameter, it
            // stores 1 in the position given by the third parameter. Otherwise, it stores 0.
            7 => {
                let i1 = get_value(&memory, iptr + 1, param_modes[0], rbase);
                let i2 = get_value(&memory, iptr + 2, param_modes[1], rbase);

                if i1 < i2 {
                    set_value(&mut memory, iptr + 3, param_modes[2], rbase, 1);
                } else {
                    set_value(&mut memory, iptr + 3, param_modes[2], rbase, 0);
                }

                step = 4;
                op = "LT".to_string();
            }

            // Opcode 8 is equals: if the first parameter is equal to the second parameter, it
            // stores 1 in the position given by the third parameter. Otherwise, it stores 0.
            8 => {
                let i1 = get_value(&memory, iptr + 1, param_modes[0], rbase);
                let i2 = get_value(&memory, iptr + 2, param_modes[1], rbase);

                if i1 == i2 {
                    set_value(&mut memory, iptr + 3, param_modes[2], rbase, 1);
                } else {
                    set_value(&mut memory, iptr + 3, param_modes[2], rbase, 0);
                }

                step = 4;
                op = "EQ".to_string();
            }

            // Opcode 9 adjusts the relative base by the value of its only parameter. The relative
            // base increases (or decreases, if the value is negative) by the value of the
            // parameter.
            9 => {
                let i1 = get_value(&memory, iptr + 1, param_modes[0], rbase);
                rbase = rbase + i1;

                step = 2;
                op = "RBASE".to_string();
            }

            // 99 means that the program is finished
            99 => {
                // halt!
                computer_halted.send(0);
                return memory;
            }

            x => {
                panic!("unrecognized opcode {}", x);
            }
        }

        /*
        // print modified state
        print!("{} executed {}", iptr, op);
        for i in 0..step {
            print!(" {}", memory[iptr + i]);
        }
        println!("");

        for (k, _) in &memory.memory {
            if old_memory.memory.contains_key(k) {
                if old_memory[*k] != memory[*k] {
                    println!("{}: {} -> {}", *k, old_memory[*k], memory[*k]);
                }
            } else {
                println!("{}: {}", *k, memory[*k]);
            }
        }

        if old_rbase != rbase {
            println!("rbase {} -> {}", old_rbase, rbase);
        }
        */

        iptr += step;
    }
}

#[test]
fn test_quine() {
    let program = vec![
        109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
    ];
    let ic = run_intcode_computer("ic".to_string(), program.clone());
    let memory: Memory = ic.ThreadHandle.join().unwrap();

    for i in 0..program.len() {
        assert_eq!(program[i], memory[i as i64]);
    }
}

#[test]
fn test_16_digit() {
    let ic = run_intcode_computer(
        "ic".to_string(),
        vec![1102, 34915192, 34915192, 7, 4, 7, 99, 0],
    );
    assert_eq!(1219070632396864, ic.recv());
}

#[test]
fn test_output_large_middle() {
    let ic = run_intcode_computer("ic".to_string(), vec![104, 1125899906842624, 99]);
    assert_eq!(1125899906842624, ic.recv());
}

fn run_amplifier_chain(program: Vec<i64>, p1: i64, p2: i64, p3: i64, p4: i64, p5: i64) -> i64 {
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
    program: Vec<i64>,
    p1: i64,
    p2: i64,
    p3: i64,
    p4: i64,
    p5: i64,
) -> i64 {
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
    let mut last_output_from_last_amplifier: Option<i64> = None;

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

fn run_intcode_computer_and_print(program: Vec<i64>, input: i64) {
    let ic = run_intcode_computer("ic".to_string(), program.clone());

    ic.send(input);

    let mut outputs: Vec<i64> = Vec::new();

    loop {
        match ic.try_recv() {
            Some(v) => {
                outputs.push(v);
            }
            None => {
                // pass
            }
        }

        if ic.halted() {
            // drain outputs
            loop {
                let opt = ic.try_recv();
                match opt {
                    Some(v) => {
                        outputs.push(v);
                    }
                    None => {
                        break;
                    }
                }
            }
            break;
        }
    }

    for i in outputs {
        println!("{}", i);
    }
}

#[test]
fn test_day_5() {
    let contents =
        fs::read_to_string("day5.input").expect("Something went wrong reading the file!");
    let program: Vec<i64> = contents
        .split(",")
        .map(|s| s.parse::<i64>().unwrap())
        .collect();

    let ic = run_intcode_computer("ic".to_string(), program.clone());

    ic.send(1);

    let mut outputs: Vec<i64> = Vec::new();

    loop {
        match ic.try_recv() {
            Some(v) => {
                outputs.push(v);
            }
            None => {
                // pass
            }
        }

        if ic.halted() {
            // drain outputs
            loop {
                let opt = ic.try_recv();
                match opt {
                    Some(v) => {
                        outputs.push(v);
                    }
                    None => {
                        break;
                    }
                }
            }
            break;
        }
    }

    for i in 0..(outputs.len() - 1) {
        assert_eq!(0, outputs[i]);
    }

    assert_eq!(7692125, outputs[outputs.len() - 1]);
}

enum Direction {
    North,
    East,
    South,
    West,
}

struct Grid {
    panels: HashMap<i32, HashMap<i32, i32>>,
}

impl Grid {
    fn get(&self, x: i32, y: i32) -> i32 {
        // take care not to create entry if just getting
        if self.panels.contains_key(&x) {
            if self.panels.get(&x).unwrap().contains_key(&y) {
                return *self.panels.get(&x).unwrap().get(&y).unwrap();
            }
        }
        return 0;
    }

    fn set(&mut self, x: i32, y: i32, v: i32) {
        // only create entry if writing
        *self.panels.entry(x).or_default().entry(y).or_default() = v;
    }

    fn num_entries(&self) -> usize {
        let mut total_keys = 0;

        for (_, v) in &self.panels {
            for (k, vv) in v {
                total_keys += 1;
            }
        }

        return total_keys;
    }
}

#[test]
fn test_grid() {
    let mut grid = Grid {
        panels: Default::default(),
    };

    assert_eq!(grid.num_entries(), 0);

    grid.set(0, 0, 1);
    assert_eq!(grid.num_entries(), 1);

    grid.set(-1, 0, 0);
    assert_eq!(grid.num_entries(), 2);

    grid.set(-1, -1, 1);
    assert_eq!(grid.num_entries(), 3);

    grid.set(0, -1, 1);
    assert_eq!(grid.num_entries(), 4);

    grid.set(0, 0, 0);
    assert_eq!(grid.num_entries(), 4);

    grid.set(1, 0, 1);
    assert_eq!(grid.num_entries(), 5);

    grid.set(1, 1, 1);
    assert_eq!(grid.num_entries(), 6);
}

fn main() {
    let contents =
        fs::read_to_string("day11.input").expect("Something went wrong reading the file!");
    let program: Vec<i64> = contents
        .split(",")
        .map(|s| s.parse::<i64>().unwrap())
        .collect();

    // power up the emergency hull painting robot!
    let ic = run_intcode_computer("ic".to_string(), program.clone());

    // 0 == black
    // 1 == white
    let mut x = 0;
    let mut y = 0;
    let mut d = Direction::North;

    let mut panels: Grid = Grid {
        panels: Default::default(),
    };

    // The robot needs to be able to move around on the grid of square panels on the side of your
    // ship, detect the color of its current panel, and paint its current panel black or white.

    // The program uses input instructions to access the robot's camera: provide 0 if the robot is
    // over a black panel or 1 if the robot is over a white panel.

    // Then, the program will output two values:
    //
    // First, it will output a value indicating the color to paint the panel the robot is over: 0
    // means to paint the panel black, and 1 means to paint the panel white.
    //
    // Second, it will output a value indicating the direction the robot should turn: 0 means it
    // should turn left 90 degrees, and 1 means it should turn right 90 degrees.
    //
    // After the robot turns, it should always move forward exactly one panel. The robot starts
    // facing up.

    // part 2 - start on white
    panels.set(x, y, 1);

    loop {
        let robot_over_color = panels.get(x, y);

        if ic.halted() {
            break;
        }

        ic.send(robot_over_color as i64);

        let paint_color = ic.recv2();
        if paint_color.is_err() {
            break;
        }

        let turn_direction = ic.recv2();
        if turn_direction.is_err() {
            break;
        }

        panels.set(x, y, paint_color.unwrap() as i32);

        if turn_direction.unwrap() == 0 {
            // turn left
            match d {
                Direction::North => {
                    d = Direction::West;
                }
                Direction::East => {
                    d = Direction::North;
                }
                Direction::South => {
                    d = Direction::East;
                }
                Direction::West => {
                    d = Direction::South;
                }
            }
        } else if turn_direction.unwrap() == 1 {
            // turn right
            match d {
                Direction::North => {
                    d = Direction::East;
                }
                Direction::East => {
                    d = Direction::South;
                }
                Direction::South => {
                    d = Direction::West;
                }
                Direction::West => {
                    d = Direction::North;
                }
            }
        }

        // go in that direction
        match d {
            Direction::North => {
                y = y - 1;
            }
            Direction::East => {
                x = x + 1;
            }
            Direction::South => {
                y = y + 1;
            }
            Direction::West => {
                x = x - 1;
            }
        }
    }

    println!("{:?}", panels.panels);
    println!("Panels painted at least once: {}", panels.num_entries());

    let mut min_x: Option<i32> = None;
    let mut min_y: Option<i32> = None;

    let mut max_x = 0;
    let mut max_y = 0;

    for (xx, v) in &panels.panels {
        for (yy, vv) in v {
            max_x = cmp::max(max_x, *xx);
            max_y = cmp::max(max_y, *yy);

            match min_x {
                Some(v) => {
                    min_x = Some(cmp::min(v, *xx));
                }
                None => {
                    min_x = Some(*xx);
                }
            }
            match min_y {
                Some(v) => {
                    min_y = Some(cmp::min(v, *yy));
                }
                None => {
                    min_y = Some(*yy);
                }
            }
        }
    }

    for y in min_y.unwrap()..(max_y + 1) {
        for x in min_x.unwrap()..(max_x + 1) {
            let c = panels.get(x, y);
            if c == 1 {
                print!("#");
            } else {
                print!(".");
            }
        }
        println!("");
    }
}
