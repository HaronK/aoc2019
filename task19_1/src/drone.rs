use anyhow::{ensure, Result};
use common::intcode_comp::*;
use common::log::*;

pub trait Grid {
    fn get_value(&mut self, x: usize, y: usize) -> Result<DataType>;
}

#[derive(Clone)]
pub struct Drone<'l> {
    comp: IntcodeComp<'l>,
}

impl<'l> Drone<'l> {
    pub fn new(prog: &str, log: &'l Log) -> Result<Self> {
        let mut comp = IntcodeComp::new(Vec::new(), log);
        comp.load_prog(prog)?;
        let res = Drone { comp };
        Ok(res)
    }

    pub fn check_area(&mut self) -> Result<usize> {
        let mut result = 0;
        let xsize = 50;
        let ysize = 50;

        for y in 0..ysize {
            for x in 0..xsize {
                self.comp.reset();

                self.comp.add_input(x as DataType);
                self.comp.add_input(y as DataType);

                self.comp.run()?;

                let output = self.comp.get_output();
                ensure!(
                    output.len() == 1,
                    "Expected 1 output value but was {}",
                    output.len()
                );

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

    pub fn find_santa(&mut self) -> Result<usize> {
        self.find_santa_common(100, 3, 7, &mut self.clone())
    }

    pub fn find_santa_common(&mut self, size: usize, off_x: usize, off_y: usize, grid: &mut dyn Grid) -> Result<usize> {
        let mut result = 0;
        let mut start_pos = off_x;
        let mut rows = Vec::new();
        let mut ray_width = 1;

        for _i in 0..off_y {
            rows.push((0, 1));
        }

        println!("#");
        println!(".");

        let mut y = off_y;
        loop {
            // for _j in 0..start_pos {
            //     print!(".");
            // }

            let mut found_begin = false;
            let mut found_end = false;
            let mut begin_pos = 0;
            let mut end_pos = 0;

            let mut x = start_pos;
            while !found_end {
                let scan_value = grid.get_value(x, y)?;

                let _ch = if scan_value == 1 {
                    if !found_begin {
                        found_begin = true;
                        begin_pos = x;

                        for _k in 0..ray_width - 1 {
                            // print!("#");
                            x += 1;
                        }
                    }
                    '#'
                } else {
                    if found_begin && !found_end {
                        found_end = true;
                        end_pos = x;
                        ray_width = end_pos - begin_pos;

                        if ray_width > 1 {
                            ray_width -= 1;
                        }
                    }
                    '.'
                };
                // print!("{}", ch);
                x += 1;
            }

            // println!("{:3} [{:4}, {:4}]: {}", y, begin_pos, end_pos, end_pos - begin_pos);

            rows.push((begin_pos, end_pos));

            if y > size && end_pos - begin_pos >= size {
                let y1 = y - size + 1;
                let (_begin_pos1, end_pos1) = &rows[y1];

                // println!("  {:3} [{:4}, {:4}] {}", y1, begin_pos1, end_pos1, *end_pos1 as isize - begin_pos as isize);

                if begin_pos + size <= *end_pos1 {
                    result = begin_pos * 10000 + y1;

                    // for y2 in y1..y + 1 {
                    //     for _x2 in 0..rows[y2].0 {
                    //         print!(".");
                    //     }

                    //     for _x2 in rows[y2].0..rows[y2].1 {
                    //         print!("#");
                    //     }

                    //     println!();
                    // }

                    break;
                }
            }

            y += 1;
            start_pos = begin_pos;
        }

        Ok(result)
    }
}

impl Grid for Drone<'_> {
    fn get_value(&mut self, x: usize, y: usize) -> Result<DataType> {
        self.comp.reset();

        self.comp.add_input(x as DataType);
        self.comp.add_input(y as DataType);

        self.comp.run()?;

        let output = self.comp.get_output();
        ensure!(
            output.len() == 1,
            "Expected 1 output value but was {}",
            output.len()
        );

        Ok(output[0])
    }
}
