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

    fn cell(&self, x: u8, y: u8) -> bool {
        self.bit(y * 5 + x)
    }

    fn set_cell(&mut self, x: u8, y: u8) {
        self.set_bit(y * 5 + x);
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

    fn bugs_count(&self) -> usize {
        let mut res = 0;

        for i in 0..25 {
            if self.bit(i) {
                res += 1;
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
    maps: Vec<Map>,
    outer_cells: Vec<(u8, u8, Vec<(u8, u8, i8)>)>,
    inner_cells: Vec<(u8, u8, Vec<(u8, u8, i8)>)>,
}

impl BiomeField {
    pub fn new(data: &str) -> Result<Self> {
        let mut map = Map::new();

        map.load(data)?;

        let outer_cells = vec![
            (0, 0, vec![(1, 0, 0), (0, 1, 0), (2, 1, -1), (1, 2, -1)]),
            (1, 0, vec![(0, 0, 0), (2, 0, 0), (1, 1, 0), (2, 1, -1)]),
            (2, 0, vec![(1, 0, 0), (3, 0, 0), (2, 1, 0), (2, 1, -1)]),
            (3, 0, vec![(2, 0, 0), (4, 0, 0), (3, 1, 0), (2, 1, -1)]),
            (4, 0, vec![(3, 0, 0), (4, 1, 0), (2, 1, -1), (3, 2, -1)]),
            (0, 1, vec![(0, 0, 0), (1, 1, 0), (0, 2, 0), (1, 2, -1)]),
            (4, 1, vec![(4, 0, 0), (3, 1, 0), (4, 2, 0), (3, 2, -1)]),
            (0, 2, vec![(0, 1, 0), (1, 2, 0), (0, 3, 0), (1, 2, -1)]),
            (4, 2, vec![(4, 1, 0), (3, 2, 0), (4, 3, 0), (3, 2, -1)]),
            (0, 3, vec![(0, 2, 0), (1, 3, 0), (0, 4, 0), (1, 2, -1)]),
            (4, 3, vec![(4, 2, 0), (3, 3, 0), (4, 4, 0), (3, 2, -1)]),
            (0, 4, vec![(0, 3, 0), (1, 4, 0), (1, 2, -1), (2, 3, -1)]),
            (1, 4, vec![(0, 4, 0), (1, 3, 0), (2, 4, 0), (2, 3, -1)]),
            (2, 4, vec![(1, 4, 0), (2, 3, 0), (3, 4, 0), (2, 3, -1)]),
            (3, 4, vec![(2, 4, 0), (3, 3, 0), (4, 4, 0), (2, 3, -1)]),
            (4, 4, vec![(3, 4, 0), (4, 3, 0), (3, 2, -1), (2, 3, -1)]),
        ];
        let inner_cells = vec![
            (1, 1, vec![(0, 1, 0), (1, 0, 0), (2, 1, 0), (1, 2, 0)]),
            (
                2,
                1,
                vec![
                    (1, 1, 0),
                    (2, 0, 0),
                    (3, 1, 0),
                    (0, 0, 1),
                    (1, 0, 1),
                    (2, 0, 1),
                    (3, 0, 1),
                    (4, 0, 1),
                ],
            ),
            (3, 1, vec![(2, 1, 0), (3, 0, 0), (4, 1, 0), (3, 2, 0)]),
            (
                1,
                2,
                vec![
                    (0, 2, 0),
                    (1, 1, 0),
                    (1, 3, 0),
                    (0, 0, 1),
                    (0, 1, 1),
                    (0, 2, 1),
                    (0, 3, 1),
                    (0, 4, 1),
                ],
            ),
            (
                3,
                2,
                vec![
                    (3, 1, 0),
                    (4, 2, 0),
                    (3, 3, 0),
                    (4, 0, 1),
                    (4, 1, 1),
                    (4, 2, 1),
                    (4, 3, 1),
                    (4, 4, 1),
                ],
            ),
            (1, 3, vec![(0, 3, 0), (1, 2, 0), (2, 3, 0), (1, 4, 0)]),
            (
                2,
                3,
                vec![
                    (1, 3, 0),
                    (3, 3, 0),
                    (2, 4, 0),
                    (0, 4, 1),
                    (1, 4, 1),
                    (2, 4, 1),
                    (3, 4, 1),
                    (4, 4, 1),
                ],
            ),
            (3, 3, vec![(2, 3, 0), (3, 2, 0), (4, 3, 0), (3, 4, 0)]),
        ];

        Ok(Self {
            maps: vec![map],
            outer_cells,
            inner_cells,
        })
    }

    fn run_iter(&self) -> Vec<Map> {
        let mut res = Vec::new();
        let first_map = &self.maps[0];
        let last_map = &self.maps[self.maps.len() - 1];

        if self
            .outer_cells
            .iter()
            .any(|(x, y, _)| first_map.cell(*x, *y))
        {
            let mut new_map = Map::new();

            for (x, y, neighbors) in &self.inner_cells {
                let mut ncount = 0;
                for (nx, ny, nlvl) in neighbors {
                    if *nlvl != 0 && first_map.cell(*nx, *ny) {
                        ncount += 1;
                    }
                }

                if ncount == 1 || ncount == 2 {
                    new_map.set_cell(*x, *y);
                }
            }

            res.push(new_map);
        }

        for (lvl, map) in self.maps.iter().enumerate() {
            let mut new_map = Map::new();

            for (x, y, neighbors) in &self.outer_cells {
                let mut ncount = 0;
                for (nx, ny, nlvl) in neighbors {
                    if *nlvl != 0 {
                        if lvl > 0 && self.maps[lvl - 1].cell(*nx, *ny) {
                            ncount += 1;
                        }
                    } else if map.cell(*nx, *ny) {
                        ncount += 1;
                    }
                }

                if map.cell(*x, *y) {
                    if ncount == 1 {
                        new_map.set_cell(*x, *y);
                    }
                } else if ncount == 1 || ncount == 2 {
                    new_map.set_cell(*x, *y);
                }
            }

            for (x, y, neighbors) in &self.inner_cells {
                let mut ncount = 0;
                for (nx, ny, nlvl) in neighbors {
                    if *nlvl != 0 {
                        if lvl < self.maps.len() - 1 && self.maps[lvl + 1].cell(*nx, *ny) {
                            ncount += 1;
                        }
                    } else if map.cell(*nx, *ny) {
                        ncount += 1;
                    }
                }

                if map.cell(*x, *y) {
                    if ncount == 1 {
                        new_map.set_cell(*x, *y);
                    }
                } else if ncount == 1 || ncount == 2 {
                    new_map.set_cell(*x, *y);
                }
            }

            res.push(new_map);
        }

        if self
            .inner_cells
            .iter()
            .any(|(x, y, _)| last_map.cell(*x, *y))
        {
            let mut new_map = Map::new();
            for (x, y, neighbors) in &self.outer_cells {
                let mut ncount = 0;
                for (nx, ny, nlvl) in neighbors {
                    if *nlvl != 0 && last_map.cell(*nx, *ny) {
                        ncount += 1;
                    }
                }

                if ncount == 1 || ncount == 2 {
                    new_map.set_cell(*x, *y);
                }
            }

            res.push(new_map);
        }

        res
    }

    pub fn bugs_count(&mut self, steps: usize) -> usize {
        for _i in 0..steps {
            self.maps = self.run_iter();
        }

        let mut res = 0;

        for map in &self.maps {
            res += map.bugs_count();
        }

        res
    }
}
