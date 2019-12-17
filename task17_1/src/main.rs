use crate::camera::*;
use anyhow::{anyhow, Result};
use common::log::*;
use std::fs::File;
use std::io::{prelude::*, BufReader};

mod camera;

fn main() -> Result<()> {
    let log = Log::new(false);
    let file = File::open("input.txt")?;
    let reader = BufReader::new(file);
    let prog_str = reader
        .lines()
        .nth(0)
        .ok_or_else(|| anyhow!("ERROR: Cannot read program string."))??;

    let mut camera = Camera::new(&prog_str, &log)?;

    camera.run()?;
    camera.show()?;

    let intersections = camera.get_intersections();

    println!("Intersections: {:?}", intersections);

    let calibration: usize = intersections.iter().map(|p| p.x * p.y).sum();

    println!("Calibration: {}", calibration);

    Ok(())
}
