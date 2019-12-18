use std::io::{self, BufRead};

fn main() {
    // As input, FFT takes a list of numbers. In the signal you
    // received (your puzzle input), each number is a single digit:
    // data like 15243 represents the sequence 1, 5, 2, 4, 3.

    let reader = io::stdin();
    let numbers: Vec<u32> =
        reader.lock()
              .lines().next().unwrap().unwrap() // iterator -> Option<io::Result<String>>
              .char_indices()                   // String -> iterator [character]
              .map(|s| s.1.to_digit(10).expect("could not turn into digit!"))
              .collect();

    println!("Input is {} characters: {:?}", numbers.len(), numbers);
}
