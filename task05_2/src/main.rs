use anyhow::{anyhow, bail, ensure, Context, Result};
use std::fs::File;
use std::io::{self, prelude::*, BufReader};

#[derive(Debug)]
enum Command {
    Add,         // 1
    Mul,         // 2
    Read,        // 3
    Write,       // 4
    JumpIfTrue,  // 5
    JumpIfFalse, // 6
    LessThan,    // 7
    Equals,      // 8
    Exit,        // 99
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum ParamMode {
    Position,  // 0
    Immediate, // 1
}

fn main() -> Result<()> {
    let file = File::open("input.txt")?;
    let reader = BufReader::new(file);
    let prog_str = reader
        .lines()
        .nth(0)
        .ok_or_else(|| anyhow!("ERROR: Cannot read program string."))??;

    let mut prog = parse_prog(&prog_str)?;
    // println!("Program: [{}]{:?}", prog.len(), prog);

    let output = eval(&mut prog)?;
    println!("Output: {:?}", output);

    Ok(())
}

fn parse_prog(commands: &String) -> Result<Vec<i32>> {
    let cmd_str: Vec<&str> = commands.split(",").collect();
    let mut prog: Vec<i32> = Vec::new();
    for cmd in cmd_str {
        prog.push(cmd.parse()?);
    }
    Ok(prog)
}

fn eval(prog: &mut Vec<i32>) -> Result<Vec<i32>> {
    let mut result: Vec<i32> = Vec::new();
    let mut ip: usize = 0;
    while ip < prog.len() {
        ip = eval_cmd(ip, prog, &mut result)?;
        if ip == 0 {
            break;
        }
    }

    Ok(result)
}

/// Returns next ip
fn eval_cmd(ip: usize, prog: &mut Vec<i32>, output: &mut Vec<i32>) -> Result<usize> {
    // check_ip(ip, prog.len(), format!("Cannot read command opcode"));

    let (cmd, params) = parse_opcode(prog[ip])?;
    // println!("Command[{}:{}]: {:?}({:?})", ip, prog[ip], cmd, params);
    check_ip(
        ip + params.len() - 1,
        prog.len(),
        format!(
            "Not enough parameters for the command {:?} at position {}",
            cmd, ip
        ),
    )?;

    match cmd {
        Command::Add => {
            ensure!(
                params.len() == 3,
                "ERROR: Expected 3 parameters in add command but was {}",
                params.len()
            );
            ensure!(
                params[2] == ParamMode::Position,
                "ERROR: Destination parameter should be in position mode."
            );

            let v1 = get_param_value(ip + 1, params[0], prog)?;
            let v2 = get_param_value(ip + 2, params[1], prog)?;

            set_param_value(ip + 3, prog, v1 + v2)?;
        }
        Command::Mul => {
            ensure!(
                params.len() == 3,
                "ERROR: Expected 3 parameters in mul command but was {}",
                params.len()
            );
            ensure!(
                params[2] == ParamMode::Position,
                "ERROR: Destination parameter should be in position mode."
            );

            let v1 = get_param_value(ip + 1, params[0], prog)?;
            let v2 = get_param_value(ip + 2, params[1], prog)?;

            set_param_value(ip + 3, prog, v1 * v2)?;
        }
        Command::Read => {
            ensure!(
                params.len() == 1,
                "ERROR: Expected 1 parameters in read command but was {}",
                params.len()
            );
            ensure!(
                params[0] == ParamMode::Position,
                "ERROR: Destination parameter should be in position mode."
            );

            let mut buf = String::new();

            print!("Input: ");
            let _ = io::stdout().flush();

            io::stdin()
                .read_line(&mut buf)
                .with_context(|| "ERROR: Failed to read from stdin")?;
            set_param_value(
                ip + 1,
                prog,
                buf.trim()
                    .parse()
                    .with_context(|| format!("ERROR: Cannot parse buffer '{}'", buf))?,
            )?;
        }
        Command::Write => {
            ensure!(
                params.len() == 1,
                "ERROR: Expected 1 parameters in write command but was {}",
                params.len()
            );

            let res = get_param_value(ip + 1, params[0], prog)?;
            println!("Output: {}", res);
            output.push(res);
        }
        Command::JumpIfTrue => {
            ensure!(
                params.len() == 2,
                "ERROR: Expected 3 parameters in add command but was {}",
                params.len()
            );
            // ensure!(params[1] == ParamMode::Position, "ERROR: Destination parameter should be in position mode.");

            let v1 = get_param_value(ip + 1, params[0], prog)?;

            if v1 != 0 {
                return Ok(get_param_value(ip + 2, params[1], prog)? as usize);
            }
        }
        Command::JumpIfFalse => {
            ensure!(
                params.len() == 2,
                "ERROR: Expected 3 parameters in add command but was {}",
                params.len()
            );
            // ensure!(params[1] == ParamMode::Position, "ERROR: Destination parameter should be in position mode.");

            let v1 = get_param_value(ip + 1, params[0], prog)?;

            if v1 == 0 {
                return Ok(get_param_value(ip + 2, params[1], prog)? as usize);
            }
        }
        Command::LessThan => {
            ensure!(
                params.len() == 3,
                "ERROR: Expected 3 parameters in mul command but was {}",
                params.len()
            );
            ensure!(
                params[2] == ParamMode::Position,
                "ERROR: Destination parameter should be in position mode."
            );

            let v1 = get_param_value(ip + 1, params[0], prog)?;
            let v2 = get_param_value(ip + 2, params[1], prog)?;

            set_param_value(ip + 3, prog, if v1 < v2 { 1 } else { 0 })?;
        }
        Command::Equals => {
            ensure!(
                params.len() == 3,
                "ERROR: Expected 3 parameters in mul command but was {}",
                params.len()
            );
            ensure!(
                params[2] == ParamMode::Position,
                "ERROR: Destination parameter should be in position mode."
            );

            let v1 = get_param_value(ip + 1, params[0], prog)?;
            let v2 = get_param_value(ip + 2, params[1], prog)?;

            set_param_value(ip + 3, prog, if v1 == v2 { 1 } else { 0 })?;
        }
        Command::Exit => return Ok(0),
    }

    Ok(ip + params.len() + 1)
}

/// Returns command and its parameter modes
fn parse_opcode(opcode: i32) -> Result<(Command, Vec<ParamMode>)> {
    let mut opc = opcode;
    let cmd_id = match opc % 100 {
        1 => Command::Add,
        2 => Command::Mul,
        3 => Command::Read,
        4 => Command::Write,
        5 => Command::JumpIfTrue,
        6 => Command::JumpIfFalse,
        7 => Command::LessThan,
        8 => Command::Equals,
        99 => Command::Exit,
        _ => bail!("ERROR: Unknown command id {}", opc % 100),
    };
    let mut params: Vec<ParamMode> = Vec::new();

    opc /= 100;

    let params_count = match cmd_id {
        Command::Add | Command::Mul | Command::LessThan | Command::Equals => 3,
        Command::Read | Command::Write => 1,
        Command::JumpIfTrue | Command::JumpIfFalse => 2,
        Command::Exit => 0,
    };

    for _i in 0..params_count {
        params.push(if opc % 10 == 0 {
            ParamMode::Position
        } else {
            ParamMode::Immediate
        });
        opc /= 10;
    }

    Ok((cmd_id, params))
}

fn check_ip(ip: usize, prog_len: usize, msg: String) -> Result<()> {
    ensure!(
        ip < prog_len,
        "ERROR: {}. Instruction pointer {} is out of program bound length {}",
        msg,
        ip,
        prog_len
    );
    Ok(())
}

fn get_param_value(param_ip: usize, mode: ParamMode, prog: &Vec<i32>) -> Result<i32> {
    let value = match mode {
        ParamMode::Position => {
            let val_ip = prog[param_ip] as usize;
            check_ip(val_ip, prog.len(), format!("Cannot read"))?;
            // println!("  in: ip={}->{} value={}", param_ip, val_ip, prog[val_ip]);
            prog[val_ip]
        }
        ParamMode::Immediate => {
            // println!("  in: ip={} value={}", param_ip, prog[param_ip]);
            prog[param_ip]
        }
    };

    Ok(value)
}

fn set_param_value(param_ip: usize, prog: &mut Vec<i32>, value: i32) -> Result<()> {
    check_ip(param_ip, prog.len(), format!("Cannot store value"))?;
    let val_ip = prog[param_ip] as usize;
    check_ip(val_ip, prog.len(), format!("Cannot store value"))?;
    prog[val_ip] = value;
    // println!("  out: ip={}->{} value={}", param_ip, val_ip, value);
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test1() -> Result<()> {
//         let mut prog = parse_prog(&"3,9,8,9,10,9,4,9,99,-1,8".to_owned())?;
//         assert_eq!(eval(&mut prog)?, vec![0]);
//         Ok(())
//     }

//     #[test]
//     fn test2() -> Result<()> {
//         let mut prog = parse_prog(&"3,9,7,9,10,9,4,9,99,-1,8".to_owned())?;
//         assert_eq!(eval(&mut prog)?, vec![0]);
//         Ok(())
//     }
// }
