use anyhow::{ensure, Result};
use common::dynamic_map::*;
use common::intcode_comp::*;
use common::log::*;
use common::point::*;
use std::io;

#[derive(Clone, PartialEq)]
struct Cell(char);

impl CellDisplay for Cell {
    fn display(&self) -> char {
        self.0
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self('.')
    }
}

impl From<DataType> for Cell {
    fn from(d: DataType) -> Self {
        Self(d as u8 as char)
    }
}

pub struct Robot<'l> {
    comp: IntcodeComp<'l>,
    map: DynamicMap<Cell>,
}

impl<'l> Robot<'l> {
    pub fn new(prog: &str, log: &'l Log) -> Result<Self> {
        let mut comp = IntcodeComp::new(Vec::new(), log);
        comp.load_prog(prog)?;
        let res = Self {
            comp,
            map: DynamicMap::new(),
        };
        Ok(res)
    }

    pub fn camera_scan(&mut self) -> Result<()> {
        self.comp.exec()?;
        ensure!(self.comp.is_halted(), "IntCode should be halted");

        let output = self.comp.get_output();

        for ch in output {
            // println!("ch: {}", ch);
            if ch == 10 {
                // println!("  south front");
                self.map.move_south_front();
            } else {
                // let abs_pos = self.map.abs_position();
                // let size = self.map.size();
                // println!("  pos: {:?} size: {:?}", abs_pos, size);
                self.map.set_cell(ch.into());
                self.map.do_move(&Direction::East);
            }
        }

        Ok(())
    }

    pub fn get_intersections(&self) -> Vec<PointU> {
        let mut result = Vec::new();
        let size = self.map.size();
        let scaffold = Cell('#');

        for i in 1..size.1 - 1 {
            for j in 1..size.0 - 1 {
                if self.map.get_cell_by_xy(j, i) == scaffold
                    && self.map.get_cell_by_xy(j, i - 1) == scaffold
                    && self.map.get_cell_by_xy(j, i + 1) == scaffold
                    && self.map.get_cell_by_xy(j - 1, i) == scaffold
                    && self.map.get_cell_by_xy(j + 1, i) == scaffold
                {
                    result.push(PointU::new(j, i));
                }
            }
        }

        result
    }

    pub fn wake_up(&mut self) {
        self.comp.set_mem(0, 2);
        self.comp.start();
    }

    pub fn move_robot(
        &mut self,
        routine: &str,
        func_a: &str,
        func_b: &str,
        func_c: &str,
    ) -> Result<DataType> {
        self.comp.add_input_vec(&mut Self::str2input(routine));
        self.comp.add_input_vec(&mut Self::str2input(func_a));
        self.comp.add_input_vec(&mut Self::str2input(func_b));
        self.comp.add_input_vec(&mut Self::str2input(func_c));
        self.comp.add_input_vec(&mut Self::str2input("n"));

        self.comp.exec()?;

        let output = self.comp.get_output();

        // println!("Output: {:?}", output);

        // for c in output {
        //     if c == 10 {
        //         println!();
        //     } else {
        //         print!("{}", c as u8 as char);
        //     }
        // }

        Ok(output[output.len() - 1])
    }

    pub fn show(&self) -> Result<()> {
        self.map.show(&mut io::stdout())
    }

    fn str2input(data: &str) -> Vec<DataType> {
        let mut result: Vec<DataType> = data.chars().map(|c| c as u8 as DataType).collect();

        result.push(10);

        result
    }
}
