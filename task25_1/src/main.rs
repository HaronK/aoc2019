use crate::droid::*;
use anyhow::{anyhow, Result};
use common::log::*;
use std::fs::File;
use std::io::{prelude::*, BufReader};

mod droid;

fn main() -> Result<()> {
    let log = Log::new(false);
    let file = File::open("input.txt")?;
    let reader = BufReader::new(file);
    let prog_str = reader
        .lines()
        .nth(0)
        .ok_or_else(|| anyhow!("ERROR: Cannot read program string."))??;

    let mut droid = Droid::new(&prog_str, &log)?;

    droid.interactive()?;

    Ok(())
}
