use anyhow::{ensure, Result};
use std::fs::File;
use std::io::prelude::*;

fn main() -> Result<()> {
    let mut content = String::new();
    let mut file = File::open("input.txt")?;

    file.read_to_string(&mut content)?;

    let (asteroid_map, xsize, ysize) = parse_map(content)?;
    // dump_map(&asteroid_map, xsize, ysize);

    let output = find_best_asteriod_visibility(&asteroid_map, xsize, ysize);

    println!("Output: {:?}", output);

    Ok(())
}

fn get_idx(x: usize, y: usize, xsize: usize) -> usize {
    y * xsize + x
}

fn get_xy(idx: usize, xsize: usize) -> (usize, usize) {
    (idx % xsize, idx / xsize)
}

fn find_best_asteriod_visibility(map: &Vec<usize>, xsize: usize, _ysize: usize) -> usize {
    let mut result = 0;
    let mut amounts = vec![0; map.len()];

    for i in 0..map.len() - 1 {
        let (xs, ys) = get_xy(map[i], xsize);

        // println!("[{}, {}]:", xs, ys);

        'n: for j in (i + 1)..map.len() {
            let (xe, ye) = get_xy(map[j], xsize);
            let dx = xe as isize - xs as isize;
            let dy = ye as isize - ys as isize;
            let gcd = gcd(dx.abs() as usize, dy.abs() as usize) as isize;

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
        }
    }

    result
}

fn gcd(mut v1: usize, mut v2: usize) -> usize {
    while v1 != 0 {
        let old_v1 = v1;
        v1 = v2 % v1;
        v2 = old_v1;
    }
    v2
}

fn dump_map(map: &Vec<usize>, xsize: usize, ysize: usize) {
    for i in 0..ysize {
        for j in 0..xsize {
            let c = if map.contains(&get_idx(j, i, xsize)) {
                "#"
            } else {
                "."
            };
            print!("{}", c);
        }
        println!("");
    }
}

fn parse_map<S: AsRef<str>>(map_str: S) -> Result<(Vec<usize>, usize, usize)> {
    let mut map = Vec::new();

    let map_rows: Vec<&str> = map_str.as_ref().lines().collect();
    ensure!(map_rows.len() > 0, "ERROR: Map is empty.");

    let first_row = map_rows[0].trim();
    let xsize = first_row.len();
    let mut ysize = 1;

    parse_row(first_row, &mut map, ysize - 1);

    for i in 1..map_rows.len() {
        let row = map_rows[i].trim();
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
    let mut x = 0;
    for c in row.chars() {
        if c == '#' {
            map.push(ypos * xsize + x);
        }
        x += 1;
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

    fn common_test(expected: usize, map_str: &str) -> Result<()> {
        let (asteroid_map, xsize, ysize) = parse_map(map_str)?;
        let output = find_best_asteriod_visibility(&asteroid_map, xsize, ysize);

        assert_eq!(expected, output);

        Ok(())
    }
}
