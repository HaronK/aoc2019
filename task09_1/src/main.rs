use crate::intcode_comp::*;
use crate::log::*;
use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{prelude::*, BufReader};

pub mod intcode_comp;
pub mod log;

fn main() -> Result<()> {
    let log = Log::new(true);
    let file = File::open("input.txt")?;
    let reader = BufReader::new(file);
    let prog_str = reader
        .lines()
        .nth(0)
        .ok_or_else(|| anyhow!("ERROR: Cannot read program string."))??;

    let prog = parse_prog(prog_str)?;
    log.println(format!("Prog: {:?}", prog));
    let mut comp = IntcodeComp::new(prog, &log);

    comp.add_input(2); // 1 for the first task

    let output = comp.exec()?;

    println!("Output: {:?}", output);

    Ok(())
}

fn parse_prog<S: AsRef<str>>(commands: S) -> Result<Vec<DataType>> {
    let cmd_str: Vec<&str> = commands.as_ref().split(",").collect();
    let mut prog: Vec<DataType> = Vec::new();
    for cmd in cmd_str {
        prog.push(cmd.parse()?);
    }
    Ok(prog)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() -> Result<()> {
        let log = Log::new(false);
        let prog = parse_prog("109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99")?;
        let mut comp = IntcodeComp::new(prog, &log);

        assert_eq!(
            vec![109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99],
            comp.exec()?
        );

        Ok(())
    }

    #[test]
    fn test2() -> Result<()> {
        let log = Log::new(false);
        let prog = parse_prog("1102,34915192,34915192,7,4,7,99,0")?;
        let mut comp = IntcodeComp::new(prog, &log);

        assert_eq!(vec![1219070632396864], comp.exec()?);

        Ok(())
    }

    #[test]
    fn test3() -> Result<()> {
        let log = Log::new(false);
        let prog = parse_prog("104,1125899906842624,99")?;
        let mut comp = IntcodeComp::new(prog, &log);

        assert_eq!(vec![1125899906842624], comp.exec()?);

        Ok(())
    }
}
