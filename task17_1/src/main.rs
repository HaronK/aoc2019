use crate::robot::*;
use anyhow::{anyhow, Result};
use common::log::*;
use std::fs::File;
use std::io::{prelude::*, BufReader};

mod robot;

fn main() -> Result<()> {
    let log = Log::new(false);
    let file = File::open("input.txt")?;
    let reader = BufReader::new(file);
    let prog_str = reader
        .lines()
        .nth(0)
        .ok_or_else(|| anyhow!("ERROR: Cannot read program string."))??;

    let mut robot = Robot::new(&prog_str, &log)?;

    robot.camera_scan()?;
    // robot.show()?;

    let intersections = robot.get_intersections();

    println!("Intersections: {:?}", intersections);

    let calibration: usize = intersections.iter().map(|p| p.x * p.y).sum();

    println!("Calibration: {}", calibration);

    robot.wake_up();

    let dust_count = robot.move_robot(
        "A,A,B,C,C,A,B,C,A,B",
        "L,12,L,12,R,12",
        "L,8,L,8,R,12,L,8,L,8",
        "L,10,R,8,R,12",
    )?;

    println!("Dust count: {}", dust_count);

    Ok(())
}

// Manually calculated :)
// L12,L12,R12,L12,L12,R12,L8,L8,R12,L8,L8,L10,R8,R12,L10,R8,R12,L12,L12,R12,L8,L8,R12,L8,L8,L10,R8,R12,L12,L12,R12,L8,L8,R12,L8,L8

// A,A,B,C,C,A,B,C,A,B

// A: L,12,L,12,R,12
// B: L,8,L,8,R,12,L,8,L,8
// C: L,10,R,8,R,12
