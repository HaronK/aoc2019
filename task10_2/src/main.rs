use anyhow::{ensure, Result};
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

fn main() -> Result<()> {
    let mut content = String::new();
    let mut file = File::open("input.txt")?;

    file.read_to_string(&mut content)?;

    let (mut asteroid_map, xsize, ysize) = parse_map(content)?;
    // dump_map(&asteroid_map, xsize, ysize);

    println!("Map size: [{}, {}]", xsize, ysize);

    let (max_asteroids, xpos, ypos) = find_best_asteriod(&asteroid_map, xsize, ysize);

    println!("Max asteroids [{}, {}]: {:?}", xpos, ypos, max_asteroids);

    let quad_map = QuadMap::new(&PointU::new(xpos, ypos), &PointU::new(xsize, ysize));

    let shoots = quad_map.shoot(&mut asteroid_map);

    print!("Shoots: [");
    for shoot in &shoots {
        print!("{}, ", shoot);
    }
    println!("]");

    if shoots.len() >= 200 {
        println!("Shoot 200: {}", shoots[199]);
    }

    Ok(())
}

struct QuadMap {
    quad_ne: Quadrant,
    quad_se: Quadrant,
    quad_sw: Quadrant,
    quad_nw: Quadrant,
    origin: PointU,
    size: PointU,
}

impl QuadMap {
    fn new(origin: &PointU, size: &PointU) -> Self {
        let mut quad_ne = Quadrant::new(origin, size, PointI::new(1, -1));
        let mut quad_se = Quadrant::new(origin, size, PointI::new(1, 1));
        let mut quad_sw = Quadrant::new(origin, size, PointI::new(-1, 1));
        let mut quad_nw = Quadrant::new(origin, size, PointI::new(-1, -1));

        quad_ne.sort();
        quad_se.sort();
        quad_sw.sort();
        quad_nw.sort();

        Self {
            quad_ne,
            quad_se,
            quad_sw,
            quad_nw,
            origin: origin.clone(),
            size: size.clone(),
        }
    }

    fn shoot(&self, asteroids_map: &mut Vec<usize>) -> Vec<PointU> {
        let mut result = Vec::new();
        let mut shoot_ne = true;
        let mut shoot_se = true;
        let mut shoot_sw = true;
        let mut shoot_nw = true;

        while !asteroids_map.is_empty() && (shoot_ne || shoot_se || shoot_sw || shoot_nw) {
            if shoot_ne {
                shoot_ne = self.shoot_quad(asteroids_map, &self.quad_ne, &mut result);
            }
            if shoot_se {
                shoot_se = self.shoot_quad(asteroids_map, &self.quad_se, &mut result);
            }
            if shoot_sw {
                shoot_sw = self.shoot_quad(asteroids_map, &self.quad_sw, &mut result);
            }
            if shoot_nw {
                shoot_nw = self.shoot_quad(asteroids_map, &self.quad_nw, &mut result);
            }
        }

        result
    }

    fn shoot_quad(
        &self,
        asteroids_map: &mut Vec<usize>,
        quad: &Quadrant,
        shoots: &mut Vec<PointU>,
    ) -> bool {
        let mut keep_shooting = false;

        // println!("Shoot quad:");
        'next: for ray in &quad.rays {
            // println!("  Ray: {}", ray.d);
            for offset in &ray.cells {
                let mut cell = PointU::default();
                cell.x = (self.origin.x as isize + offset.x) as usize;
                cell.y = (self.origin.y as isize + offset.y) as usize;

                let idx = get_idx(cell.x, cell.y, self.size.x);

                if let Some(pos) = asteroids_map.iter().position(|&v| v == idx) {
                    asteroids_map.remove(pos);
                    shoots.push(cell);
                    keep_shooting = true;
                    continue 'next;
                }
            }
        }

        keep_shooting
    }
}

impl fmt::Display for QuadMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Quadrant NE:\n{}", self.quad_ne)?;
        write!(f, "Quadrant SE:\n{}", self.quad_se)?;
        write!(f, "Quadrant SW:\n{}", self.quad_sw)?;
        write!(f, "Quadrant NW:\n{}", self.quad_nw)?;
        Ok(())
    }
}

