use anyhow::{ensure, Result};
use std::fmt;

#[derive(Clone, PartialEq)]
struct Map {
    map: u32,
}

impl Map {
    fn new() -> Self {
        Self { map: 0 }
    }

    fn load(&mut self, data: &str) -> Result<()> {
        let height = data.lines().count();
        ensure!(height == 5, "Expected height 5 but was {}", height);

        let width = data.lines().nth(0).unwrap().len();
        ensure!(width == 5, "Expected width 5 but was {}", width);

        let mut i = 0;
        for line in data.lines() {
            let line = line.trim();

            for ch in line.chars() {
                if ch == '#' {
                    self.set_bit(i);
                }
                i += 1;
            }
        }

        Ok(())
    }

    fn cell(&self, x: i8, y: i8) -> bool {
        if x < 0 || x >= 5 || y < 0 || y >= 5 {
            return false;
        }
        self.bit(y as u8 * 5 + x as u8)
    }

    fn set_cell(&mut self, x: i8, y: i8) {
        if x < 0 || x >= 5 || y < 0 || y >= 5 {
            return;
        }
        self.set_bit(y as u8 * 5 + x as u8);
    }

    fn bit(&self, pos: u8) -> bool {
        (self.map & 1 << pos) != 0
    }

    fn set_bit(&mut self, pos: u8) {
        self.map |= 1 << pos;
    }

    // fn clear_bit(&mut self, pos: u8) {
    //     self.map &= !(1 << pos);
    // }

    fn bio_diversity_rating(&self) -> u64 {
        let mut res = 0;

        for i in 0..25 {
            if self.bit(i) {
                res += 1 << i;
            }
        }

        res
    }
}

impl fmt::Debug for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..5 {
            for x in 0..5 {
                if self.cell(x, y) {
                    write!(f, "#")?;
                } else {
                    write!(f, ".")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub struct BiomeField {
    stages: Vec<Map>,
}

impl BiomeField {
    pub fn new(data: &str) -> Result<Self> {
        let mut map = Map::new();

        map.load(data)?;

        Ok(Self { stages: vec![map] })
    }

    fn run_iter(&self) -> Map {
        let last_map = &self.stages[self.stages.len() - 1];
        let mut map = Map::new();

        for y in 0..5 {
            for x in 0..5 {
                let cell = last_map.cell(x, y);
                let mut neighbors = 0;

                if last_map.cell(x, y - 1) {
                    neighbors += 1;
                }
                if last_map.cell(x, y + 1) {
                    neighbors += 1;
                }
                if last_map.cell(x - 1, y) {
                    neighbors += 1;
                }
                if last_map.cell(x + 1, y) {
                    neighbors += 1;
                }

                if cell {
                    if neighbors == 1 {
                        map.set_cell(x, y);
                    }
                } else if neighbors == 1 || neighbors == 2 {
                    map.set_cell(x, y);
                }
            }
        }

        map
    }

    pub fn get_rating(&mut self) -> u64 {
        // let mut i = 0;

        // println!("{} min\n{:?}", i, self.stages[0]);

        loop {
            let new_map = self.run_iter();

            // println!("{} min\n{:?}", i + 1, new_map);

            for map in &self.stages {
                if *map == new_map {
                    return map.bio_diversity_rating();
                }
            }

            self.stages.push(new_map);

            // i += 1;
        }
    }
}
