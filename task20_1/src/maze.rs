use anyhow::{ensure, Result};
use common::point::*;
use pathfinding::prelude::astar;
use std::collections::HashMap;
use std::fmt;

#[derive(Clone)]
enum Cell {
    Void,
    Wall,
    Free,
    /// Teleport id and destination point
    Teleport(u8, PointU),
    /// Exit id
    Exit(u8),
}

impl Cell {
    fn to_exit(&mut self) -> bool {
        let mut res = false;
        *self = match std::mem::replace(self, Self::Exit(Default::default())) {
            Self::Teleport(id, p) if p == PointU::max() => {
                res = true;
                Self::Exit(id)
            }
            v => v,
        };
        res
    }
}

impl fmt::Debug for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ch = match self {
            Self::Void => ' ',
            Self::Wall => '#',
            Self::Free => '.',
            Self::Teleport(_, _) => '+',
            Self::Exit(_) => '*',
        };
        write!(f, "{}", ch)
    }
}

struct Map {
    map: Vec<Vec<Cell>>,
    anomaly: HashMap<(char, char), (u8, PointU, PointU)>,
    exits: Vec<PointU>,
}

impl Map {
    fn new() -> Self {
        Self {
            map: Vec::new(),
            anomaly: HashMap::new(),
            exits: Vec::new(),
        }
    }

    fn check_teleport(
        &mut self,
        char_map: &[Vec<char>],
        x: usize,
        y: usize,
        sx: isize,
        sy: isize,
    ) -> Option<Cell> {
        let (dx1, dx2, dy1, dy2) = if sx == 0 {
            if sy == 1 {
                (x, x, y + 1, y + 2)
            } else {
                (x, x, y - 2, y - 1)
            }
        } else if sx == 1 {
            (x + 1, x + 2, y, y)
        } else {
            (x - 2, x - 1, y, y)
        };

        let tx = (x as isize + sx) as usize;
        let ty = (y as isize + sy) as usize;
        if char_map[ty][tx].is_alphanumeric() {
            let name = (char_map[dy1][dx1], char_map[dy2][dx2]);
            let id = self.anomaly.len() as u8;
            let entry = self
                .anomaly
                .entry(name)
                .or_insert((id, PointU::max(), PointU::max()));

            let cur_pos = PointU::new(x - 2, y - 2);
            let pos = if entry.1 == PointU::max() {
                entry.1 = cur_pos;
                PointU::max()
            } else {
                entry.2 = cur_pos.clone();
                let p = entry.1.clone();
                // connect first teleport
                self.map[p.y][p.x] = Cell::Teleport(id as u8, cur_pos);
                p
            };

            return Some(Cell::Teleport(id as u8, pos));
        }
        None
    }

    fn load(&mut self, data: &str) -> Result<()> {
        let width = data.lines().nth(0).unwrap().len();
        let height = data.lines().count();
        let xb = 2;
        let xe = width - 3;
        let yb = 2;
        let ye = height - 3;

        let mut char_map = Vec::new();

        for line in data.lines() {
            char_map.push(line.chars().collect::<Vec<char>>());
        }

        for y in yb..=ye {
            let mut row = Vec::new();
            for x in xb..=xe {
                let ch = char_map[y][x];
                let cell = match ch {
                    // ' ' => Cell::Void,
                    '#' => Cell::Wall,
                    '.' => {
                        if let Some(t) = self.check_teleport(&char_map, x, y, 0, -1) {
                            t
                        } else if let Some(t) = self.check_teleport(&char_map, x, y, 0, 1) {
                            t
                        } else if let Some(t) = self.check_teleport(&char_map, x, y, -1, 0) {
                            t
                        } else if let Some(t) = self.check_teleport(&char_map, x, y, 1, 0) {
                            t
                        } else {
                            Cell::Free
                        }
                    }
                    _ => Cell::Void,
                };
                row.push(cell);
            }
            self.map.push(row);
        }

        // find exits
        let sz = self.size();
        for y in 0..sz.y {
            for x in 0..sz.x {
                if self.map[y][x].to_exit() {
                    self.exits.push(PointU::new(x, y));
                }
            }
        }
        ensure!(
            self.exits.len() == 2,
            "Expected 2 exits but was {}",
            self.exits.len()
        );

        Ok(())
    }

