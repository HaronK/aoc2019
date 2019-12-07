use crate::intcode_comp::*;
use crate::log::*;
use anyhow::{ensure, Result};

pub struct Amplifier<'a> {
    prog: Vec<i32>,
    comps: Vec<IntcodeComp<'a>>,
    log: &'a Log,
}

impl<'a> Amplifier<'a> {
    pub fn new(commands: &String, log: &'a Log) -> Result<Self> {
        let mut result = Self {
            prog: Vec::new(),
            comps: Vec::new(),
            log,
        };

        result.parse(commands)?;

        Ok(result)
    }

    pub fn run(&mut self, phase_settings: &Vec<i32>) -> Result<i32> {
        self.log.println(format!("Commands: {}", self.prog.len()));
        self.log.println(format!("Phases: {:?}", phase_settings));

        let steps = phase_settings.len();
        ensure!(steps > 0, "ERROR: No phase settings are set.");

        self.comps.clear();

        for phase in phase_settings {
            self.comps
                .push(IntcodeComp::new(self.prog.clone(), *phase, self.log));
        }

        let mut result = 0;

        let mut k = 0;
        while !self.comps[self.comps.len() - 1].is_halted() {
            self.log.println(format!("Iteration {}", k));
            k += 1;

            for i in 0..steps {
                self.log.println(format!("  Comp: {}", i));
                result = self.comps[i].run(result)?;
            }
        }

        Ok(result)
    }

    fn parse(&mut self, commands: &String) -> Result<()> {
        let cmd_str: Vec<&str> = commands.split(",").collect();
        for cmd in cmd_str {
            self.prog.push(cmd.parse()?);
        }
        Ok(())
    }
}
