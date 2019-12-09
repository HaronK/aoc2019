use crate::log::*;
use anyhow::{bail, ensure, Result};

#[derive(Debug)]
enum Command {
    Add,           // 1
    Mul,           // 2
    Read,          // 3
    Write,         // 4
    JumpIfTrue,    // 5
    JumpIfFalse,   // 6
    LessThan,      // 7
    Equals,        // 8
    AdjustRelBase, // 9
    Exit,          // 99
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum ParamMode {
    Position,  // 0
    Immediate, // 1
    Relative,  // 2
}

#[derive(PartialEq, Debug)]
enum Status {
    Running,
    Paused,
    Halted,
}

pub type DataType = i64;

pub struct IntcodeComp<'a> {
    prog: Vec<DataType>,
    ip: usize,
    rel_base: usize,
    input: Vec<DataType>,
    status: Status,
    log: &'a Log,
}

impl<'a> IntcodeComp<'a> {
    pub fn new(prog: Vec<DataType>, log: &'a Log) -> Self {
        log.println("    New comp");
        Self {
            prog,
            ip: 0,
            rel_base: 0,
            input: Vec::new(),
            status: Status::Running,
            log,
        }
    }

    pub fn is_running(&self) -> bool {
        self.status == Status::Running
    }

    pub fn is_halted(&self) -> bool {
        self.status == Status::Halted
    }

    pub fn add_input(&mut self, input: DataType) {
        self.input.push(input);
    }

    /// Run whole program and return outputs
    pub fn exec(&mut self) -> Result<Vec<DataType>> {
        let mut output = Vec::new();
        while !self.is_halted() {
            output.push(self.run()?);
        }
        output.pop();
        Ok(output)
    }

    /// Run computer until next output
    pub fn run(&mut self) -> Result<DataType> {
        ensure!(
            !self.is_halted(),
            "ERROR: Program was halted. ip={} status={:?}.",
            self.ip,
            self.status
        );

        let mut output = if self.input.len() > 0 { self.input[self.input.len() - 1] } else { 0 };

        self.status = Status::Running;

        self.log.println(format!("Input: {:?}", self.input));

        while self.eval_cmd(&mut output)? {}

        if self.input.len() > 0 {
            self.log.println(format!(
                "WARNING: Input buffer was not consumed completely. Remaining values: {:?}.",
                self.input
            ));
        }

        self.log.println(format!("Output: {}", output));
        self.log.println(format!("Status: {:?}", self.status));

        Ok(output)
    }

    fn dump_cmd(&mut self, cmd: &Command, params: &Vec<ParamMode>) -> Result<()> {
        self.log.print(format!(
            "  Command[{}:{}]: {:?}(",
            self.ip, self.prog[self.ip], cmd
        ));

        for i in 0..params.len() {
            let val = self.get_param_value(i + 1, params[i])?;
            self.log.print(format!(
                "{:?}:{}->{}, ",
                params[i],
                self.prog[self.ip + i + 1],
                val
            ));
        }

        self.log.println(format!(")"));
        Ok(())
    }

    fn rel_ip(&self, offset: DataType) -> usize {
        (self.rel_base as DataType + offset) as usize
    }

    /// Returns false if execution should be stopped or paused
    fn eval_cmd(&mut self, output: &mut DataType) -> Result<bool> {
        ensure!(self.is_running(), "ERROR: Program is not running.");
        // self.check_ip(self.ip, format!("Cannot read command opcode"));

        let (cmd, params) = self.parse_opcode(self.ip)?;

        self.dump_cmd(&cmd, &params)?;

        // self.check_ip(
        //     self.ip + params.len() - 1,
        //     format!(
        //         "Not enough parameters for the command {:?} at position {}",
        //         cmd, self.ip
        //     ),
        // )?;

        match cmd {
            Command::Add => {
                // ensure!(
                //     params.len() == 3,
                //     "ERROR: Expected 3 parameters in add command but was {}",
                //     params.len()
                // );

                let v1 = self.get_param_value(1, params[0])?;
                let v2 = self.get_param_value(2, params[1])?;

                self.set_param_value(3, params[2], v1 + v2)?;
            }
            Command::Mul => {
                // ensure!(
                //     params.len() == 3,
                //     "ERROR: Expected 3 parameters in mul command but was {}",
                //     params.len()
                // );

                let v1 = self.get_param_value(1, params[0])?;
                let v2 = self.get_param_value(2, params[1])?;

                self.set_param_value(3, params[2], v1 * v2)?;
            }
            Command::Read => {
                // ensure!(
                //     params.len() == 1,
                //     "ERROR: Expected 1 parameters in read command but was {}",
                //     params.len()
                // );
                ensure!(self.input.len() > 0, "ERROR: Input buffer is empty.");

                let value = self.input.remove(0);

                self.set_param_value(1, params[0], value)?;
            }
            Command::Write => {
                // ensure!(
                //     params.len() == 1,
                //     "ERROR: Expected 1 parameters in write command but was {}",
                //     params.len()
                // );

                *output = self.get_param_value(1, params[0])?;
                self.status = Status::Paused;
            }
            Command::JumpIfTrue => {
                // ensure!(
                //     params.len() == 2,
                //     "ERROR: Expected 3 parameters in add command but was {}",
                //     params.len()
                // );

                let v1 = self.get_param_value(1, params[0])?;

                if v1 != 0 {
                    self.ip = self.get_param_value(2, params[1])? as usize;
                    return Ok(true);
                }
            }
            Command::JumpIfFalse => {
                // ensure!(
                //     params.len() == 2,
                //     "ERROR: Expected 3 parameters in add command but was {}",
                //     params.len()
                // );

                let v1 = self.get_param_value(1, params[0])?;

                if v1 == 0 {
                    self.ip = self.get_param_value(2, params[1])? as usize;
                    return Ok(true);
                }
            }
            Command::LessThan => {
                // ensure!(
                //     params.len() == 3,
                //     "ERROR: Expected 3 parameters in mul command but was {}",
                //     params.len()
                // );

                let v1 = self.get_param_value(1, params[0])?;
                let v2 = self.get_param_value(2, params[1])?;

                self.set_param_value(3, params[2], if v1 < v2 { 1 } else { 0 })?;
            }
            Command::Equals => {
                // ensure!(
                //     params.len() == 3,
                //     "ERROR: Expected 3 parameters in mul command but was {}",
                //     params.len()
                // );

                let v1 = self.get_param_value(1, params[0])?;
                let v2 = self.get_param_value(2, params[1])?;

                self.set_param_value(3, params[2], if v1 == v2 { 1 } else { 0 })?;
            }
            Command::AdjustRelBase => {
                // ensure!(
                //     params.len() == 1,
                //     "ERROR: Expected 1 parameters in mul command but was {}",
                //     params.len()
                // );

                let v1 = self.get_param_value(1, params[0])?;

                self.rel_base = self.rel_ip(v1);
            }
            Command::Exit => {
                self.status = Status::Halted;
            }
        }

        self.ip += params.len() + 1;

        Ok(self.status == Status::Running)
    }

