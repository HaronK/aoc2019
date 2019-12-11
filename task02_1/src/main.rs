use anyhow::{bail, Result};
use std::fs::File;
use std::io::{prelude::*, BufReader};

fn main() -> Result<()> {
    let file = File::open("input.txt")?;
    let reader = BufReader::new(file);

    if let Some(cmd_str) = reader.lines().nth(0) {
        let commands = cmd_str?;
        println!("Input: {}", commands);

        let mut prog = parse(&commands)?;
        prog[1] = 12;
        prog[2] = 2;

        eval(&mut prog)?;

        println!(
            "Result: {}",
            prog.iter()
                .map(|v| format!("{}", v))
                .collect::<Vec<String>>()
                .join(",")
        );
    } else {
        bail!("ERROR: Cannot read command.");
    }

    Ok(())
}

fn parse(commands: &str) -> Result<Vec<u32>> {
    let cmd_str: Vec<&str> = commands.split(',').collect();
    let mut prog: Vec<u32> = Vec::new();
    for cmd in cmd_str {
        prog.push(cmd.parse()?);
    }
    Ok(prog)
}

fn eval(prog: &mut Vec<u32>) -> Result<()> {
    let mut ip: usize = 0;
    loop {
        if ip >= prog.len() {
            break;
        }

        match prog[ip] {
            1 => {
                let (param1_idx, param2_idx, res_idx) = get_cmd_params(ip, &prog)?;
                prog[res_idx] = prog[param1_idx] + prog[param2_idx];
            }
            2 => {
                let (param1_idx, param2_idx, res_idx) = get_cmd_params(ip, &prog)?;
                prog[res_idx] = prog[param1_idx] * prog[param2_idx];
            }
            99 => {
                break;
            }
            _ => bail!("ERROR: Unsupported command: {}", prog[ip]),
        }
        ip += 4;
    }

    Ok(())
}

fn get_cmd_params(ip: usize, prog: &[u32]) -> Result<(usize, usize, usize)> {
    if ip + 3 >= prog.len() {
        bail!(
            "ERROR: Not enough parameters for the command {} at position {}. Prog len: {}.",
            prog[ip],
            ip,
            prog.len()
        );
    }

    let param1_idx = prog[ip + 1] as usize;
    if param1_idx >= prog.len() {
        bail!("ERROR: First parameter index {} of the of the command {} at position {} is out of program buffer. Prog len: {}.", param1_idx, prog[ip], ip, prog.len());
    }

    let param2_idx = prog[ip + 2] as usize;
    if param2_idx >= prog.len() {
        bail!("ERROR: Second parameter index {} of the of the command {} at position {} is out of program buffer. Prog len: {}.", param2_idx, prog[ip], ip, prog.len());
    }

    let res_idx = prog[ip + 3] as usize;
    if res_idx >= prog.len() {
        bail!("ERROR: Result parameter index {} of the of the command {} at position {} is out of program buffer. Prog len: {}.", res_idx, prog[ip], ip, prog.len());
    }

    Ok((param1_idx, param2_idx, res_idx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add1() -> Result<()> {
        let mut prog = parse(&"1,0,0,0,99".to_string())?;
        eval(&mut prog)?;
        assert_eq!(prog, vec![2, 0, 0, 0, 99]);
        Ok(())
    }

    #[test]
    fn test_add2() -> Result<()> {
        let mut prog = parse(&"1,1,1,4,99,5,6,0,99".to_string())?;
        eval(&mut prog)?;
        assert_eq!(prog, vec![30, 1, 1, 4, 2, 5, 6, 0, 99]);
        Ok(())
    }

    #[test]
    fn test_mul1() -> Result<()> {
        let mut prog = parse(&"2,3,0,3,99".to_string())?;
        eval(&mut prog)?;
        assert_eq!(prog, vec![2, 3, 0, 6, 99]);
        Ok(())
    }

    #[test]
    fn test_mul2() -> Result<()> {
        let mut prog = parse(&"2,4,4,5,99,0".to_string())?;
        eval(&mut prog)?;
        assert_eq!(prog, vec![2, 4, 4, 5, 99, 9801]);
        Ok(())
    }
}