#[derive(PartialEq, Debug, Clone)]
struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: Default> Default for Point<T> {
    fn default() -> Self {
        Self {
            x: T::default(),
            y: T::default(),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Point<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

type PointI = Point<isize>;

struct Ray {
    d: PointI,
    cells: Vec<PointI>,
}

impl Ray {
    fn new(dx: isize, dy: isize) -> Self {
        let min_dir = Ray::min_dir(dx, dy);
        Self {
            d: min_dir.clone(),
            cells: vec![min_dir],
        }
    }

    fn min_dir(dx: isize, dy: isize) -> PointI {
        let gcd = gcd(dx, dy);
        PointI::new(dx / gcd, dy / gcd)
    }

    fn append(&mut self, dx: isize, dy: isize) -> bool {
        let min_dir = Ray::min_dir(dx, dy);

        if self.d == min_dir {
            self.cells.push(PointI::new(dx, dy));
            return true;
        }

        false
    }

    fn sort(&mut self) {
        self.cells.sort_by(|d1, d2| d1.x.abs().cmp(&d2.x.abs()));
    }
}

impl Default for Ray {
    fn default() -> Self {
        Self {
            d: Point::default(),
            cells: Vec::default(),
        }
    }
}

impl fmt::Display for Ray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: [", self.d)?;
        for cell in &self.cells {
            write!(f, "{}, ", cell)?;
        }
        write!(f, "]")
    }
}

type PointU = Point<usize>;

struct Quadrant {
    rays: Vec<Ray>,
}

impl Quadrant {
    fn new(origin: &PointU, size: &PointU, dir: PointI) -> Self {
        let mut result = Self { rays: Vec::new() };
        let mut start_dir = PointI::default();

        if dir.x * dir.y < 0 {
            start_dir.x = 0;
            start_dir.y = dir.y;
        } else {
            start_dir.x = dir.x;
            start_dir.y = 0;
        }

        // println!("Origin: {}", origin);

        // fill starting vertical/horizontal ray
        let mut cell = origin.clone();
        cell.x = (cell.x as isize + start_dir.x) as usize;
        cell.y = (cell.y as isize + start_dir.y) as usize;

        // println!("Starting ray(dir={}):", start_dir);
        while (0..size.x).contains(&cell.x) && (0..size.y).contains(&cell.y) {
            // println!("  Cell: {}", cell);

            result.process_cell(&cell, &origin);

            cell.x = (cell.x as isize + start_dir.x) as usize;
            cell.y = (cell.y as isize + start_dir.y) as usize;
        }

        // fill quadrant rays
        cell.x = (origin.x as isize + dir.x) as usize;

        // println!("Quad rays(dir={}):", dir);
        while (0..size.x).contains(&cell.x) {
            cell.y = (origin.y as isize + dir.y) as usize;

            while (0..size.y).contains(&cell.y) {
                // println!("  Cell: {}", cell);

                result.process_cell(&cell, &origin);

                cell.y = (cell.y as isize + dir.y) as usize;
            }

            cell.x = (cell.x as isize + dir.x) as usize;
        }

        result
    }

    fn process_cell(&mut self, cell: &PointU, origin: &PointU) {
        let dx = cell.x as isize - origin.x as isize;
        let dy = cell.y as isize - origin.y as isize;

        // println!("    d: [{}, {}]", dx, dy);

        if !self.rays.iter_mut().any(|ray: &mut Ray| ray.append(dx, dy)) {
            self.rays.push(Ray::new(dx, dy));
        }
    }

    fn sort(&mut self) {
        self.rays.sort_by(|r1, r2| {
            if r1.d.x == 0 || r1.d.y == 0 {
                std::cmp::Ordering::Less
            } else if r2.d.x == 0 || r2.d.y == 0 {
                std::cmp::Ordering::Greater
            } else {
                (r2.d.x * r1.d.y).cmp(&(r1.d.x * r2.d.y))
            }
        });

        // NOTE: do not sort first ray because it is already in proper order
        for i in 1..self.rays.len() {
            self.rays[i].sort();
        }
    }
}

impl fmt::Display for Quadrant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ray in &self.rays {
            writeln!(f, "{}", ray)?;
        }
        Ok(())
    }
}

fn get_idx(x: usize, y: usize, xsize: usize) -> usize {
    y * xsize + x
}

fn get_xy(idx: usize, xsize: usize) -> (usize, usize) {
    (idx % xsize, idx / xsize)
}

fn find_best_asteriod(map: &[usize], xsize: usize, _ysize: usize) -> (usize, usize, usize) {
    let mut result = 0;
    let mut result_x = 0;
    let mut result_y = 0;
    let mut amounts = vec![0; map.len()];

    for i in 0..map.len() - 1 {
        let (xs, ys) = get_xy(map[i], xsize);

        // println!("[{}, {}]:", xs, ys);

        'n: for j in (i + 1)..map.len() {
            let (xe, ye) = get_xy(map[j], xsize);
            let dx = xe as isize - xs as isize;
            let dy = ye as isize - ys as isize;
            let gcd = gcd(dx, dy);

            // println!("  [{}, {}]: ({}, {}) {}", xe, ye, dx, dy, gcd);

            if gcd > 1 {
                let xoff = dx / gcd;
                let yoff = dy / gcd;

                for k in 1..gcd {
                    let x = (xs as isize + k * xoff) as usize;
                    let y = (ys as isize + k * yoff) as usize;
                    let idx = get_idx(x, y, xsize);

                    // println!("    [{}, {}]", x, y);

                    if map.contains(&idx) {
                        // println!("      hit at [{}, {}]", x, y);
                        continue 'n;
                    }
                }
            }

            amounts[i] += 1;
            amounts[j] += 1;
        }

        // println!("  Visible: {}", amounts[i]);

        if amounts[i] > result {
            result = amounts[i];
            result_x = xs;
            result_y = ys;
        }
    }

    (result, result_x, result_y)
}

