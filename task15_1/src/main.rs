use crate::repair_droid::*;
use anyhow::{anyhow, Result};
use common::log::*;
use std::fs::File;
use std::io::{prelude::*, BufReader};

mod repair_droid;

fn main() -> Result<()> {
    let log = Log::new(false);
    let file = File::open("input.txt")?;
    let reader = BufReader::new(file);
    let prog_str = reader
        .lines()
        .nth(0)
        .ok_or_else(|| anyhow!("ERROR: Cannot read program string."))??;

    let mut droid = RepairDroid::new(&prog_str, &log)?;

    // droid.interactive()?;
    droid.open_map(false)?;

    let dist = droid.distance_to_oxygen(true)?;

    println!("Oxygen dist: {}", dist);

    Ok(())
}
