use anyhow::{anyhow, bail, Result};
use std::fs::File;
use std::io::{prelude::*, BufReader};

#[derive(Debug)]
enum DirectedLine {
    Up { x: i32, y: i32, len: i32 },
    Down { x: i32, y: i32, len: i32 },
    Left { x: i32, y: i32, len: i32 },
    Right { x: i32, y: i32, len: i32 },
}

impl DirectedLine {
    // fn x(&self) -> i32 {
    //     match self {
    //         DirectedLine::Up{x, y: _, len: _} => *x,
    //         DirectedLine::Down{x, y: _, len: _} => *x,
    //         DirectedLine::Left{x, y: _, len: _} => *x,
    //         DirectedLine::Right{x, y: _, len: _} => *x,
    //     }
    // }

    // fn y(&self) -> i32 {
    //     match self {
    //         DirectedLine::Up{x: _, y, len: _} => *y,
    //         DirectedLine::Down{x: _, y, len: _} => *y,
    //         DirectedLine::Left{x: _, y, len: _} => *y,
    //         DirectedLine::Right{x: _, y, len: _} => *y,
    //     }
    // }

    fn len(&self) -> i32 {
        match self {
            DirectedLine::Up { len, .. } => *len,
            DirectedLine::Down { len, .. } => *len,
            DirectedLine::Left { len, .. } => *len,
            DirectedLine::Right { len, .. } => *len,
        }
    }

    // fn is_horizontal(&self) -> bool {
    //     match self {
    //         DirectedLine::Up{x: _, y: _, len: _} => false,
    //         DirectedLine::Down{x: _, y: _, len: _} => false,
    //         DirectedLine::Left{x: _, y: _, len: _} => true,
    //         DirectedLine::Right{x: _, y: _, len: _} => true,
    //     }
    // }

    // fn is_vertical(&self) -> bool {
    //     match self {
    //         DirectedLine::Up{x: _, y: _, len: _} => true,
    //         DirectedLine::Down{x: _, y: _, len: _} => true,
    //         DirectedLine::Left{x: _, y: _, len: _} => false,
    //         DirectedLine::Right{x: _, y: _, len: _} => false,
    //     }
    // }
}

fn main() -> Result<()> {
    let file = File::open("input.txt")?;
    let reader = BufReader::new(file);

    let mut wire_lines = reader.lines();
    let wire1 = parse(
        &wire_lines
            .nth(0)
            .ok_or_else(|| anyhow!("ERROR: Direction line is not specified."))??,
    )?;
    let wire2 = parse(
        &wire_lines
            .nth(0)
            .ok_or_else(|| anyhow!("ERROR: Direction line is not specified."))??,
    )?;

    // println!("Wire1: {:?}", wire1);
    // println!("Wire2: {:?}", wire2);

    let dist = closest_distance(&wire1, &wire2);

    println!("Dist: {}", dist);

    Ok(())
}

fn parse(directions: &str) -> Result<Vec<DirectedLine>> {
    let dir_str: Vec<&str> = directions.split(',').collect();
    let mut result: Vec<DirectedLine> = Vec::new();
    let mut x: i32 = 0;
    let mut y: i32 = 0;

    for dir in dir_str {
        let dir_char = dir
            .chars()
            .nth(0)
            .ok_or_else(|| anyhow!("ERROR: Missing direction letter."))?;
        match dir_char {
            'U' => {
                let len = dir[1..].parse()?;
                result.push(DirectedLine::Up { x, y, len });
                y += len;
            }
            'D' => {
                let len = dir[1..].parse()?;
                result.push(DirectedLine::Down { x, y, len });
                y -= len;
            }
            'L' => {
                let len = dir[1..].parse()?;
                result.push(DirectedLine::Left { x, y, len });
                x -= len;
            }
            'R' => {
                let len = dir[1..].parse()?;
                result.push(DirectedLine::Right { x, y, len });
                x += len;
            }
            _ => bail!(""),
        }
    }
    Ok(result)
}

