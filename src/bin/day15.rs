use std::cmp;
use std::collections::HashMap;
use std::fs;
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};
use std::sync::mpsc;
use std::thread;

use petgraph::algo::{all_simple_paths, dijkstra};
use petgraph::graph::{DefaultIx, NodeIndex};
use petgraph::graph::{Graph, UnGraph};

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
    WaitReceiver: mpsc::Receiver<i64>,
    WaitingOnInput: bool,
    ThreadHandle: thread::JoinHandle<Memory>,
}

fn run_intcode_computer(name: String, program: Vec<i64>) -> IntcodeComputer {
    let (isend, irecv) = mpsc::channel();
    let (osend, orecv) = mpsc::channel();
    let (hsend, hrecv) = mpsc::channel();
    let (wsend, wrecv) = mpsc::channel();
    return IntcodeComputer {
        InputSender: isend,
        OutputReceiver: orecv,
        HaltReceiver: hrecv,
        WaitReceiver: wrecv,
        WaitingOnInput: false,
        ThreadHandle: thread::Builder::new()
            .name(name)
            .spawn(move || {
                let memory_output = intcode_program(program, 0, irecv, osend, hsend, wsend);
                return memory_output;
            })
            .unwrap(),
    };
}

impl IntcodeComputer {
    fn send(&mut self, v: i64) {
        self.WaitingOnInput = false;
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

    fn waiting_on_input(&mut self) -> bool {
        if self.WaitingOnInput {
            return true;
        }

        let result: Result<i64, mpsc::TryRecvError> = self.WaitReceiver.try_recv();

        if !result.is_err() {
            println!("{:?} wants input!", result);
            self.WaitingOnInput = true;
        }
        return self.WaitingOnInput;
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
    wait_output: mpsc::Sender<i64>,
) -> Memory {
    let mut iptr = ip;
    let mut rbase: i64 = 0;
    let mut memory: Memory = Memory {
        memory: Default::default(),
    };

    for i in 0..input.len() {
        memory[i as i64] = input[i];
        //println!("{}:{}?", i, input[i]);
        //println!("{}:{},", i, memory[i as i64]);
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

        //println!("executing {}", opcode);

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
                wait_output.send(0);
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
    let mut ic = run_intcode_computer("ic".to_string(), program.clone());
    let memory: Memory = ic.ThreadHandle.join().unwrap();

    for i in 0..program.len() {
        assert_eq!(program[i], memory[i as i64]);
    }
}

#[test]
fn test_16_digit() {
    let mut ic = run_intcode_computer(
        "ic".to_string(),
        vec![1102, 34915192, 34915192, 7, 4, 7, 99, 0],
    );
    assert_eq!(1219070632396864, ic.recv());
}

#[test]
fn test_output_large_middle() {
    let mut ic = run_intcode_computer("ic".to_string(), vec![104, 1125899906842624, 99]);
    assert_eq!(1125899906842624, ic.recv());
}

fn run_amplifier_chain(program: Vec<i64>, p1: i64, p2: i64, p3: i64, p4: i64, p5: i64) -> i64 {
    let mut ic0 = run_intcode_computer("ic0".to_string(), program.clone());
    let mut ic1 = run_intcode_computer("ic1".to_string(), program.clone());
    let mut ic2 = run_intcode_computer("ic2".to_string(), program.clone());
    let mut ic3 = run_intcode_computer("ic3".to_string(), program.clone());
    let mut ic4 = run_intcode_computer("ic4".to_string(), program.clone());

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
    let mut ic0 = run_intcode_computer("ic0".to_string(), program.clone());
    let mut ic1 = run_intcode_computer("ic1".to_string(), program.clone());
    let mut ic2 = run_intcode_computer("ic2".to_string(), program.clone());
    let mut ic3 = run_intcode_computer("ic3".to_string(), program.clone());
    let mut ic4 = run_intcode_computer("ic4".to_string(), program.clone());

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
    let mut ic = run_intcode_computer("ic".to_string(), program.clone());

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

    let mut ic = run_intcode_computer("ic".to_string(), program.clone());

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

#[derive(Copy, Clone)]
enum GridItem {
    NotSure = 0,
    Wall,
    Empty,
    Oxygen,
}

struct Grid {
    panels: HashMap<i32, HashMap<i32, GridItem>>,
}

impl Grid {
    fn get(&self, x: i32, y: i32) -> GridItem {
        // take care not to create entry if just getting
        if self.panels.contains_key(&x) {
            if self.panels.get(&x).unwrap().contains_key(&y) {
                return *self.panels.get(&x).unwrap().get(&y).unwrap();
            }
        }
        return GridItem::NotSure;
    }

    fn set(&mut self, x: i32, y: i32, v: GridItem) {
        // only create entry if writing
        *self
            .panels
            .entry(x)
            .or_default()
            .entry(y)
            .or_insert(GridItem::NotSure) = v;
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

fn display(panels: &Grid, dx: i32, dy: i32) {
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

    match min_x {
        None => {
            return;
        }
        Some(_) => {
            // pass
        }
    }
    match min_y {
        None => {
            return;
        }
        Some(_) => {
            // pass
        }
    }

    for y in min_y.unwrap()..(max_y + 1) {
        for x in min_x.unwrap()..(max_x + 1) {
            if x == dx && y == dy {
                print!("D");
            } else {
                let c = panels.get(x, y);
                match c {
                    GridItem::NotSure => {
                        // not sure
                        print!(" ");
                    }
                    GridItem::Wall => {
                        // wall
                        print!("#");
                    }
                    GridItem::Empty => {
                        // empty
                        print!(".");
                    }
                    GridItem::Oxygen => {
                        // oxygen!
                        print!("O");
                    }
                    _ => {
                        panic!("bad value seen!");
                    }
                }
            }
        }
        println!("");
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum NodeStatus {
    NotSure,
    Empty,
    Wall,
    Oxygen,
}

#[derive(Debug, Clone)]
struct Node {
    x: i32,
    y: i32,
    status: NodeStatus,
    index: NodeIndex<DefaultIx>,
}

struct Map {
    graph: UnGraph<Node, usize>,
    index: usize,
}

impl Map {
    fn new() -> Map {
        return Map {
            graph: Graph::new_undirected(),
            index: 0,
        };
    }

    fn add_node(&mut self, x: i32, y: i32, status: NodeStatus) {
        self.graph.add_node(Node {
            x: x,
            y: y,
            status: status,
            index: NodeIndex::new(self.index),
        });
        self.index += 1;
    }

    fn add_edge(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        self.graph.add_edge(
            self.node_index(x1, y1).unwrap(),
            self.node_index(x2, y2).unwrap(),
            1,
        );
    }

    fn node_index(&self, x: i32, y: i32) -> Option<NodeIndex<DefaultIx>> {
        for node_index in self.graph.node_indices() {
            let node = self.graph.node_weight(node_index).unwrap();
            if node.x == x && node.y == y {
                return Some(node.index);
            }
        }
        return None;
    }

    fn find_oxygen_node(&self) -> NodeIndex<DefaultIx> {
        for node_index in self.graph.node_indices() {
            let node = self.graph.node_weight(node_index).unwrap();
            if node.status == NodeStatus::Oxygen {
                return node_index;
            }
        }
        panic!("bad!");
    }

    fn node_exists(&self, x: i32, y: i32) -> bool {
        match self.node_index(x, y) {
            Some(_) => {
                return true;
            }
            None => {
                return false;
            }
        }
    }

    fn get_node_by_index(&self, i: NodeIndex<DefaultIx>) -> &Node {
        return self.graph.node_weight(i).unwrap();
    }

    fn get_node_by_index_mut(&mut self, i: NodeIndex<DefaultIx>) -> &mut Node {
        return self.graph.node_weight_mut(i).unwrap();
    }

    fn update_node(&mut self, x: i32, y: i32, status: NodeStatus) {
        let index = self.node_index(x, y).unwrap();
        self.get_node_by_index_mut(index).status = status;
    }

    fn remove_edge(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let e = self
            .graph
            .find_edge(
                self.node_index(x1, y1).unwrap(),
                self.node_index(x2, y2).unwrap(),
            )
            .unwrap();
        self.graph.remove_edge(e);
    }

    fn return_shortest_path_length(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> usize {
        let from_node = self.node_index(x1, y1).unwrap();
        let to_node = self.node_index(x2, y2).unwrap();

        let dijkstra_result: HashMap<NodeIndex<DefaultIx>, usize> =
            dijkstra(&self.graph, from_node, Some(to_node), |e| *e.weight());
        let shortest_path_length = *dijkstra_result.get(&to_node).unwrap();

        return shortest_path_length;
    }

    fn return_shortest_path(
        &self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    ) -> Vec<NodeIndex<DefaultIx>> {
        let shortest_path_length = self.return_shortest_path_length(x1, y1, x2, y2);

        let from_node = self.node_index(x1, y1).unwrap();
        let to_node = self.node_index(x2, y2).unwrap();

        let mut result: Vec<NodeIndex<DefaultIx>> = Vec::new();

        for item in all_simple_paths::<Vec<_>, _>(
            &self.graph,
            from_node,
            to_node,
            0,
            Some(shortest_path_length),
        )
        .next()
        .unwrap()
        {
            if item == from_node {
                continue;
            }
            result.push(item);
        }

        return result;
    }
}

fn main() {
    let contents =
        fs::read_to_string("day15.input").expect("Something went wrong reading the file!");
    let mut program: Vec<i64> = contents
        .split(",")
        .map(|s| s.parse::<i64>().unwrap())
        .collect();

    let mut panels: Grid = Grid {
        panels: Default::default(),
    };

    let mut ic = run_intcode_computer("ic".to_string(), program.clone());

    // drone coords
    let mut dx = 0;
    let mut dy = 0;

    // The remote control program executes the following steps in a loop forever:
    //
    // - Accept a movement command via an input instruction.
    // - Send the movement command to the repair droid.
    // - Wait for the repair droid to finish the movement operation.
    // - Report on the status of the repair droid via an output instruction.

    // Only four movement commands are understood: north (1), south (2), west (3), and east (4)

    // The repair droid can reply with any of the following status codes:
    //
    // 0: The repair droid hit a wall. Its position has not changed.
    // 1: The repair droid has moved one step in the requested direction.
    // 2: The repair droid has moved one step in the requested direction; its new position is the location of the oxygen system.

    // it's a backtracking search!

    // construct a stack for DFS
    let mut search_stack: Vec<(i32, i32)> = Vec::new();

    // graph
    let mut map: Map = Map::new();

    // drone is at (0, 0) - assume that (0,0) is empty
    map.add_node(dx, dy, NodeStatus::Empty);
    panels.set(dx, dy, GridItem::Empty);

    // search in 4 cardinal directions
    search_stack.push((dx - 1, dy));
    map.add_node(dx - 1, dy, NodeStatus::NotSure);
    map.add_edge(dx, dy, dx - 1, dy);

    search_stack.push((dx + 1, dy));
    map.add_node(dx + 1, dy, NodeStatus::NotSure);
    map.add_edge(dx, dy, dx + 1, dy);

    search_stack.push((dx, dy - 1));
    map.add_node(dx, dy - 1, NodeStatus::NotSure);
    map.add_edge(dx, dy, dx, dy - 1);

    search_stack.push((dx, dy + 1));
    map.add_node(dx, dy + 1, NodeStatus::NotSure);
    map.add_edge(dx, dy, dx, dy + 1);

    loop {
        if ic.halted() {
            println!("saw halt");
            break;
        }

        println!("----------------");

        display(&panels, dx, dy);

        // pop off search stack
        println!("{:?}", search_stack);
        let search_item_option = search_stack.pop();
        let search_item: (i32, i32);
        match search_item_option {
            Some(v) => {
                search_item = v;
            }
            None => {
                println!("exhausted search stack");
                break;
            }
        }

        println!("{} {}, popped {:?}", dx, dy, search_item);

        // move there
        let target_node = map.node_index(search_item.0, search_item.1);
        let mut status: i32 = 0;
        let mut direction: i64 = 0;

        // track previous location
        let mut pdx = dx;
        let mut pdy = dy;

        println!("{:?}", map.graph);

        match target_node {
            Some(_) => {
                println!("movement to point!");
                let movement_path = map.return_shortest_path(dx, dy, search_item.0, search_item.1);
                let moves: i32 = 0;

                for movement in movement_path {
                    let movement_node = map.get_node_by_index(movement);
                    println!("{} {} movement node is {:?}", dx, dy, movement_node);

                    pdx = dx;
                    pdy = dy;

                    direction = 0;
                    if dx == movement_node.x {
                        // north or south
                        if dy > movement_node.y {
                            direction = 1; // north
                            dy -= 1;
                        } else if dy < movement_node.y {
                            direction = 2; // south
                            dy += 1;
                        } else {
                            panic!("asdf");
                        }
                    } else {
                        // east or west
                        if dx > movement_node.x {
                            direction = 3; // west
                            dx -= 1;
                        } else if dx < movement_node.x {
                            direction = 4; // east
                            dx += 1;
                        } else {
                            panic!("asdf");
                        }
                    }

                    println!("sending {}", direction);
                    ic.send(direction);
                    status = ic.recv() as i32;
                    println!("saw {}", status);
                }
            }
            None => {
                panic!("123");
            }
        }

        // the last transition will either pass or fail
        match status {
            0 => {
                panels.set(dx, dy, GridItem::Wall);
                map.update_node(dx, dy, NodeStatus::Wall);

                // remove edge!
                map.remove_edge(pdx, pdy, dx, dy);

                // if it failed, reset the drone coords
                match direction {
                    1 => {
                        // drone was going north
                        dy += 1;
                    }
                    2 => {
                        // drone was going south
                        dy -= 1;
                    }
                    3 => {
                        // drone was going west
                        dx += 1;
                    }
                    4 => {
                        // drone was going east
                        dx -= 1;
                    }
                    _ => {
                        panic!("bleh");
                    }
                }

                println!("hit wall, reset to {} {}", dx, dy);
            }
            1 => {
                println!("success from {} {} to {} {}", pdx, pdy, dx, dy);

                // if successful, add node to graph (plus edge)
                panels.set(dx, dy, GridItem::Empty);
                map.update_node(dx, dy, NodeStatus::Empty);

                // add more search locations, skip what we've searched before
                if !map.node_exists(dx - 1, dy) {
                    println!("pushing {} {}", dx - 1, dy);
                    search_stack.push((dx - 1, dy));
                    map.add_node(dx - 1, dy, NodeStatus::NotSure);
                    map.add_edge(dx - 1, dy, dx, dy);
                }
                if !map.node_exists(dx + 1, dy) {
                    println!("pushing {} {}", dx + 1, dy);
                    search_stack.push((dx + 1, dy));
                    map.add_node(dx + 1, dy, NodeStatus::NotSure);
                    map.add_edge(dx + 1, dy, dx, dy);
                }
                if !map.node_exists(dx, dy - 1) {
                    println!("pushing {} {}", dx, dy - 1);
                    search_stack.push((dx, dy - 1));
                    map.add_node(dx, dy - 1, NodeStatus::NotSure);
                    map.add_edge(dx, dy - 1, dx, dy);
                }
                if !map.node_exists(dx, dy + 1) {
                    println!("pushing {} {}", dx, dy + 1);
                    search_stack.push((dx, dy + 1));
                    map.add_node(dx, dy + 1, NodeStatus::NotSure);
                    map.add_edge(dx, dy + 1, dx, dy);
                }
            }
            2 => {
                // if oxygen, report shortest path to (0,0)
                panels.set(dx, dy, GridItem::Oxygen);
                map.update_node(dx, dy, NodeStatus::Oxygen);

                println!(
                    "shortest path: {}",
                    map.return_shortest_path_length(dx, dy, 0, 0)
                );

                // part 2: make complete map
                if !map.node_exists(dx - 1, dy) {
                    println!("pushing {} {}", dx - 1, dy);
                    search_stack.push((dx - 1, dy));
                    map.add_node(dx - 1, dy, NodeStatus::NotSure);
                    map.add_edge(dx - 1, dy, dx, dy);
                }
                if !map.node_exists(dx + 1, dy) {
                    println!("pushing {} {}", dx + 1, dy);
                    search_stack.push((dx + 1, dy));
                    map.add_node(dx + 1, dy, NodeStatus::NotSure);
                    map.add_edge(dx + 1, dy, dx, dy);
                }
                if !map.node_exists(dx, dy - 1) {
                    println!("pushing {} {}", dx, dy - 1);
                    search_stack.push((dx, dy - 1));
                    map.add_node(dx, dy - 1, NodeStatus::NotSure);
                    map.add_edge(dx, dy - 1, dx, dy);
                }
                if !map.node_exists(dx, dy + 1) {
                    println!("pushing {} {}", dx, dy + 1);
                    search_stack.push((dx, dy + 1));
                    map.add_node(dx, dy + 1, NodeStatus::NotSure);
                    map.add_edge(dx, dy + 1, dx, dy);
                }
            }
            _ => {
                panic!("bleh");
            }
        }
    }

    println!("checking fill time");

    let mut oxygen_stack: Vec<Vec<NodeIndex<DefaultIx>>> = Vec::new();
    let mut minutes: i32 = 0;

    oxygen_stack.push(vec![map.find_oxygen_node()]);

    while let Some(node_list) = oxygen_stack.pop() {
        println!("----------------");
        display(&panels, dx, dy);

        let mut next_stack: Vec<NodeIndex<DefaultIx>> = Vec::new();

        for node in node_list {
            let neighbor_indexes: Vec<NodeIndex<DefaultIx>> = map.graph.neighbors(node).collect();
            for neighbor_index in neighbor_indexes {
                let neighbor_node = map.get_node_by_index_mut(neighbor_index);
                if neighbor_node.status == NodeStatus::Empty {
                    neighbor_node.status = NodeStatus::Oxygen;
                    panels.set(neighbor_node.x, neighbor_node.y, GridItem::Oxygen);
                    next_stack.push(neighbor_node.index);
                }
            }
        }

        if next_stack.len() > 0 {
            oxygen_stack.push(next_stack);
            minutes += 1;
        }
    }

    println!("minutes to fill: {}", minutes);
}
