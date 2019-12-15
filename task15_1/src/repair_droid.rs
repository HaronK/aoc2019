use anyhow::{bail, ensure, Result};
use common::dynamic_map::*;
use common::intcode_comp::*;
use common::log::*;
use std::fmt;
use std::io;
use std::io::Write;
use std::{thread, time};
use termion;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

#[derive(Clone, PartialEq)]
enum Cell {
    Undefined,
    Empty,
    Wall,
    Oxygen,
}

impl CellDisplay for Cell {
    fn display(&self) -> char {
        match self {
            Self::Undefined => ' ',
            Self::Empty => '░',
            Self::Wall => '█',
            Self::Oxygen => '♻',
        }
    }

    fn start(&self) -> Option<char> {
        Some('★')
    }

    fn current(&self) -> Option<char> {
        Some('⛑')
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::Undefined
    }
}

struct CellDirections {
    back_dir: Direction,
    directions: Vec<Direction>,
}

impl fmt::Debug for CellDirections {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<back: {:?}, dirs: {:?}>",
            self.back_dir, self.directions
        )
    }
}

impl CellDirections {
    fn new(back_dir: Direction, directions: Vec<Direction>) -> Self {
        Self {
            back_dir,
            directions,
        }
    }
}

pub struct RepairDroid<'l> {
    comp: IntcodeComp<'l>,
    map: DynamicMap<Cell>,
    visualize: bool,
    log: &'l Log,
}

impl<'l> RepairDroid<'l> {
    pub fn new(prog: &str, log: &'l Log) -> Result<Self> {
        let mut comp = IntcodeComp::new(Vec::new(), log);
        comp.load_prog(prog)?;
        let res = Self {
            comp,
            map: DynamicMap::new(),
            visualize: false,
            log,
        };
        Ok(res)
    }

    pub fn open_map(&mut self, visualize: bool) -> Result<()> {
        let mut back_dir = Direction::North;
        let mut route = Vec::new();
        let mut stdout = io::stdout();
        let delay = time::Duration::from_millis(40);

        self.visualize = visualize;

        self.begin_show(&mut stdout)?;

        loop {
            self.show(&mut stdout)?;

            let mut dirs = self.available_directions()?;

            // Are there yet unknown directions?
            if let Some(dir) = dirs.pop() {
                let cell_dir = CellDirections::new(back_dir.clone(), dirs);
                self.do_move(&dir)?;
                back_dir = dir.opposite();
                route.push(cell_dir);
            } else {
                // Cannot move anywhere? Go back and try from there
                self.do_move(&back_dir)?;

                while let Some(cell_dir) = &mut route.pop() {
                    if let Some(dir) = cell_dir.directions.pop() {
                        self.do_move(&dir)?;
                        back_dir = dir.opposite();
                        route.push(CellDirections::new(
                            cell_dir.back_dir.clone(),
                            cell_dir.directions.clone(),
                        ));
                        break;
                    } else {
                        // If there is no directions from the current cell then go back and try previous
                        self.do_move(&cell_dir.back_dir)?;
                    }
                }

                // Route is empty? We have discovered everything
                if route.is_empty() {
                    break;
                }
            }

            if self.visualize {
                thread::sleep(delay);
            }
        }

        self.end_show(&mut stdout)?;

        Ok(())
    }

    /// Get directions where it's we didn't go before from the current position on the map.
    fn available_directions(&mut self) -> Result<Vec<Direction>> {
        let mut res = Vec::new();
        let all_directions = vec![
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ];

        for dir in all_directions {
            let (new_cell, old_cell) = self.do_move(&dir)?;

            if new_cell != Cell::Wall {
                self.do_move(&dir.opposite())?;
                if old_cell == Cell::Undefined {
                    res.push(dir);
                }
            }
        }

        Ok(res)
    }

    pub fn interactive(&mut self) -> Result<()> {
        self.visualize = true;
        let mut stdout = io::stdout().into_raw_mode()?;
        let mut stdin = io::stdin().keys();

        self.begin_show(&mut stdout)?;

        self.show(&mut stdout)?;
        stdout.lock().flush()?;

        loop {
            // Read input (if any)
            let input = stdin.next();

            // If a key was pressed
            if let Some(c) = input {
                let (arrow, dir) = match c? {
                    Key::Up => (true, Direction::North),
                    Key::Down => (true, Direction::South),
                    Key::Left => (true, Direction::West),
                    Key::Right => (true, Direction::East),
                    Key::Esc => {
                        break;
                    }
                    _ => (false, Direction::North),
                };

                if arrow {
                    self.do_move(&dir)?;
                    self.show(&mut stdout)?;
                    stdout.lock().flush()?;
                }
            }
        }

        self.end_show(&mut stdout)?;

        Ok(())
    }

    fn begin_show(&self, f: &mut dyn io::Write) -> Result<()> {
        if self.visualize {
            write!(f, "{}", termion::cursor::Hide)?;
        }
        Ok(())
    }

    fn show(&self, f: &mut dyn io::Write) -> Result<()> {
        if self.visualize {
            write!(f, "{}", termion::clear::All)?;

            self.map.show(f)?;

            // let (_, ys) = self.map.size();
            // write!(f, "{}Current cell: '{}'\n", termion::cursor::Goto(1, ys as u16 + 1), self.map.get_cell().display())?;
            // write!(f, "{}Arrows to move, Esc to exit.", termion::cursor::Goto(1, ys as u16 + 2))?;
            // println!();
        }
        Ok(())
    }

    fn end_show(&self, f: &mut dyn io::Write) -> Result<()> {
        if self.visualize {
            write!(f, "{}", termion::cursor::Show)?;
        }
        Ok(())
    }

    fn dir_to_code(dir: &Direction) -> DataType {
        match dir {
            Direction::North => 1,
            Direction::South => 2,
            Direction::West => 3,
            Direction::East => 4,
        }
    }

    /// Returns new and old cell values
    fn do_move(&mut self, dir: &Direction) -> Result<(Cell, Cell)> {
        ensure!(!self.comp.is_halted(), "IntCode is halted");

        self.comp.add_input(Self::dir_to_code(&dir));

        self.comp.run()?;
        let output = self.comp.get_output();
        ensure!(
            output.len() == 1,
            "Expected single output but was {}",
            output.len()
        );

        let old_cell = self.map.do_move(dir);

        let new_cell = match output[0] {
            0 => Cell::Wall,
            1 => Cell::Empty,
            2 => Cell::Oxygen,
            _ => bail!("Unknown status code: {}", output[0]),
        };

        self.map.set_cell(new_cell.clone());

        if new_cell == Cell::Wall {
            self.map.do_move(&dir.opposite());
        }

        self.map.set_cell(new_cell.clone());

        Ok((new_cell, old_cell))
    }
}
