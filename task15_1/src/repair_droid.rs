use anyhow::{bail, ensure, Result};
use common::dynamic_map::*;
use common::intcode_comp::*;
use common::log::*;
use common::point::*;
use pathfinding::prelude::astar;
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

impl CellDirections {
    fn new(back_dir: Direction, directions: Vec<Direction>) -> Self {
        Self {
            back_dir,
            directions,
        }
    }
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

pub struct RepairDroid<'l> {
    comp: IntcodeComp<'l>,
    map: DynamicMap<Cell>,
    visualize: bool,
    oxygen_pos: PointU,
    log: &'l Log,
}

impl<'l> RepairDroid<'l> {
    pub fn new(prog: &str, log: &'l Log) -> Result<Self> {
        let mut comp = IntcodeComp::new(Vec::new(), log);
        comp.load_prog(prog)?;
        let res = Self {
            comp,
            map: DynamicMap::default(),
            visualize: false,
            oxygen_pos: PointU::default(),
            log,
        };
        Ok(res)
    }

    fn neighbors(&self, pos: &PointU) -> Vec<PointU> {
        let mut result = Vec::new();

        if self.map.get_cell_by_xy(pos.x, pos.y - 1) != Cell::Wall {
            result.push(PointU::new(pos.x, pos.y - 1));
        }

        if self.map.get_cell_by_xy(pos.x, pos.y + 1) != Cell::Wall {
            result.push(PointU::new(pos.x, pos.y + 1));
        }

        if self.map.get_cell_by_xy(pos.x - 1, pos.y) != Cell::Wall {
            result.push(PointU::new(pos.x - 1, pos.y));
        }

        if self.map.get_cell_by_xy(pos.x + 1, pos.y) != Cell::Wall {
            result.push(PointU::new(pos.x + 1, pos.y));
        }

        result
    }

    pub fn distance_to_oxygen(&mut self, visualize: bool) -> Result<isize> {
        self.visualize = visualize;

        let start_pos = self.map.abs_position().clone();
        let (path, dist) = self.distance_between(&start_pos, &self.oxygen_pos.clone())?;

        self.show_with_path(&mut io::stdout(), &path)?;
        // println!("Start pos: {:?}", start_pos);
        // println!("End pos: {:?}", end_pos);
        // println!("Path[{}]: {:?}", path.len(), path);
        Ok(dist)
    }

    pub fn max_dist_from_oxygen2(&self) -> Result<isize> {
        let mut max_dist = 0;
        let mut edge_cells = vec![self.oxygen_pos.clone()];
        let mut edge_cells2 = Vec::new();
        let mut processed_cells = vec![self.oxygen_pos.clone()];

        while !edge_cells.is_empty() {
            while let Some(pos) = edge_cells.pop() {
                let neighbors = self.neighbors(&pos);
                for n in neighbors {
                    if !processed_cells.iter().any(|pos| *pos == n) {
                        edge_cells2.push(n.clone());
                        processed_cells.push(n);
                    }
                }
            }
            edge_cells = edge_cells2.clone();
            edge_cells2.clear();
            max_dist += 1;
        }

        Ok(max_dist)
    }

    pub fn max_dist_from_oxygen(&mut self, visualize: bool) -> Result<isize> {
        self.visualize = visualize;

        let mut max_dist = 0;
        let mut max_path = Vec::new();
        let start_pos = self.oxygen_pos.clone();

        let (width, height) = self.map.size();
        for i in 0..height {
            for j in 0..width {
                let cell = self.map.get_cell_by_xy(j, i);
                if cell == Cell::Empty {
                    let (path, dist) = self.distance_between(&start_pos, &PointU::new(j, i))?;

                    if max_dist < dist {
                        max_dist = dist;
                        max_path = path;
                    }
                }
            }
        }

        self.show_with_path(&mut io::stdout(), &max_path)?;
        // println!("Start pos: {:?}", start_pos);
        // println!("End pos: {:?}", end_pos);
        // println!("Path[{}]: {:?}", path.len(), path);

        Ok(max_dist + 1)
    }

    fn distance_between(
        &mut self,
        start_pos: &PointU,
        end_pos: &PointU,
    ) -> Result<(Vec<PointU>, isize)> {
        Ok(astar(
            start_pos,
            |pos| self.neighbors(&pos).into_iter().map(|p| (p, 1)),
            |pos| {
                (pos.x as isize - end_pos.x as isize).abs()
                    + (pos.y as isize - end_pos.y as isize).abs()
            },
            |pos| *end_pos == *pos,
        )
        .unwrap_or((Vec::new(), 0)))
    }

    pub fn open_map(&mut self, visualize: bool) -> Result<()> {
        let mut stdout = io::stdout();
        let delay = time::Duration::from_millis(10);

        self.visualize = visualize;

        self.begin_show(&mut stdout)?;

        let mut route = Vec::new();
        let mut back_dir = Direction::North;
        let mut oxygen_pos = PointI::default();

        loop {
            self.show(&mut stdout)?;

            let mut dirs = self.available_directions()?;

            // Are there yet unknown directions?
            if let Some(dir) = dirs.pop() {
                let (cell, _) = self.do_move(&dir)?;

                if cell == Cell::Oxygen {
                    oxygen_pos = self.map.position();
                }

                let cell_dir = CellDirections::new(back_dir.clone(), dirs);
                back_dir = dir.opposite();

                route.push(cell_dir);
            } else {
                // Cannot move anywhere? Go back and try from there
                self.do_move(&back_dir)?;

                while let Some(cell_dir) = &mut route.pop() {
                    if let Some(dir) = cell_dir.directions.pop() {
                        let (cell, _) = self.do_move(&dir)?;

                        if cell == Cell::Oxygen {
                            oxygen_pos = self.map.position();
                        }

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

        self.oxygen_pos = self.map.get_abs_position(&oxygen_pos);

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
        self.show_with_path(f, &Vec::new())
    }

    fn show_with_path(&self, f: &mut dyn io::Write, path: &[PointU]) -> Result<()> {
        if self.visualize {
            write!(f, "{}", termion::clear::All)?;

            self.map.show_with_path(f, path)?;

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
