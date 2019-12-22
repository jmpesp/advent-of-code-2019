use std::io::{self, BufRead};
use std::convert;

// Fuel required to launch a given module is based on its mass.
// Specifically, to find the fuel required for a module, take its mass,
// divide by three, round down, and subtract 2.

fn get_fuel(mass: i32) -> i32 {
    return mass / 3 - 2;
}

#[test]
fn test_get_fuel() {
    assert_eq!(get_fuel(12), 2);
    assert_eq!(get_fuel(14), 2);
    assert_eq!(get_fuel(1969), 654);
    assert_eq!(get_fuel(100756), 33583);
}

fn main() {
    let reader = io::stdin();
    let numbers: Vec<i32> =
        reader.lock()
              .lines()
              .map(|s| s.unwrap().parse::<i32>().unwrap())
              .collect();

    let mut fuel: i32 = 0;

    for i in 0..numbers.len() {
        fuel += get_fuel(numbers[i]);
    }

    println!("{}", fuel);
}
