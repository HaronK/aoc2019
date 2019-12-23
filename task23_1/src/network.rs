use anyhow::{bail, ensure, Result};
use common::intcode_comp::*;
use common::log::*;

pub struct Network<'l> {
    comps: Vec<IntcodeComp<'l>>,
    is_running: bool,
}

impl<'l> Network<'l> {
    pub fn new(prog: &str, count: usize, log: &'l Log) -> Result<Self> {
        let mut comps = Vec::new();

        for i in 0..count {
            let mut comp = IntcodeComp::new(Vec::new(), log);
            comp.load_prog(prog)?;
            comp.add_input(i as DataType);
            comp.add_input(-1);
            comps.push(comp);
        }

        Ok(Self { comps, is_running: true })
    }

    fn run_iter(&mut self) -> Result<Option<DataType>> {
        let mut nic = vec![Vec::new(); self.comps.len()];
        let comps_count = self.comps.len();

        self.is_running = false;

        for comp in &mut self.comps {
            if comp.is_halted() {
                continue;
            }

            self.is_running = true;

            comp.run()?;

            let mut output = comp.get_output();

            while !output.is_empty() {
                let addr = output.remove(0) as usize;
                let x = output.remove(0);
                let y = output.remove(0);

                if addr == 255 {
                    return Ok(Some(y));
                }

                ensure!(addr < comps_count, "Expected addr to be [0, {}] but was {}", self.comps.len() - 1, addr);

                nic[addr].push(x);
                nic[addr].push(y);
            }
        }

        let mut i = 0;
        for mut input in &mut nic {
            if !input.is_empty() {
                self.comps[i].add_input_vec(&mut input);
            } else {
                self.comps[i].add_input(-1);
            }
            i += 1;
        }

        Ok(None)
    }

    pub fn run(&mut self) -> Result<DataType> {
        while self.is_running {
            if let Some(y) = self.run_iter()? {
                return Ok(y);
            }
        }

        bail!("Didn't receive data in the address 255.");
    }
}
