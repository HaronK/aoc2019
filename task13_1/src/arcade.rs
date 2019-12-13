use crate::intcode_comp::*;
use crate::log::*;
use anyhow::{bail, ensure, Result};
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
        self.log.println("Arcade map start");

        self.comp.run()?;
        ensure!(
            self.comp.is_halted(),
            "Intcomp should finish in the Halt state"
        );

        let output = self.comp.get_output();
        ensure!(
            output.len() % 3 == 0,
            "Expected output to contain sequence of 3 elements but was {}",
            output.len()
        );

        self.log.println(format!("  Output: {:?}", output));

        for i in 0..(output.len() / 3) {
            let x = output[i * 3];
            let y = output[i * 3 + 1];
            let tile_id = output[i * 3 + 2];

            let tile = TileType::new(tile_id)?;

            if tile == TileType::Paddle {
                self.paddle_pos = (x, y);
            }

            self.log
                .println(format!("  Tile[{}, {}]: {}", x, y, tile_id));

            self.set_tile(x as usize, y as usize, tile);
        }

        if self.visualize {
            self.dump_screen();
        }

        self.log.println(format!(
            "Map size: [{}, {}]",
            self.screen[0].len(),
            self.screen.len()
        ));

        Ok(())
    }

    pub fn run(&mut self) -> Result<usize> {
        self.comp.set_mem(0, 2);
        self.comp.start();

        self.log.println("Arcade start");

        let mut score = 0;
        let delay = time::Duration::from_millis(5);

        self.comp.add_input(0);

        while !self.comp.is_halted() {
            self.comp.run()?;

            let output = self.comp.get_output();
            ensure!(
                output.len() % 3 == 0,
                "Expected output to contain sequence of 3 elements but was {}",
                output.len()
            );

            for i in 0..(output.len() / 3) {
                let x = output[i * 3];
                let y = output[i * 3 + 1];

                if x == -1 && y == 0 {
                    score = output[i * 3 + 2] as usize;
                } else {
                    let tile = TileType::new(output[i * 3 + 2])?;

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
            }

            if self.visualize {
                thread::sleep(delay);
                self.dump_screen();
                println!("          Score: {}", score);
            }
        }

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
