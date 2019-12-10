use crate::log::*;
use anyhow::{bail, ensure, Context, Result};

#[derive(Debug)]
enum Command {
    Add(ParamMode, ParamMode, ParamMode),      // 1
    Mul(ParamMode, ParamMode, ParamMode),      // 2
    Read(ParamMode),                           // 3
    Write(ParamMode),                          // 4
    JumpIfTrue(ParamMode, ParamMode),          // 5
    JumpIfFalse(ParamMode, ParamMode),         // 6
    LessThan(ParamMode, ParamMode, ParamMode), // 7
    Equals(ParamMode, ParamMode, ParamMode),   // 8
    AdjustRelBase(ParamMode),                  // 9
    Exit,                                      // 99
}

impl Command {
    fn parse(opc: DataType) -> Result<Self> {
        let cmd_id = opc % 100;
        let params = opc / 100;
        let cmd = match cmd_id {
            1 => {
                let params = ParamMode::parse(params, 3)?;
                Command::Add(params[0], params[1], params[2])
            }
            2 => {
                let params = ParamMode::parse(params, 3)?;
                Command::Mul(params[0], params[1], params[2])
            }
            3 => {
                let params = ParamMode::parse(params, 1)?;
                Command::Read(params[0])
            }
            4 => {
                let params = ParamMode::parse(params, 1)?;
                Command::Write(params[0])
            }
            5 => {
                let params = ParamMode::parse(params, 2)?;
                Command::JumpIfTrue(params[0], params[1])
            }
            6 => {
                let params = ParamMode::parse(params, 2)?;
                Command::JumpIfFalse(params[0], params[1])
            }
            7 => {
                let params = ParamMode::parse(params, 3)?;
                Command::LessThan(params[0], params[1], params[2])
            }
            8 => {
                let params = ParamMode::parse(params, 3)?;
                Command::Equals(params[0], params[1], params[2])
            }
            9 => {
                let params = ParamMode::parse(params, 1)?;
                Command::AdjustRelBase(params[0])
            }
            99 => Command::Exit,
            _ => bail!("ERROR: Unknown command id {}.", opc % 100),
        };

        Ok(cmd)
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum ParamMode {
    Position,  // 0
    Immediate, // 1
    Relative,  // 2
}

impl ParamMode {
    fn parse(opcode: DataType, count: u8) -> Result<Vec<ParamMode>> {
        let mut result = Vec::new();
        let mut opc = opcode;
        for _i in 0..count {
            let param_mode = match opc % 10 {
                0 => ParamMode::Position,
                1 => ParamMode::Immediate,
                2 => ParamMode::Relative,
                _ => bail!("ERROR: Unknown parameter mode {}", opc % 10),
            };
            result.push(param_mode);
            opc /= 10;
        }
        Ok(result)
    }
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

        let mut output = if self.input.len() > 0 {
            self.input[self.input.len() - 1]
        } else {
            0
        };

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

    fn rel_ip(&self, offset: DataType) -> usize {
        (self.rel_base as DataType + offset) as usize
    }

    /// Returns false if execution should be stopped or paused
    fn eval_cmd(&mut self, output: &mut DataType) -> Result<bool> {
        ensure!(self.is_running(), "ERROR: Program is not running.");
        // self.check_ip(self.ip, format!("Cannot read command opcode"));

        let cmd = Command::parse(self.prog[self.ip]).with_context(|| format!("ip={}", self.ip))?;

        self.log.println(format!(
            "  Command[{}:{}]: {:?}",
            self.ip, self.prog[self.ip], cmd
        ));

        match cmd {
            Command::Add(m1, m2, m3) => {
                let v1 = self.get_param_value(1, m1)?;
                let v2 = self.get_param_value(2, m2)?;

                self.set_param_value(3, m3, v1 + v2)?;
                self.ip += 3;
            }
            Command::Mul(m1, m2, m3) => {
                let v1 = self.get_param_value(1, m1)?;
                let v2 = self.get_param_value(2, m2)?;

                self.set_param_value(3, m3, v1 * v2)?;
                self.ip += 3;
            }
            Command::Read(m1) => {
                ensure!(self.input.len() > 0, "ERROR: Input buffer is empty.");

                let value = self.input.remove(0);

                self.set_param_value(1, m1, value)?;
                self.ip += 1;
            }
            Command::Write(m1) => {
                *output = self.get_param_value(1, m1)?;
                self.status = Status::Paused;
                self.ip += 1;
            }
            Command::JumpIfTrue(m1, m2) => {
                let v1 = self.get_param_value(1, m1)?;

                if v1 != 0 {
                    let v2 = self.get_param_value(2, m2)?;
                    self.ip = v2 as usize;
                    return Ok(true);
                }
            }
            Command::JumpIfFalse(m1, m2) => {
                let v1 = self.get_param_value(1, m1)?;

                if v1 == 0 {
                    let v2 = self.get_param_value(2, m2)?;
                    self.ip = v2 as usize;
                    return Ok(true);
                }
            }
            Command::LessThan(m1, m2, m3) => {
                let v1 = self.get_param_value(1, m1)?;
                let v2 = self.get_param_value(2, m2)?;

                self.set_param_value(3, m3, if v1 < v2 { 1 } else { 0 })?;
                self.ip += 3;
            }
            Command::Equals(m1, m2, m3) => {
                let v1 = self.get_param_value(1, m1)?;
                let v2 = self.get_param_value(2, m2)?;

                self.set_param_value(3, m3, if v1 == v2 { 1 } else { 0 })?;
                self.ip += 3;
            }
            Command::AdjustRelBase(m1) => {
                let v1 = self.get_param_value(1, m1)?;

                self.rel_base = self.rel_ip(v1);
                self.log.println(format!("    rel_base={}", self.rel_base));
                self.ip += 1;
            }
            Command::Exit => {
                self.status = Status::Halted;
            }
        }

        self.ip += 1;

        Ok(self.status == Status::Running)
    }

    // fn check_ip(&self, ip: usize, msg: String) -> Result<()> {
    //     ensure!(
    //         ip < self.prog.len(),
    //         "ERROR: {}. Instruction pointer {} is out of program bound length {}.",
    //         msg,
    //         ip,
    //         self.prog.len()
    //     );
    //     Ok(())
    // }

    fn check_and_extend(&mut self, ip: usize) {
        if ip >= self.prog.len() {
            self.log.println(format!(
                "    Extending program from {} to {} elements.",
                self.prog.len(),
                ip + 1
            ));
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
                self.log.println(format!("    in(pos): ip={}->{} value={}", ip, val_ip, self.prog[val_ip]));
                self.prog[val_ip]
            }
            ParamMode::Immediate => {
                self.log.println(format!("    in(imm): ip={} value={}", ip, self.prog[ip]));
                self.prog[ip]
            }
            ParamMode::Relative => {
                let val_ip = self.rel_ip(self.prog[ip]);
                self.check_and_extend(val_ip);
                self.log.println(format!("    in(rel): ip={}->{}(+{}) value={}", ip, val_ip, self.rel_base, self.prog[val_ip]));
                self.prog[val_ip]
            }
        };

        Ok(value)
    }

    fn set_param_value(
        &mut self,
        param_offset: usize,
        mode: ParamMode,
        value: DataType,
    ) -> Result<()> {
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
        self.log.println(format!("    out({:?}): ip={}->{} value={}", mode, ip, val_ip, value));

        Ok(())
    }
}
