use anyhow::Result;
use std::fs::File;
use std::io::{prelude::*, BufReader};

fn main() -> Result<()> {
    let file = File::open("input.txt")?;
    let reader = BufReader::new(file);

    let mut total_fuel: u32 = 0;
    for line in reader.lines() {
        let mass: u32 = line?.parse()?;
        let fuel = mass / 3 - 2;
        println!("{} -> {}", mass, fuel);
        total_fuel += fuel;
    }
    println!("Total fuel: {}", total_fuel);

    Ok(())
}
