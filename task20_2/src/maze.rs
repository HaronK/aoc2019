use anyhow::{bail, ensure, Result};
use common::color_text::*;
use common::point::*;
use pathfinding::prelude::*;
use std::collections::HashMap;
use std::fmt;
use std::io::Write;
use std::{thread, time};
use termion;

enum NeighborStatus {
    Void,
    Blocked,
    Free(PointU),
}

#[derive(Clone, Debug, PartialEq)]
enum Side {
    Outer,
    Inner,
}

#[derive(Clone)]
enum Cell {
    Void,
    Wall,
    Free,
    /// Teleport id and destination point
    Teleport(u8, PointU, Side),
    /// Exit id
    Exit(u8),
}

impl Cell {
    fn to_exit(&mut self) -> bool {
        let mut res = false;
        *self = match std::mem::replace(self, Self::Exit(Default::default())) {
            Self::Teleport(id, p, _) if p == PointU::max() => { res = true; Self::Exit(id) }
            v => v,
        };
        res
    }
}

impl fmt::Debug for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ch = match self {
            Self::Void => ' ',
            Self::Wall => '█',
            Self::Free => '░',
            Self::Teleport(_, _, _) => '+',
            Self::Exit(_) => '◊',
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

    fn get_anomaly_name(&self, id: u8) -> String {
        let mut res = String::new();
        let ((ch1, ch2), _) = self.anomaly.iter().find(|(_k, v)| v.0 == id).unwrap();

        res.push(*ch1);
        res.push(*ch2);

        res
    }

