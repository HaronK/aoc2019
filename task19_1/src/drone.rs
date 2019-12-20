use anyhow::{ensure, Result};
use common::intcode_comp::*;
use common::log::*;

pub struct Drone<'l> {
    comp: IntcodeComp<'l>,
}

impl<'l> Drone<'l> {
    pub fn new(prog: &str, log: &'l Log) -> Result<Self> {
        let mut comp = IntcodeComp::new(Vec::new(), log);
        comp.load_prog(prog)?;
        let res = Drone {
            comp,
        };
        Ok(res)
    }

    pub fn check_area(&mut self) -> Result<usize> {
        let mut result = 0;
        let xsize = 50;
        let ysize = 50;

        for i in 0..ysize {
            for j in 0..xsize {
                self.comp.reset();

                self.comp.add_input(i as DataType);
                self.comp.add_input(j as DataType);

                self.comp.run()?;

                let output = self.comp.get_output();
                ensure!(output.len() == 1, "Expected 1 output value but was {}", output.len());

                let ch = if output[0] == 1 {
                    result += 1;
                    '#'
                } else {
                    '.'
                };
                print!("{}", ch);
            }
            println!();
        }

        Ok(result)
    }
}
