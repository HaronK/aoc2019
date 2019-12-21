use anyhow::Result;
use common::intcode_comp::*;
use common::log::*;

pub struct SpringDroid<'l> {
    comp: IntcodeComp<'l>,
}

impl<'l> SpringDroid<'l> {
    pub fn new(prog: &str, log: &'l Log) -> Result<Self> {
        let mut comp = IntcodeComp::new(Vec::new(), log);
        comp.load_prog(prog)?;
        let res = Self { comp };
        Ok(res)
    }

    pub fn move_droid(&mut self, commands: &[&str]) -> Result<DataType> {
        for cmd in commands {
            self.comp.add_input_vec(&mut Self::str2input(cmd));
        }

        self.comp.exec()?;

        let output = self.comp.get_output();
        let output_str = Self::output2str(&output);
        let output_vec: Vec<&str> = output_str.lines().collect();

        if output_vec.len() < 6 || output_vec[5] != "Didn't make it across:" {
            println!("Output:\n{}", output_str);
        }

        // for c in output {
        //     if c == 10 {
        //         println!();
        //     } else {
        //         print!("{}", c as u8 as char);
        //     }
        // }

        Ok(output[output.len() - 1])
    }

    fn str2input(data: &str) -> Vec<DataType> {
        let mut result: Vec<DataType> = data.chars().map(|c| c as u8 as DataType).collect();

        result.push(10);

        result
    }

    fn output2str(data: &[DataType]) -> String {
        let mut res = String::new();

        for v in data {
            if *v == 10 {
                res.push('\n');
            } else {
                res.push(*v as u8 as char);
            }
        }

        res
    }
}
