use crate::intcode_comp::*;
use crate::log::*;
use crate::utils::*;
use anyhow::{ensure, Result};
use std::collections::HashMap;

#[derive(Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct Robot<'l> {
    comp: IntcodeComp<'l>,
    grid: Vec<Vec<u8>>,
    start_offset: PointI,
    position: PointI,
    dir: Direction,
    log: &'l Log,
}

impl<'l> Robot<'l> {
    pub fn new(prog: Vec<DataType>, start_color: u8, log: &'l Log) -> Self {
        Self {
            comp: IntcodeComp::new(prog, log),
            grid: vec![vec![start_color]],
            start_offset: PointI::new(0, 0),
            position: PointI::new(0, 0),
            dir: Direction::Up,
            log,
        }
    }

    pub fn dump_grid(&self) {
        for row in &self.grid {
            for cell in row {
                self.log
                    .print(format!("{}", if *cell == 0 { "." } else { "#" }));
            }
            self.log.println("");
        }
    }

    pub fn run(&mut self) -> Result<usize> {
        let mut painted_panels = HashMap::new();
        let mut steps = 0;

        self.log.println("Robot start");

        while !self.comp.is_halted() {
            let cur_color = self.get_color();

            self.log.println(format!(
                "  Step. Position: {} Dir: {:?} Color: {}",
                self.position, self.dir, cur_color
            ));

            self.comp.add_input(cur_color as DataType);

            let new_color = self.comp.run()?;
            ensure!(
                new_color == 0 || new_color == 1,
                "Wrong color. Expected 0|1 but was {}",
                new_color
            );

            if self.comp.is_halted() {
                break;
            }

            self.log.println(format!("  New color: {}", new_color));

            self.set_color(new_color as u8);

            painted_panels.insert(self.position.clone(), 0);

            let turn = self.comp.run()?;
            ensure!(
                turn == 0 || turn == 1,
                "Wrong turn. Expected 0|1 but was {}",
                turn
            );

            self.make_turn(turn as u8);

            self.log
                .println(format!("  Turn: {} New direction: {:?}", turn, self.dir));

            self.do_move();

            steps += 1;
        }

        self.log.println(format!("Steps: {}", steps));

        Ok(painted_panels.len())
    }

    fn abs_position(&self) -> PointU {
        PointU::new(
            (self.position.x + self.start_offset.x) as usize,
            (self.position.y + self.start_offset.y) as usize,
        )
    }

    fn get_color(&self) -> u8 {
        let abs_pos = self.abs_position();

        self.grid[abs_pos.y][abs_pos.x]
    }

    fn set_color(&mut self, color: u8) {
        let abs_pos = self.abs_position();

        self.grid[abs_pos.y][abs_pos.x] = color;
    }

    fn move_up(&mut self) {
        if self.position.y + self.start_offset.y == 0 {
            self.grid.insert(0, vec![0; self.grid[0].len()]);
            self.start_offset.y += 1;
        }
        self.position.y -= 1;
    }

    fn move_down(&mut self) {
        if (self.position.y + self.start_offset.y) as usize == self.grid.len() - 1 {
            self.grid.push(vec![0; self.grid[0].len()]);
        }
        self.position.y += 1;
    }

    fn move_left(&mut self) {
        if self.position.x + self.start_offset.x == 0 {
            self.grid.iter_mut().for_each(|row| row.insert(0, 0));
            self.start_offset.x += 1;
        }
        self.position.x -= 1;
    }

    fn move_right(&mut self) {
        if (self.position.x + self.start_offset.x) as usize == self.grid[0].len() - 1 {
            self.grid.iter_mut().for_each(|row| row.push(0));
        }
        self.position.x += 1;
    }

    fn do_move(&mut self) {
        match self.dir {
            Direction::Up => self.move_up(),
            Direction::Down => self.move_down(),
            Direction::Left => self.move_left(),
            Direction::Right => self.move_right(),
        }
    }

    fn make_turn(&mut self, turn: u8) {
        self.dir = if turn == 0 {
            // turn left
            match self.dir {
                Direction::Up => Direction::Left,
                Direction::Down => Direction::Right,
                Direction::Left => Direction::Down,
                Direction::Right => Direction::Up,
            }
        } else {
            // turn right
            match self.dir {
                Direction::Up => Direction::Right,
                Direction::Down => Direction::Left,
                Direction::Left => Direction::Up,
                Direction::Right => Direction::Down,
            }
        };
    }
}
