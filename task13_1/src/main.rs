use crate::arcade::*;
use crate::intcode_comp::*;
use crate::log::*;
use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{prelude::*, BufReader};

pub mod arcade;
pub mod intcode_comp;
pub mod log;

fn main() -> Result<()> {
    let log = Log::new(false);
    let file = File::open("input.txt")?;
    let reader = BufReader::new(file);
    let prog_str = reader
        .lines()
        .nth(0)
        .ok_or_else(|| anyhow!("ERROR: Cannot read program string."))??;

    let prog = parse_prog(prog_str)?;
    println!("Prog len: {}", prog.len());
    log.println(format!("Prog: {:?}", prog));
    let mut arcade = Arcade::new(prog, false, &log);

    arcade.build_map()?;
    println!(
        "Blocks count: {:?}",
        arcade.get_tiles_by_id(TileType::Block)
    );

    let score = arcade.run()?;
    println!("Score: {}", score);

    Ok(())
}

fn parse_prog<S: AsRef<str>>(commands: S) -> Result<Vec<DataType>> {
    let cmd_str: Vec<&str> = commands.as_ref().split(',').collect();
    let mut prog: Vec<DataType> = Vec::new();
    for cmd in cmd_str {
        prog.push(cmd.parse()?);
    }
    Ok(prog)
}
