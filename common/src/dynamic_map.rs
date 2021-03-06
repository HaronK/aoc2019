use crate::point::*;
use anyhow::Result;
use std::io::Write;
use termion;
use termion::color;

#[derive(Clone, Debug, PartialEq)]
pub enum Direction {
    North,
    South,
    West,
    East,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }
}

pub trait CellDisplay {
    fn display(&self) -> char;

    fn start(&self) -> Option<char> {
        None
    }

    fn current(&self) -> Option<char> {
        None
    }
}

#[derive(Default)]
pub struct DynamicMap<T> {
    map: Vec<Vec<T>>,
    start_offset: PointU,
    position: PointI,
}

impl<T: Default> DynamicMap<T> {
    pub fn new() -> Self {
        Self {
            map: vec![vec![T::default()]],
            start_offset: PointU::default(),
            position: PointI::default(),
        }
    }
}

impl<T> DynamicMap<T> {
    pub fn size(&self) -> (usize, usize) {
        (self.map[0].len(), self.map.len())
    }

    pub fn position(&self) -> PointI {
        self.position.clone()
    }

    pub fn to_start(&mut self) {
        self.position = PointI::default();
    }

    pub fn offset(&self) -> PointU {
        self.start_offset.clone()
    }

    pub fn abs_position(&self) -> PointU {
        self.get_abs_position(&self.position())
    }

    pub fn get_abs_position(&self, point: &PointI) -> PointU {
        PointU::new(
            (point.x + self.start_offset.x as isize) as usize,
            (point.y + self.start_offset.y as isize) as usize,
        )
    }

    pub fn set_cell(&mut self, value: T) {
        let abs_pos = self.abs_position();

        self.map[abs_pos.y][abs_pos.x] = value;
    }
}

impl<T: Clone> DynamicMap<T> {
    pub fn get_cell(&self) -> T {
        let abs_pos = self.abs_position();

        self.map[abs_pos.y][abs_pos.x].clone()
    }
}

impl<T: Clone + Default> DynamicMap<T> {
    pub fn get_cell_by_xy(&self, x: usize, y: usize) -> T {
        self.map[y][x].clone()
    }

    pub fn get_cell_dir(&mut self, dir: &Direction) -> T {
        self.do_move(dir);
        let cell = self.get_cell();
        self.do_move(&dir.opposite());
        cell
    }

    pub fn set_cell_dir(&mut self, dir: &Direction, value: T) {
        self.do_move(dir);
        self.set_cell(value);
        self.do_move(&dir.opposite());
    }

    pub fn do_move(&mut self, dir: &Direction) -> T {
        match dir {
            Direction::North => self.move_north(),
            Direction::South => self.move_south(),
            Direction::West => self.move_west(),
            Direction::East => self.move_east(),
        }
    }

    pub fn move_south_front(&mut self) {
        self.move_south();
        self.position.x = -(self.start_offset.x as isize);
    }

    fn move_north(&mut self) -> T {
        self.position.y -= 1;

        if self.position.y.abs() > self.start_offset.y as isize {
            self.map.insert(0, vec![T::default(); self.map[0].len()]);
            self.start_offset.y += 1;

            return T::default();
        }

        self.get_cell()
    }

    fn move_south(&mut self) -> T {
        self.position.y += 1;

        if (self.position.y + self.start_offset.y as isize) as usize == self.map.len() {
            self.map.push(vec![T::default(); self.map[0].len()]);

            return T::default();
        }

        self.get_cell()
    }

    fn move_west(&mut self) -> T {
        self.position.x -= 1;

        if self.position.x.abs() > self.start_offset.x as isize {
            self.map
                .iter_mut()
                .for_each(|row| row.insert(0, T::default()));
            self.start_offset.x += 1;

            return T::default();
        }

        self.get_cell()
    }

    fn move_east(&mut self) -> T {
        self.position.x += 1;

        if (self.position.x + self.start_offset.x as isize) as usize == self.map[0].len() {
            self.map.iter_mut().for_each(|row| row.push(T::default()));

            return T::default();
        }

        self.get_cell()
    }
}

impl<T: CellDisplay> DynamicMap<T> {
    pub fn show(&self, f: &mut dyn Write) -> Result<()> {
        self.show_with_path(f, &Vec::new())
    }

    pub fn show_with_msg(&self, f: &mut dyn Write, msg: &String) -> Result<()> {
        self.show_with_path_msg(f, &Vec::new(), msg)
    }

    pub fn show_with_path(&self, f: &mut dyn Write, path: &[PointU]) -> Result<()> {
        self.show_with_path_msg(f, path, &"".to_string())
    }

    pub fn show_with_path_msg(&self, f: &mut dyn Write, path: &[PointU], msg: &String) -> Result<()> {
        let mut buf = String::new();
        let abs_pos = self.abs_position();

        for (i, row) in self.map.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                let mut cell_set = false;

                if abs_pos.x == j && abs_pos.y == i {
                    if let Some(cur_char) = cell.current() {
                        buf.push(cur_char);
                        cell_set = true;
                    }
                }

                if !cell_set && self.start_offset.x == j && self.start_offset.y == i {
                    if let Some(start_char) = cell.start() {
                        buf.push(start_char);
                        cell_set = true;
                    }
                }

                if !cell_set {
                    if path.iter().any(|pos| pos.x == j && pos.y == i) {
                        buf += format!("{}{}", color::Bg(color::Blue), cell.display()).as_str();
                    } else {
                        buf += format!("{}{}", color::Bg(color::Black), cell.display()).as_str();
                    }
                }
            }
            buf.push('\n');
        }

        write!(f,
            "{}",
            termion::clear::All,
        )?;

        let mut pos = 1;

        for line in buf.lines() {
            write!(f,
                "{}{}",
                termion::cursor::Goto(1, pos),
                line
            )?;
            pos += 1;
        }

        for line in msg.lines() {
            write!(f, "{}{}", termion::cursor::Goto(1, pos), line)?;
            pos += 1;
        }

        // std::io::stdout().flush().unwrap();

        Ok(())
    }
}
