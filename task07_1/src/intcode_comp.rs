use anyhow::{bail, ensure, Result};

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

pub struct IntcodeComp {
    prog: Vec<i32>,
    ip: usize,
    input: Vec<i32>,
}

impl IntcodeComp {
    pub fn new(commands: &String) -> Result<Self> {
        let mut result = Self {
            prog: Vec::new(),
            ip: 0,
            input: Vec::new(),
        };

        result.parse(commands)?;

        Ok(result)
    }

    pub fn run(&mut self, input: Vec<i32>) -> Result<Vec<i32>> {
        let mut output: Vec<i32> = Vec::new();

        self.ip = 0;
        self.input = input;

        while self.eval_cmd(&mut output)? {
            self.check_ip(
                self.ip,
                format!("Command evaluation produced wrong instruction pointer"),
            )?;
        }

        if self.input.len() > 0 {
            println!(
                "WARNING: Input buffer was not consumed completely. Remaining values: {:?}",
                self.input
            );
        }

        Ok(output)
    }

    fn parse(&mut self, commands: &String) -> Result<()> {
        let cmd_str: Vec<&str> = commands.split(",").collect();
        for cmd in cmd_str {
            self.prog.push(cmd.parse()?);
        }
        Ok(())
    }

    /// Returns false if execution should be stopped
    fn eval_cmd(&mut self, output: &mut Vec<i32>) -> Result<bool> {
        // self.check_ip(self.ip, format!("Cannot read command opcode"));

        let (cmd, params) = IntcodeComp::parse_opcode(self.prog[self.ip])?;
        // println!("Command[{}:{}]: {:?}({:?})", self.ip, self.prog[self.ip], cmd, params);

        self.check_ip(
            self.ip + params.len() - 1,
            format!(
                "Not enough parameters for the command {:?} at position {}",
                cmd, self.ip
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

                let v1 = self.get_param_value(1, params[0])?;
                let v2 = self.get_param_value(2, params[1])?;

                self.set_param_value(3, v1 + v2)?;
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

                let v1 = self.get_param_value(1, params[0])?;
                let v2 = self.get_param_value(2, params[1])?;

                self.set_param_value(3, v1 * v2)?;
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
                ensure!(self.input.len() > 0, "ERROR: Input buffer is empty.");

                let value = self.input.remove(0);

                self.set_param_value(1, value)?;
            }
            Command::Write => {
                ensure!(
                    params.len() == 1,
                    "ERROR: Expected 1 parameters in write command but was {}",
                    params.len()
                );

                let res = self.get_param_value(1, params[0])?;

                output.push(res);
            }
            Command::JumpIfTrue => {
                ensure!(
                    params.len() == 2,
                    "ERROR: Expected 3 parameters in add command but was {}",
                    params.len()
                );

                let v1 = self.get_param_value(1, params[0])?;

                if v1 != 0 {
                    self.ip = self.get_param_value(2, params[1])? as usize;
                    return Ok(true);
                }
            }
            Command::JumpIfFalse => {
                ensure!(
                    params.len() == 2,
                    "ERROR: Expected 3 parameters in add command but was {}",
                    params.len()
                );

                let v1 = self.get_param_value(1, params[0])?;

                if v1 == 0 {
                    self.ip = self.get_param_value(2, params[1])? as usize;
                    return Ok(true);
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

                let v1 = self.get_param_value(1, params[0])?;
                let v2 = self.get_param_value(2, params[1])?;

                self.set_param_value(3, if v1 < v2 { 1 } else { 0 })?;
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

                let v1 = self.get_param_value(1, params[0])?;
                let v2 = self.get_param_value(2, params[1])?;

                self.set_param_value(3, if v1 == v2 { 1 } else { 0 })?;
            }
            Command::Exit => {
                self.ip += params.len() + 1;
                return Ok(false);
            }
        }

        self.ip += params.len() + 1;

        Ok(true)
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

    fn check_ip(&self, ip: usize, msg: String) -> Result<()> {
        ensure!(
            ip < self.prog.len(),
            "ERROR: {}. Instruction pointer {} is out of program bound length {}.",
            msg,
            ip,
            self.prog.len()
        );
        Ok(())
    }

    fn get_param_value(&self, param_offset: usize, mode: ParamMode) -> Result<i32> {
        let ip = self.ip + param_offset;

        self.check_ip(ip, format!("Cannot read value"))?;

        let value = match mode {
            ParamMode::Position => {
                let val_ip = self.prog[ip] as usize;
                self.check_ip(val_ip, format!("Cannot read"))?;
                // println!("  in: ip={}->{} value={}", ip, val_ip, self.prog[val_ip]);
                self.prog[val_ip]
            }
            ParamMode::Immediate => {
                // println!("  in: ip={} value={}", ip, self.prog[ip]);
                self.prog[ip]
            }
        };

        Ok(value)
    }

    fn set_param_value(&mut self, param_offset: usize, value: i32) -> Result<()> {
        let ip = self.ip + param_offset;

        self.check_ip(ip, format!("Cannot store value"))?;

        let val_ip = self.prog[ip] as usize;

        self.check_ip(val_ip, format!("Cannot store value"))?;

        self.prog[val_ip] = value;
        // println!("  out: ip={}->{} value={}", ip, val_ip, value);

        Ok(())
    }
}