    fn check_teleport(&mut self, char_map: &Vec<Vec<char>>, x: usize, y: usize, sx: isize, sy: isize, side: &Side) -> Option<Cell> {
        let (dx1, dx2, dy1, dy2) = if sx == 0 {
            if sy == 1 {
                (x, x, y + 1, y + 2)
            } else {
                (x, x, y - 2, y - 1)
            }
        } else {
            if sx == 1 {
                (x + 1, x + 2, y, y)
            } else {
                (x - 2, x - 1, y, y)
            }
        };

        let tx = (x as isize + sx) as usize;
        let ty = (y as isize + sy) as usize;
        if char_map[ty][tx].is_alphanumeric() {
            let name = (char_map[dy1][dx1], char_map[dy2][dx2]);
            let mut id = self.anomaly.len() as u8;
            let entry = self.anomaly.entry(name).or_insert((id, PointU::max(), PointU::max()));

            id = entry.0;

            let cur_pos = PointU::new(x - 2, y - 2);
            let pos = if entry.1 == PointU::max() {
                entry.1 = cur_pos;
                PointU::max()
            } else {
                entry.2 = cur_pos.clone();
                let p = entry.1.clone();
                // connect first teleport
                if let Cell::Teleport(_, _, s) = &self.map[p.y][p.x] {
                    self.map[p.y][p.x] = Cell::Teleport(id, cur_pos, s.clone());
                } else {
                    unreachable!();
                }
                p
            };

            return Some(Cell::Teleport(id as u8, pos, side.clone()))
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
                        let side = if x == xb || x == xe || y == yb || y == ye {
                            Side::Outer
                        } else {
                            Side::Inner
                        };

                        if let Some(t) = self.check_teleport(&char_map, x, y, 0, -1, &side) {
                            t
                        } else if let Some(t) = self.check_teleport(&char_map, x, y, 0, 1, &side) {
                            t
                        } else if let Some(t) = self.check_teleport(&char_map, x, y, -1, 0, &side) {
                            t
                        } else if let Some(t) = self.check_teleport(&char_map, x, y, 1, 0, &side) {
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
        ensure!(self.exits.len() == 2, "Expected 2 exits but was {}", self.exits.len());

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

    fn cell2dir(&self, pos: PointU, z: usize) -> NeighborStatus {
        let cell = self.cell(&pos);

        match cell {
            Cell::Void => NeighborStatus::Void,
            Cell::Wall => NeighborStatus::Blocked,
            // Teleports doesn't work on the outer side of the top most level
            Cell::Teleport(_, _, s) if *s == Side::Outer && z == 0 => NeighborStatus::Blocked,
            // Exits work only on the top most level
            Cell::Exit(_) if z > 0 => NeighborStatus::Blocked,
            _ => NeighborStatus::Free(pos),
        }
}

    fn cell2north(&self, pos: &PointU) -> NeighborStatus {
        if pos.y == 0 {
            NeighborStatus::Void
        } else {
            let p = PointU::new(pos.x, pos.y - 1);

            self.cell2dir(p, pos.z)
        }
    }

    fn cell2south(&self, pos: &PointU) -> NeighborStatus {
        if pos.y >= self.map.len() - 1 {
            NeighborStatus::Void
        } else {
            let p = PointU::new(pos.x, pos.y + 1);

            self.cell2dir(p, pos.z)
        }
    }

    fn cell2west(&self, pos: &PointU) -> NeighborStatus {
        if pos.x == 0 {
            NeighborStatus::Void
        } else {
            let p = PointU::new(pos.x - 1, pos.y);

            self.cell2dir(p, pos.z)
        }
    }

    fn cell2east(&self, pos: &PointU) -> NeighborStatus {
        if pos.x >= self.map[0].len() - 1 {
            NeighborStatus::Void
        } else {
            let p = PointU::new(pos.x + 1, pos.y);

            self.cell2dir(p, pos.z)
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
            Cell::Teleport(_, dest, side) => {
                for dc in dir_cells {
                    match dc {
                        NeighborStatus::Free(mut p) => {
                            // keep the level if we can move
                            p.z = pos.z;
                            result.push(p);
                        }
                        NeighborStatus::Void => {
                            // void means we can teleport
                            let mut d = dest.clone();
                            // change level on teleportation
                            match side {
                                // go up from the outer level
                                Side::Outer => d.z = pos.z - 1,
                                // go deep from the inner level
                                Side::Inner => d.z = pos.z + 1,
                            }
                            result.push(d);
                        }
                        NeighborStatus::Blocked => {}
                    }
                }
            }
            _ => {
                for dc in dir_cells {
                    match dc {
                        NeighborStatus::Free(mut p) => {
                            // keep the level if we can move
                            p.z = pos.z;
                            result.push(p);
                        }
                        NeighborStatus::Void => {
                            // this should be possible only on exit points
                            // unreachable!();
                        }
                        NeighborStatus::Blocked => {}
                    }
                }
            }
        }

        result
    }

    fn build_path(&self) -> Result<Vec<PointU>> {
        let p1 = self.exits[1].clone();
        let p2 = self.exits[0].clone();
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
            |pos| (pos.x as isize - p2.x as isize).abs() + (pos.y as isize - p2.y as isize).abs() + (pos.z as isize - p2.z as isize).abs() * 100,
            |pos| p2 == *pos,
        )
        .unwrap_or((Vec::new(), 0)).0)

        // Ok(bfs(&p1, |pos| self.neighbors(&pos), |pos| p2 == *pos).unwrap_or(Vec::new()))
    }

    fn draw_slide(&self, user: char, user_pos: &PointU, user_color: &Color, teleports: &Vec<(char, Color)>) {
        let mut buf = String::new();
        let sz = self.size();
        for y in 0.. sz.y {
            for x in 0..sz.x {
                if user_pos.x == x && user_pos.y == y {
                    buf += color_str(user_color, &user.to_string()).as_str();
                } else {
                    let cell = self.cell(&PointU::new(x, y));

                    match cell {
                        Cell::Void => buf += " ",
                        Cell::Wall => buf += "█",
                        Cell::Free => buf += "░",
                        Cell::Teleport(id, _, side) => {
                            let (ch, color) = &teleports[*id as usize];

                            if *side == Side::Outer && user_pos.z == 0 {
                                buf += color_str(&Color::Red, &"X".to_string()).as_str();
                            } else {
                                buf += color_str(color, &ch.to_string()).as_str();
                            }
                        }
                        Cell::Exit(id) => {
                            let (_, color) = &teleports[*id as usize];

                            if user_pos.z > 0 {
                                buf += color_str(&Color::Red, &"X".to_string()).as_str();
                            } else {
                                buf += color_str(color, &"◊".to_string()).as_str();
                            }
                        }
                    }
                }
            }
            buf.push('\n');
        }

        print!(
            "{}{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            buf
        );
        std::io::stdout().flush().unwrap();
    }
}

impl fmt::Debug for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Map[{:?}]:", self.size())?;
        for row in &self.map {
            for cell in row {
                write!(f, "{:?}", cell)?;
            }
            writeln!(f)?;
        }

