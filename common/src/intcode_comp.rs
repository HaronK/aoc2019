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
    fn parse(opc: DataType) -> Result<(Self, usize)> {
        let cmd_id = opc % 100;
        let params = opc / 100;
        let cmd = match cmd_id {
            1 => {
                let params = ParamMode::parse(params, 3)?;
                (Command::Add(params[0], params[1], params[2]), 3)
            }
            2 => {
                let params = ParamMode::parse(params, 3)?;
                (Command::Mul(params[0], params[1], params[2]), 3)
            }
            3 => {
                let params = ParamMode::parse(params, 1)?;
                (Command::Read(params[0]), 1)
            }
            4 => {
                let params = ParamMode::parse(params, 1)?;
                (Command::Write(params[0]), 1)
            }
            5 => {
                let params = ParamMode::parse(params, 2)?;
                (Command::JumpIfTrue(params[0], params[1]), 2)
            }
            6 => {
                let params = ParamMode::parse(params, 2)?;
                (Command::JumpIfFalse(params[0], params[1]), 2)
            }
            7 => {
                let params = ParamMode::parse(params, 3)?;
                (Command::LessThan(params[0], params[1], params[2]), 3)
            }
            8 => {
                let params = ParamMode::parse(params, 3)?;
                (Command::Equals(params[0], params[1], params[2]), 3)
            }
            9 => {
                let params = ParamMode::parse(params, 1)?;
                (Command::AdjustRelBase(params[0]), 1)
            }
            99 => (Command::Exit, 0),
            _ => bail!("Unknown command id {}.", opc % 100),
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
                _ => bail!("Unknown parameter mode {}", opc % 10),
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
    WaitForInput,
    Halted,
}

pub type DataType = i64;

pub struct IntcodeComp<'l> {
    prog: Vec<DataType>,
    prog_backup: Vec<DataType>,
    ip: usize,
    rel_base: usize,
    input: Vec<DataType>,
    output: Vec<DataType>,
    status: Status,
    log: &'l Log,
}

impl<'l> IntcodeComp<'l> {
    pub fn new(prog: Vec<DataType>, log: &'l Log) -> Self {
        log.println(format!("=> New comp. Size: {}", prog.len()));
        Self {
            prog: prog.clone(),
            prog_backup: prog,
            ip: 0,
            rel_base: 0,
            input: Vec::new(),
            output: Vec::new(),
            status: Status::Running,
            log,
        }
    }

    pub fn load_prog(&mut self, data: &str) -> Result<()> {
        let cmd_str: Vec<&str> = data.split(',').collect();

        self.prog_backup.clear();

        for cmd in cmd_str {
            self.prog_backup.push(cmd.parse()?);
        }

        self.prog = self.prog_backup.clone();

        self.log.println(format!("=> Load prog. Size: {}", self.prog.len()));

        Ok(())
    }

    pub fn start(&mut self) {
        self.status = Status::Running;
    }

    pub fn restart(&mut self) {
        self.status = Status::Running;
        self.ip = 0;
    }

    pub fn reset(&mut self) {
        self.prog = self.prog_backup.clone();
        self.restart();
    }

    pub fn is_running(&self) -> bool {
        self.status == Status::Running
    }

    pub fn is_halted(&self) -> bool {
        self.status == Status::Halted
    }

    pub fn get_ip(&self) -> usize {
        self.ip
    }

    pub fn add_input(&mut self, input: DataType) {
        self.input.push(input);
    }

    pub fn add_input_vec(&mut self, mut input: &mut Vec<DataType>) {
        self.input.append(&mut input);
    }

    pub fn get_output(&mut self) -> Vec<DataType> {
        let output = self.output.clone();
        self.output.clear();
        output
    }

    /// Run whole program and return outputs
    pub fn exec(&mut self) -> Result<()> {
        while self.is_running() {
            self.run()?;
        }

        ensure!(
            self.status == Status::Halted,
            "Program was not finished properly. Status: {:?}",
            self.status
        );

        Ok(())
    }

    pub fn set_mem(&mut self, addr: usize, value: DataType) {
        self.check_and_extend(addr);
        self.prog[addr] = value;
    }

    /// Run computer until next input
    pub fn run(&mut self) -> Result<()> {
        ensure!(
            !self.is_halted(),
            "Program was halted. ip={} status={:?}.",
            self.ip,
            self.status
        );

        self.status = Status::Running;

        self.log.println(format!("=> Input: {:?}", self.input));

        while self.eval_cmd()? {}

        if !self.input.is_empty() {
            self.log.println(format!(
                "WARNING: Input buffer was not consumed completely. Remaining values[{}]: {:?}.",
                self.input.len(), self.input
            ));
        }

        self.log.println(format!("=> Status: {:?}", self.status));

        Ok(())
    }

