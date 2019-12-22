use std::io::{self, BufRead};
use std::convert;

// Fuel required to launch a given module is based on its mass.
// Specifically, to find the fuel required for a module, take its mass,
// divide by three, round down, and subtract 2.

// part 2:
// Fuel itself requires fuel just like a module - take its mass, divide
// by three, round down, and subtract 2. However, that fuel also
// requires fuel, and that fuel requires fuel, and so on. Any mass that
// would require negative fuel should instead be treated as if it
// requires zero fuel; the remaining mass, if any, is instead handled by
// wishing really hard, which has no mass and is outside the scope of
// this calculation.

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

fn get_fuel_part_2(mass: i32) -> i32 {
    let fuel = get_fuel(mass);
    if (fuel < 0) {
        return 0;
    }
    return fuel + get_fuel_part_2(fuel);
}

#[test]
fn test_get_fuel_part_2() {
    assert_eq!(get_fuel_part_2(14), 2);
    assert_eq!(get_fuel_part_2(1969), 966);
    assert_eq!(get_fuel_part_2(100756), 50346);
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
        fuel += get_fuel_part_2(numbers[i]);
    }

    println!("{}", fuel);
}