        writeln!(f, "Anomalies[{}]:", self.anomaly.len())?;
        for ((ch1, ch2), (id, p1, p2)) in &self.anomaly {
            writeln!(f, "  {}{}[{:2}]: {:?} {:?}", ch1, ch2, id, p1, p2)?;
        }
        writeln!(f, "Exits[{}]: {:?}", self.exits.len(), self.exits)?;

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

    fn dump_path(&self, path: &Vec<PointU>) {
        println!("Path[{}]:", path.len());

        for pos in path {
            print!("{}{:?}", "  ".repeat(pos.z), pos);

            let cell = self.map.cell(&pos);
            match cell {
                Cell::Exit(id) => println!(" Exit {}", id),
                Cell::Teleport(id, dest, side) => println!(" Teleport {} {:?} {:?}", id, dest, side),
                _ => println!(),
            }
        }
    }

    fn dump_portals(&self, path: &Vec<PointU>) {
        print!("Teleports:");

        let mut prev_portal = false;
        for pos in path {
            let cell = self.map.cell(&pos);

            match cell {
                Cell::Exit(id) => {
                    let name = self.map.get_anomaly_name(*id);
                    print!(" {}", name);
                }
                Cell::Teleport(id, _, side) => {
                    if prev_portal {
                        prev_portal = false;
                    } else {
                        let sign = if *side == Side::Outer { '-' } else { '+' };
                        let name = self.map.get_anomaly_name(*id);
                        print!(" {}{}", sign, name);
                        prev_portal = true;
                    }
                }
                _ => {},
            }
        }

        println!();
    }

    fn animate_path(&self, path: &Vec<PointU>) {
        let delay = time::Duration::from_millis(100);
        let colors = vec![Color::Green, Color::Yellow, Color::Blue, Color::Magenta, Color::Cyan];
        let mut teleports = vec![('0', Color::White); self.map.anomaly.len()];

        for (_, (id, _, _)) in &self.map.anomaly {
            let ch = ('0' as u8 + id % 10) as char;
            let color = colors[*id as usize % colors.len()].clone();

            teleports[*id as usize] = (ch, color);
        }

        for (i, pos) in path.iter().enumerate() {
            self.map.draw_slide('●', &pos, &Color::LightBlue, &teleports);
            println!("Level: {}", pos.z);
            println!("Step: {}/{}", i, path.len());

            thread::sleep(delay);
        }
    }

    fn validate_path(&self, path: &Vec<PointU>) -> Result<()> {
        for pos in path {
            let cell = self.map.cell(&pos);

            match cell {
                Cell::Void | Cell::Wall => bail!("Path node is in the wall or void."),
                Cell::Teleport(_, _, side) if *side == Side::Outer && pos.z == 0 => bail!("Outer teleports are not available on the top level."),
                Cell::Exit(_) if pos.z > 0 => bail!("Exits are available only on the top level."),
                _ => {}
            }
        }
        Ok(())
    }

    pub fn find_shortest_path(&self, animate: bool) -> Result<usize> {
        let path = self.map.build_path()?;

        self.validate_path(&path)?;

        // self.dump_path(&path);

        if animate {
            self.animate_path(&path);
        }

        self.dump_portals(&path);

        Ok(path.len() - 1)
    }
}
