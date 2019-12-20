use crate::drone::*;
use anyhow::{anyhow, Result};
use common::log::*;
use std::fs::File;
use std::io::{prelude::*, BufReader};

mod drone;

fn main() -> Result<()> {
    let log = Log::new(false);
    let file = File::open("input.txt")?;
    let reader = BufReader::new(file);
    let prog_str = reader
        .lines()
        .nth(0)
        .ok_or_else(|| anyhow!("ERROR: Cannot read program string."))??;

    let mut drone = Drone::new(&prog_str, &log)?;

    let area_size = drone.check_area()?;

    println!("Area size: {:?}", area_size);

    let santa_pos = drone.find_santa()?;

    println!("Santa position: {:?}", santa_pos);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use common::intcode_comp::*;

    #[test]
    fn test1() -> Result<()> {
        let log = Log::new(false);
        let mut drone = Drone::new("", &log)?;
        let mut manual_grid = ManualGrid::load(
            r#"#.......................................
            .#......................................
            ..##....................................
            ...###..................................
            ....###.................................
            .....####...............................
            ......#####.............................
            ......######............................
            .......#######..........................
            ........########........................
            .........#########......................
            ..........#########.....................
            ...........##########...................
            ...........############.................
            ............############................
            .............#############..............
            ..............##############............
            ...............###############..........
            ................###############.........
            ................#################.......
            .................########OOOOOOOOOO.....
            ..................#######OOOOOOOOOO#....
            ...................######OOOOOOOOOO###..
            ....................#####OOOOOOOOOO#####
            .....................####OOOOOOOOOO#####
            .....................####OOOOOOOOOO#####
            ......................###OOOOOOOOOO#####
            .......................##OOOOOOOOOO#####
            ........................#OOOOOOOOOO#####
            .........................OOOOOOOOOO#####
            ..........................##############
            ..........................##############
            ...........................#############
            ............................############
            .............................###########"#,
        );

        // manual_grid.dump();

        let santa_pos = drone.find_santa_common(10, 0, 0, &mut manual_grid)?;

        assert_eq!(250020, santa_pos);

        Ok(())
    }

    struct ManualGrid {
        grid: Vec<Vec<DataType>>,
    }

    impl ManualGrid {
        fn load(data: &str) -> Self {
            let mut grid = Vec::new();

            for line in data.lines() {
                let row_str = line.trim();
                let mut row = Vec::new();

                for ch in row_str.chars() {
                    row.push(if ch != '.' { 1 } else { 0 });
                }

                grid.push(row);
            }

            Self { grid }
        }

        fn dump(&self) {
            for row in &self.grid {
                for v in row {
                    print!("{}", if *v == 1 { '#' } else { '.' });
                }
                println!();
            }
        }
    }

    impl Grid for ManualGrid {
        fn get_value(&mut self, x: usize, y: usize) -> Result<DataType> {
            if x >= self.grid[0].len() || y >= self.grid.len() {
                return Ok(0);
            }

            Ok(self.grid[y][x])
        }
    }
}
