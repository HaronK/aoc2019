use crate::intcode_comp::*;
use crate::log::*;
use anyhow::{bail, Result};
use std::io::Write;
use std::{thread, time};
use termion;

#[derive(PartialEq, Clone)]
pub enum TileType {
    Empty,  // 0
    Wall,   // 1
    Block,  // 2
    Paddle, // 3
    Ball,   // 4
}

impl TileType {
    fn new(value: DataType) -> Result<Self> {
        let res = match value {
            0 => Self::Empty,
            1 => Self::Wall,
            2 => Self::Block,
            3 => Self::Paddle,
            4 => Self::Ball,
            _ => bail!("Unknown tile type: {}", value),
        };
        Ok(res)
    }
}

pub struct Arcade<'l> {
    comp: IntcodeComp<'l>,
    screen: Vec<Vec<TileType>>,
    paddle_pos: (DataType, DataType),
    visualize: bool,
    log: &'l Log,
}

impl<'l> Arcade<'l> {
    pub fn new(prog: Vec<DataType>, visualize: bool, log: &'l Log) -> Self {
        Self {
            comp: IntcodeComp::new(prog, log),
            screen: vec![vec![TileType::Empty]],
            paddle_pos: (0, 0),
            visualize,
            log,
        }
    }

    pub fn get_tiles_by_id(&self, id: TileType) -> usize {
        let mut result = 0;
        for row in &self.screen {
            for cell in row {
                if *cell == id {
                    result += 1;
                }
            }
        }
        result
    }

    pub fn dump_screen(&self) {
        let mut buf = String::new();

        for row in &self.screen {
            for cell in row {
                buf += match cell {
                    TileType::Empty => "░",
                    TileType::Wall => "█",
                    TileType::Block => "▓",
                    TileType::Paddle => "▬",
                    TileType::Ball => "●",
                };
            }
            buf += "\n";
        }

        print!(
            "{}{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            buf
        );
        std::io::stdout().flush().unwrap();
    }

    pub fn build_map(&mut self) -> Result<()> {
        let mut steps = 0;

        self.log.println("Arcade map start");

        while !self.comp.is_halted() {
            let x = self.comp.run()?;

            if self.comp.is_halted() {
                break;
            }

            let y = self.comp.run()?;
            let tile_id = self.comp.run()?;
            let tile = TileType::new(tile_id)?;

            if tile == TileType::Paddle {
                self.paddle_pos = (x, y);
            }

            self.log
                .println(format!("  Tile[{}, {}]: {}", x, y, tile_id));

            self.set_tile(x as usize, y as usize, tile);

            steps += 1;
        }

        self.log.println(format!("Steps: {}", steps));

        Ok(())
    }

    pub fn run(&mut self) -> Result<usize> {
        let mut steps = 0;

        self.comp.set_mem(0, 2);
        self.comp.start();

        self.log.println("Arcade start");
        steps = 0;

        let mut score = 0;
        let delay = time::Duration::from_millis(5);

        self.comp.add_input(0);

        while !self.comp.is_halted() {

            let x = self.comp.run()?;
            let y = self.comp.run()?;
            let tile_id = self.comp.run()?;

            if x == -1 && y == 0 {
                score = tile_id as usize;
            } else {
                let tile = TileType::new(tile_id)?;

                if tile == TileType::Ball {
                    let ball_pos = (x, y);

                    let joystick = if ball_pos.0 < self.paddle_pos.0 {
                        -1
                    } else if ball_pos.0 > self.paddle_pos.0 {
                        1
                    } else {
                        0
                    };

                    self.comp.add_input(joystick);
                } else if tile == TileType::Paddle {
                    self.paddle_pos = (x, y);
                }

                self.set_tile(x as usize, y as usize, tile);
            }

            if self.visualize {
                thread::sleep(delay);
                self.dump_screen();
                println!("          Score: {}", score);
            }

            steps += 1;
        }

        self.log.println(format!("Steps: {}", steps));

        Ok(score)
    }

    fn set_tile(&mut self, x: usize, y: usize, tile_id: TileType) {
        self.extend_right(x);
        self.extend_down(y);

        self.screen[y][x] = tile_id;
    }

    fn extend_down(&mut self, y: usize) {
        if y >= self.screen.len() {
            let count = y - self.screen.len() + 1;
            for _ in 0..count {
                self.screen
                    .push(vec![TileType::Empty; self.screen[0].len()]);
            }
        }
    }

    fn extend_right(&mut self, x: usize) {
        let width = self.screen[0].len();
        if x >= width {
            let count = x - width + 1;
            self.screen.iter_mut().for_each(|row| {
                for _ in 0..count {
                    row.push(TileType::Empty);
                }
            });
        }
    }
}