    fn size(&self) -> PointU {
        let ysize = self.map.len();
        let xsize = if ysize > 0 { self.map[0].len() } else { 0 };

        PointU::new(xsize, ysize)
    }

    fn cell(&self, pos: &PointU) -> &Cell {
        &self.map[pos.y][pos.x]
    }

    fn cell2north(&self, pos: &PointU) -> Option<PointU> {
        if pos.y == 0 {
            None
        } else {
            let p = PointU::new(pos.x, pos.y - 1);
            let cell = self.cell(&p);

            match cell {
                Cell::Void | Cell::Wall => None,
                _ => Some(p),
            }
        }
    }

    fn cell2south(&self, pos: &PointU) -> Option<PointU> {
        if pos.y >= self.map.len() - 1 {
            None
        } else {
            let p = PointU::new(pos.x, pos.y + 1);
            let cell = self.cell(&p);

            match cell {
                Cell::Void | Cell::Wall => None,
                _ => Some(p),
            }
        }
    }

    fn cell2west(&self, pos: &PointU) -> Option<PointU> {
        if pos.x == 0 {
            None
        } else {
            let p = PointU::new(pos.x - 1, pos.y);
            let cell = self.cell(&p);

            match cell {
                Cell::Void | Cell::Wall => None,
                _ => Some(p),
            }
        }
    }

    fn cell2east(&self, pos: &PointU) -> Option<PointU> {
        if pos.x >= self.map[0].len() - 1 {
            None
        } else {
            let p = PointU::new(pos.x + 1, pos.y);
            let cell = self.cell(&p);

            match cell {
                Cell::Void | Cell::Wall => None,
                _ => Some(p),
            }
        }
    }

    fn neighbors(&self, pos: &PointU) -> Vec<PointU> {
        let mut result = Vec::new();

        let cell = self.cell(pos);

        let dir_cells = vec![
            self.cell2north(&pos),
            self.cell2south(&pos),
            self.cell2west(&pos),
            self.cell2east(&pos),
        ];

        match cell {
            Cell::Teleport(_, dest) => {
                for dc in dir_cells {
                    if let Some(p) = dc {
                        result.push(p);
                    } else {
                        result.push(dest.clone());
                    }
                }
            }
            _ => {
                for dc in dir_cells {
                    if let Some(p) = dc {
                        result.push(p);
                    }
                }
            }
        }

        result
    }

    fn build_path(&self) -> Result<Vec<PointU>> {
        let p1 = self.exits[0].clone();
        let p2 = self.exits[1].clone();
        let map_size = self.size();

        ensure!(
            p1.x < map_size.x && p1.y < map_size.y,
            "Point is out of map. p1: {:?} > {:?}",
            p1,
            map_size
        );
        ensure!(
            p2.x < map_size.x && p2.y < map_size.y,
            "Point is out of map. p2: {:?} > {:?}",
            p2,
            map_size
        );

        Ok(astar(
            &p1,
            |pos| self.neighbors(&pos).into_iter().map(|p| (p, 1)),
            |pos| (pos.x as isize - p2.x as isize).abs() + (pos.y as isize - p2.y as isize).abs(),
            |pos| p2 == *pos,
        )
        .unwrap_or((Vec::new(), 0))
        .0)
    }
}

impl fmt::Debug for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.map {
            for cell in row {
                write!(f, "{:?}", cell)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub struct Maze {
    map: Map,
}

impl Maze {
    pub fn new(data: &str) -> Result<Self> {
        let mut map = Map::new();
        map.load(data)?;
        Ok(Self { map })
    }

    pub fn dump_map(&self) {
        println!("{:?}", self.map);
    }

    pub fn find_shortest_path(&self) -> Result<usize> {
        let path = self.map.build_path()?;

        Ok(path.len() - 1)
    }
}