    fn rel_ip(&self, offset: DataType) -> usize {
        (self.rel_base as DataType + offset) as usize
    }

    /// Returns false if execution should be stopped or paused
    fn eval_cmd(&mut self) -> Result<bool> {
        ensure!(self.is_running(), "Program is not running.");

        let (cmd, params_count) =
            Command::parse(self.prog[self.ip]).with_context(|| format!("ip={}", self.ip))?;

        self.log.print(format!("[{:4}] ", self.ip));

        match cmd {
            Command::Add(m1, m2, m3) => {
                self.log.print("ADD");
                let v1 = self.get_param_value(1, m1)?;
                let v2 = self.get_param_value(2, m2)?;

                self.set_param_value(3, m3, v1 + v2)?;
            }
            Command::Mul(m1, m2, m3) => {
                self.log.print("MUL");
                let v1 = self.get_param_value(1, m1)?;
                let v2 = self.get_param_value(2, m2)?;

                self.set_param_value(3, m3, v1 * v2)?;
            }
            Command::Read(m1) => {
                self.log.print("GET");
                if self.input.is_empty() {
                    self.status = Status::WaitForInput;
                    self.log.println("    Waiting for input");
                    return Ok(false);
                }

                let value = self.input.remove(0);

                self.set_param_value(1, m1, value)?;
            }
            Command::Write(m1) => {
                self.log.print("SET");
                let value = self.get_param_value(1, m1)?;
                self.output.push(value);
            }
            Command::JumpIfTrue(m1, m2) => {
                self.log.print("JIT");
                let v1 = self.get_param_value(1, m1)?;

                if v1 != 0 {
                    let v2 = self.get_param_value(2, m2)?;
                    self.ip = v2 as usize;
                    self.log.println("");
                    return Ok(true);
                }
            }
            Command::JumpIfFalse(m1, m2) => {
                self.log.print("JIF");
                let v1 = self.get_param_value(1, m1)?;

                if v1 == 0 {
                    let v2 = self.get_param_value(2, m2)?;
                    self.ip = v2 as usize;
                    self.log.println("");
                    return Ok(true);
                }
            }
            Command::LessThan(m1, m2, m3) => {
                self.log.print(" LT");
                let v1 = self.get_param_value(1, m1)?;
                let v2 = self.get_param_value(2, m2)?;

                self.set_param_value(3, m3, if v1 < v2 { 1 } else { 0 })?;
            }
            Command::Equals(m1, m2, m3) => {
                self.log.print(" EQ");
                let v1 = self.get_param_value(1, m1)?;
                let v2 = self.get_param_value(2, m2)?;

                self.set_param_value(3, m3, if v1 == v2 { 1 } else { 0 })?;
            }
            Command::AdjustRelBase(m1) => {
                self.log.print("ARB");
                let v1 = self.get_param_value(1, m1)?;

                self.rel_base = self.rel_ip(v1);
                self.log.print(format!("->{}", self.rel_base));
            }
            Command::Exit => {
                self.log.print("EXIT");
                self.status = Status::Halted;
            }
        }
        self.log.println("");

        self.ip += params_count + 1;

        Ok(self.status == Status::Running)
    }

    fn check_and_extend(&mut self, ip: usize) {
        if ip >= self.prog.len() {
            // self.log.println(format!(
            //     "    Extending program from {} to {} elements.",
            //     self.prog.len(),
            //     ip + 1
            // ));
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
                self.log.print(format!(
                    " p[{}]->{}",
                    val_ip, self.prog[val_ip]
                ));
                self.prog[val_ip]
            }
            ParamMode::Immediate => {
                self.log.print(format!(
                    " i[{}]",
                    self.prog[ip]
                ));
                self.prog[ip]
            }
            ParamMode::Relative => {
                let val_ip = self.rel_ip(self.prog[ip]);
                self.check_and_extend(val_ip);
                self.log.print(format!(
                    " r[{}+{}]->{}",
                    self.prog[ip], self.rel_base, self.prog[val_ip]
                ));
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
            "Wrong destination parameter. Expected Position or Relative but was {:?}.",
            mode
        );

        let ip = self.ip + param_offset;

        self.check_and_extend(ip);

        let val_ip = if mode == ParamMode::Relative {
            self.log.print(format!(" r[{}+{}]<-", self.prog[ip], self.rel_base));
            self.rel_ip(self.prog[ip])
        } else {
            self.log.print(format!(" p[{}]<-", self.prog[ip]));
            self.prog[ip] as usize
        };
        self.log.print(format!("{}", value));

        self.check_and_extend(val_ip as usize);

        self.prog[val_ip as usize] = value;

        Ok(())
    }
}