fn gcd(mut v1: isize, mut v2: isize) -> isize {
    while v1 != 0 {
        let old_v1 = v1;
        v1 = v2 % v1;
        v2 = old_v1;
    }
    v2.abs()
}

// fn dump_map(map: &[usize], xsize: usize, ysize: usize) {
//     for i in 0..ysize {
//         for j in 0..xsize {
//             let c = if map.contains(&get_idx(j, i, xsize)) {
//                 "#"
//             } else {
//                 "."
//             };
//             print!("{}", c);
//         }
//         println!();
//     }
// }

fn parse_map<S: AsRef<str>>(map_str: S) -> Result<(Vec<usize>, usize, usize)> {
    let mut map = Vec::new();

    let map_rows: Vec<&str> = map_str.as_ref().lines().collect();
    ensure!(!map_rows.is_empty(), "ERROR: Map is empty.");

    let first_row = map_rows[0].trim();
    let xsize = first_row.len();
    let mut ysize = 1;

    parse_row(first_row, &mut map, ysize - 1);

    for r in map_rows.iter().skip(1) {
        let row = r.trim();
        ensure!(
            xsize == row.len(),
            "ERROR: Wrong size of the row. Expected {} but was {}.",
            xsize,
            row.len()
        );

        parse_row(row, &mut map, ysize);
        ysize += 1;
    }

    Ok((map, xsize, ysize))
}

fn parse_row<S: AsRef<str>>(row_str: S, map: &mut Vec<usize>, ypos: usize) {
    let row = row_str.as_ref();
    let xsize = row.len();

    for (i, c) in row.chars().enumerate() {
        if c == '#' {
            map.push(ypos * xsize + i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() -> Result<()> {
        common_test(
            8,
            r#".#..#
        .....
        #####
        ....#
        ...##"#,
        )
    }

    #[test]
    fn test2() -> Result<()> {
        common_test(
            33,
            r#"......#.#.
        #..#.#....
        ..#######.
        .#.#.###..
        .#..#.....
        ..#....#.#
        #..#....#.
        .##.#..###
        ##...#..#.
        .#....####"#,
        )
    }

    #[test]
    fn test3() -> Result<()> {
        common_test(
            35,
            r#"#.#...#.#.
            .###....#.
            .#....#...
            ##.#.#.#.#
            ....#.#.#.
            .##..###.#
            ..#...##..
            ..##....##
            ......#...
            .####.###."#,
        )
    }

    #[test]
    fn test4() -> Result<()> {
        common_test(
            41,
            r#".#..#..###
            ####.###.#
            ....###.#.
            ..###.##.#
            ##.##.#.#.
            ....###..#
            ..#.#..#.#
            #..#.#.###
            .##...##.#
            .....#.#.."#,
        )
    }

    #[test]
    fn test5() -> Result<()> {
        common_test(
            210,
            r#".#..##.###...#######
            ##.############..##.
            .#.######.########.#
            .###.#######.####.#.
            #####.##.#.##.###.##
            ..#####..#.#########
            ####################
            #.####....###.#.#.##
            ##.#################
            #####.##.###..####..
            ..######..##.#######
            ####.##.####...##..#
            .#####..#.######.###
            ##...#.##########...
            #.##########.#######
            .####.#.###.###.#.##
            ....##.##.###..#####
            .#.#.###########.###
            #.#.#.#####.####.###
            ###.##.####.##.#..##"#,
        )
    }

    #[test]
    fn test6() {
        test_quad("NE", (1, -1));
        test_quad("SE", (1, 1));
        test_quad("SW", (-1, 1));
        test_quad("NW", (-1, -1));
    }

    fn test_quad(name: &str, dir: (isize, isize)) {
        let mut quad = Quadrant::new(
            &PointU::new(2, 2),
            &PointU::new(5, 5),
            PointI::new(dir.0, dir.1),
        );
        quad.sort();

        println!("Quadrant[{}]: \n{}", name, quad);
    }

    fn common_test(expected: usize, map_str: &str) -> Result<()> {
        let (asteroid_map, xsize, ysize) = parse_map(map_str)?;

        let output = find_best_asteriod(&asteroid_map, xsize, ysize);

        assert_eq!(expected, output.0);

        Ok(())
    }
}
