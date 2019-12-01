use anyhow::Result;
use std::fs::File;
use std::io::{prelude::*, BufReader};

fn main() -> Result<()> {
    let file = File::open("input.txt")?;
    let reader = BufReader::new(file);

    let mut total_fuel: u32 = 0;
    for line in reader.lines() {
        let mut mass: u32 = line?.parse()?;

        loop {
            let mass3 = mass / 3;
            if mass3 <= 2 {
                break;
            }
            let fuel = mass3 - 2;
            // println!("{} -> {}", mass, fuel);
            total_fuel += fuel;
            mass = fuel;
        }
    }
    println!("Total fuel: {}", total_fuel);

    Ok(())
}