    /// Returns command and its parameter modes
    fn parse_opcode(&self, ip: usize) -> Result<(Command, Vec<ParamMode>)> {
        self.check_ip(ip, format!("Cannot read opcode"))?;

        let mut opc = self.prog[ip];
        let cmd_id = match opc % 100 {
            1 => Command::Add,
            2 => Command::Mul,
            3 => Command::Read,
            4 => Command::Write,
            5 => Command::JumpIfTrue,
            6 => Command::JumpIfFalse,
            7 => Command::LessThan,
            8 => Command::Equals,
            9 => Command::AdjustRelBase,
            99 => Command::Exit,
            _ => bail!("ERROR: Unknown command id {}. ip={}.", opc % 100, ip),
        };
        let mut params: Vec<ParamMode> = Vec::new();

        opc /= 100;

        let params_count = match cmd_id {
            Command::Add | Command::Mul | Command::LessThan | Command::Equals => 3,
            Command::Read | Command::Write | Command::AdjustRelBase => 1,
            Command::JumpIfTrue | Command::JumpIfFalse => 2,
            Command::Exit => 0,
        };

        for _i in 0..params_count {
            let param_mode = match opc % 10 {
                0 => ParamMode::Position,
                1 => ParamMode::Immediate,
                2 => ParamMode::Relative,
                _ => bail!("ERROR: Unknown parameter mode {}", opc % 10),
            };
            params.push(param_mode);
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

    fn check_and_extend(&mut self, ip: usize) {
        if ip >= self.prog.len() {
            self.log.println(format!("    Extending program from {} to {} elements.", self.prog.len(), ip + 1));
            self.prog.resize(ip + 1, 0);
        }
    }

    fn get_param_value(&mut self, param_offset: usize, mode: ParamMode) -> Result<DataType> {
        let ip = self.ip + param_offset;
        // self.log.println(format!("    get_param_value({}, {:?}): ip={} prog={} ip_off={}", param_offset, mode, self.ip, self.prog.len(), ip));

        self.check_and_extend(ip);

        let value = match mode {
            ParamMode::Position => {
                let val_ip = self.prog[ip] as usize;
                self.check_and_extend(val_ip);
                // self.log.println(format!("  in: ip={}->{} value={}", ip, val_ip, self.prog[val_ip]));
                self.prog[val_ip]
            }
            ParamMode::Immediate => {
                // self.log.println(format!("  in: ip={} value={}", ip, self.prog[ip]));
                self.prog[ip]
            }
            ParamMode::Relative => {
                let val_ip = self.rel_ip(self.prog[ip]);
                // self.log.print(format!("<val_ip={} rel_base={}>", val_ip, self.rel_base));
                self.check_and_extend(val_ip);
                // self.log.println(format!("  in: ip={}->{}+{} value={}", ip, val_ip, self.rel_base, self.prog[val_ip]));
                self.prog[val_ip]
            }
        };

        Ok(value)
    }

    fn set_param_value(&mut self, param_offset: usize, mode: ParamMode, value: DataType) -> Result<()> {
        ensure!(
            mode == ParamMode::Position || mode == ParamMode::Relative,
            "ERROR: Wrong destination parameter. Expected Position or Relative but was {:?}.",
            mode
        );

        let ip = self.ip + param_offset;

        self.check_and_extend(ip);

        let val_ip = if mode == ParamMode::Relative {
            self.rel_ip(self.prog[ip])
        } else {
            self.prog[ip] as usize
        };

        self.check_and_extend(val_ip as usize);

        self.prog[val_ip as usize] = value;
        // self.log.println(format!("  out: ip={}->{} value={}", ip, val_ip, value));

        Ok(())
    }
}