fn closest_distance(wire1: &[DirectedLine], wire2: &[DirectedLine]) -> i32 {
    let mut dist = std::i32::MAX;
    let mut route1: i32 = 0;

    for (i, seg1) in wire1.iter().enumerate() {
        let mut route2: i32 = 0;
        for (j, seg2) in wire2.iter().enumerate() {
            // skip first segments check
            if i == 0 && j == 0 {
                continue;
            }

            match seg1 {
                DirectedLine::Up {
                    x: x1,
                    y: y1,
                    len: len1,
                } => match seg2 {
                    DirectedLine::Left {
                        x: x2,
                        y: y2,
                        len: len2,
                    } => {
                        dist = min_cross_route(
                            dist,
                            route1 + route2,
                            *x1,
                            *y1,
                            *y1 + *len1,
                            *y2,
                            *x2,
                            *x2 - *len2,
                        )
                    }
                    DirectedLine::Right {
                        x: x2,
                        y: y2,
                        len: len2,
                    } => {
                        dist = min_cross_route(
                            dist,
                            route1 + route2,
                            *x1,
                            *y1,
                            *y1 + *len1,
                            *y2,
                            *x2,
                            *x2 + *len2,
                        )
                    }
                    _ => (),
                },
                DirectedLine::Down {
                    x: x1,
                    y: y1,
                    len: len1,
                } => match seg2 {
                    DirectedLine::Left {
                        x: x2,
                        y: y2,
                        len: len2,
                    } => {
                        dist = min_cross_route(
                            dist,
                            route1 + route2,
                            *x1,
                            *y1,
                            *y1 - *len1,
                            *y2,
                            *x2,
                            *x2 - *len2,
                        )
                    }
                    DirectedLine::Right {
                        x: x2,
                        y: y2,
                        len: len2,
                    } => {
                        dist = min_cross_route(
                            dist,
                            route1 + route2,
                            *x1,
                            *y1,
                            *y1 - *len1,
                            *y2,
                            *x2,
                            *x2 + *len2,
                        )
                    }
                    _ => (),
                },
                DirectedLine::Left {
                    x: x1,
                    y: y1,
                    len: len1,
                } => match seg2 {
                    DirectedLine::Up {
                        x: x2,
                        y: y2,
                        len: len2,
                    } => {
                        dist = min_cross_route(
                            dist,
                            route1 + route2,
                            *x2,
                            *y2,
                            *y2 + *len2,
                            *y1,
                            *x1,
                            *x1 - *len1,
                        )
                    }
                    DirectedLine::Down {
                        x: x2,
                        y: y2,
                        len: len2,
                    } => {
                        dist = min_cross_route(
                            dist,
                            route1 + route2,
                            *x2,
                            *y2,
                            *y2 - *len2,
                            *y1,
                            *x1,
                            *x1 - *len1,
                        )
                    }
                    _ => (),
                },
                DirectedLine::Right {
                    x: x1,
                    y: y1,
                    len: len1,
                } => match seg2 {
                    DirectedLine::Up {
                        x: x2,
                        y: y2,
                        len: len2,
                    } => {
                        dist = min_cross_route(
                            dist,
                            route1 + route2,
                            *x2,
                            *y2,
                            *y2 + *len2,
                            *y1,
                            *x1,
                            *x1 + *len1,
                        )
                    }
                    DirectedLine::Down {
                        x: x2,
                        y: y2,
                        len: len2,
                    } => {
                        dist = min_cross_route(
                            dist,
                            route1 + route2,
                            *x2,
                            *y2,
                            *y2 - *len2,
                            *y1,
                            *x1,
                            *x1 + *len1,
                        )
                    }
                    _ => (),
                },
            }
            route2 += seg2.len();
        }
        route1 += seg1.len();
    }
    dist
}

fn min_cross_route(
    dist: i32,
    route: i32,
    x: i32,
    y1: i32,
    y2: i32,
    y: i32,
    x1: i32,
    x2: i32,
) -> i32 {
    if (x1 <= x && x <= x2 || x2 <= x && x <= x1) && (y1 <= y && y <= y2 || y2 <= y && y <= y1) {
        let d = route + (x - x1).abs() + (y - y1).abs();
        if d < dist {
            return d;
        }
    }
    dist
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() -> Result<()> {
        let wire1 = parse(&"R8,U5,L5,D3".to_string())?;
        let wire2 = parse(&"U7,R6,D4,L4".to_string())?;

        assert_eq!(30, closest_distance(&wire1, &wire2));

        Ok(())
    }

    #[test]
    fn test2() -> Result<()> {
        let wire1 = parse(&"R75,D30,R83,U83,L12,D49,R71,U7,L72".to_string())?;
        let wire2 = parse(&"U62,R66,U55,R34,D71,R55,D58,R83".to_string())?;

        assert_eq!(610, closest_distance(&wire1, &wire2));

        Ok(())
    }

    #[test]
    fn test3() -> Result<()> {
        let wire1 = parse(&"R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51".to_string())?;
        let wire2 = parse(&"U98,R91,D20,R16,D67,R40,U7,R15,U6,R7".to_string())?;

        assert_eq!(410, closest_distance(&wire1, &wire2));

        Ok(())
    }
}
