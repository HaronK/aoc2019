use anyhow::{anyhow, ensure, Result};
use crate::point::*;
use pathfinding::prelude::astar;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{prelude::*, BufReader};

mod point;

fn main() -> Result<()> {
    let file = File::open("input.txt")?;
    let reader = BufReader::new(file);
    let data = reader
        .lines()
        .nth(0)
        .ok_or_else(|| anyhow!("ERROR: Cannot read program string."))??;

    let (map, pathes, start_pos, end_pos) = parse_map(&data)?;
    let shortest_path = find_shortest_path(&map, &pathes, &start_pos, &end_pos)?;

    println!("Shortest path: {:?}", shortest_path);

    Ok(())
}

#[derive(Clone)]
struct KeyDoorPath {
    key_pos: PointU,
    door_pos: PointU,
    /// Path from the key to the door including end points
    path: Vec<PointU>,
}

impl KeyDoorPath {
    fn contains(&self, point: &PointU) -> bool {
        self.path.iter().any(|p| p == point)
    }

    fn dist_to_key(&self, pos: &PointU) -> usize {
        self.path.iter().position(|p| p == pos).unwrap()
    }
}

impl Default for KeyDoorPath {
    fn default() -> Self {
        Self {
            key_pos: PointU::new(std::usize::MAX, std::usize::MAX),
            door_pos: PointU::new(std::usize::MAX, std::usize::MAX),
            path: Vec::new(),
        }
    }
}

impl fmt::Debug for KeyDoorPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "key: {:?}, door: {:?} path: {:?}",
            self.key_pos, self.door_pos, self.path
        )
    }
}


fn parse_map(data: &str) -> Result<(Vec<Vec<char>>, HashMap<char, KeyDoorPath>, PointU, PointU)> {
    let mut map: Vec<Vec<char>> = Vec::new();
    let mut pathes = HashMap::new();
    let mut start_pos = PointU::new(std::usize::MAX, std::usize::MAX);

    for line in data.lines() {
        let row = line.trim();

        if !map.is_empty() {
            ensure!(map[0].len() == row.len(), "Wrong row size. Expected {} but was {}", map[0].len(), row.len());
        }

        for (i, c) in row.chars().enumerate() {
            if c == '@' {
                start_pos.set(i, map.len());
            } else if c.is_alphabetic() {
                let key = c.to_lowercase().to_string().chars().nth(0).unwrap();
                let path = pathes.entry(key).or_insert(KeyDoorPath::default());

                if c.is_lowercase() {
                    (*path).key_pos.set(i, map.len());
                } else {
                    (*path).door_pos.set(i, map.len());
                }
            }
        }

        map.push(row.chars().collect());
    }

    println!("Map: [{}, {}]", map[0].len(), map.len());
    let mut end_key_ch = ' ';
    // let mut end_key = PointU::new(std::usize::MAX, std::usize::MAX);
    for (ch, path) in &mut pathes {
        if path.door_pos.x == std::usize::MAX && path.door_pos.y == std::usize::MAX {
            end_key_ch = *ch;
        } else {
            path.path = build_path(&map, &path.key_pos, &path.door_pos)?;
            println!("{}: {:?}", ch, path);
        }
    }

    let end_pos = pathes.remove(&end_key_ch).unwrap().key_pos;
    println!("end_pos[{}]: {:?}", end_key_ch, end_pos);

    Ok((map, pathes, start_pos, end_pos))
}

fn neighbors(map: &Vec<Vec<char>>, pos: &PointU, barriers: &str) -> Vec<PointU> {
    let mut result = Vec::new();

    if !barriers.contains(map[pos.y - 1][pos.x]) {
        result.push(PointU::new(pos.x, pos.y - 1));
    }

    if !barriers.contains(map[pos.y + 1][pos.x]) {
        result.push(PointU::new(pos.x, pos.y + 1));
    }

    if !barriers.contains(map[pos.y][pos.x - 1]) {
        result.push(PointU::new(pos.x - 1, pos.y));
    }

    if !barriers.contains(map[pos.y][pos.x + 1]) {
        result.push(PointU::new(pos.x + 1, pos.y));
    }

    result
}

fn build_path(map: &Vec<Vec<char>>, p1: &PointU, p2: &PointU) -> Result<Vec<PointU>> {
    let height = map.len();
    let width = map[0].len();
    ensure!(p1.x < width && p1.y < height, "Point is out of map. p1: {:?} > [{}, {}]", p1, width, height);
    ensure!(p2.x < width && p2.y < height, "Point is out of map. p2: {:?} > [{}, {}]", p2, width, height);

    Ok(astar(
        p1,
        |pos| neighbors(map, &pos, "#").into_iter().map(|p| (p, 1)),
        |pos| {
            (pos.x as isize - p2.x as isize).abs()
                + (pos.y as isize - p2.y as isize).abs()
        },
        |pos| *p2 == *pos,
    )
    .unwrap_or((Vec::new(), 0)).0)
}

fn find_shortest_path(map: &Vec<Vec<char>>, kd_pathes: &HashMap<char, KeyDoorPath>, start_pos: &PointU, end_pos: &PointU) -> Result<usize> {
    let mut result = 0;
    let mut cur_pos = start_pos.clone();
    let mut keys = Vec::new();
    let mut pathes = kd_pathes.clone();

    loop {
        // find first 'free' path: path that contains current position and doesn't contain any other doors
        // if there are several such a pathes choose one with a closest distance between current position and door (check if there is a real reason to do this check)
        // this can be possible in following situation:
        //  A     b @   a   B
        // let mut key_door_ch = ' ';
        // 'p1: for (ch, path) in &pathes {
        //     if path.contains(&cur_pos) {
        //         for (ch1, path1) in &pathes {
        //             if ch != ch1 && path.contains(&path1.door_pos) {
        //                 continue 'p1;
        //             }
        //         }
        //         key_door_ch = *ch;
        //         break;
        //     }
        // }

        // // we can remove key-door record here because we get a key
        // if let Some(key_door_path) = pathes.remove(&key_door_ch) {//.ok_or_else(|| anyhow!("Cannot find a key to pick up. Found keys: {:?}", keys))?;
        //     result += key_door_path.dist_to_key(&cur_pos);
        //     cur_pos = key_door_path.key_pos;
        //     keys.push(key_door_ch);
        // } else {
            // if current position is now outside of key-door pairs find distance to closest key that is not blocked by any other door
            let mut closest_dist = std::usize::MAX;
            let mut closest_key_ch = ' ';
            let mut closest_key_pos = PointU::default();

            'p2: for (ch, path) in &pathes {
                let cur_path = build_path(map, &cur_pos, &path.key_pos)?;

                if closest_dist > cur_path.len() {
                    for (ch1, path1) in &pathes {
                        if cur_path.contains(&path1.door_pos) {
                            continue 'p2;
                        }
                    }
    
                    closest_dist = cur_path.len();
                    closest_key_ch = *ch;
                    closest_key_pos = path.key_pos.clone();
                }
            }

            println!(" {}: {:?} -> {:?} {}", closest_key_ch, cur_pos, closest_key_pos, closest_dist - 1);
            pathes.remove(&closest_key_ch).unwrap();
            result += closest_dist - 1;
            cur_pos = closest_key_pos;
            keys.push(closest_key_ch);
        // }

        if pathes.is_empty() {
            println!("Keys pick order: {:?}", keys);
            break;
        }
    }

    let end_path = build_path(map, &cur_pos, end_pos)?;
    println!(" {}: {:?} -> {:?} {}", '+', cur_pos, end_pos, end_path.len() - 1);
    result += end_path.len() - 1;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() -> Result<()> {
        common_test(
            8,
            r#"#########
            #b.A.@.a#
            #########"#)
    }

    #[test]
    fn test2() -> Result<()> {
        common_test(
            86,
            r#"########################
            #f.D.E.e.C.b.A.@.a.B.c.#
            ######################.#
            #d.....................#
            ########################"#)
    }

    #[test]
    fn test3() -> Result<()> {
        common_test(
            132,
            r#"########################
            #...............b.C.D.f#
            #.######################
            #.....@.a.B.c.d.A.e.F.g#
            ########################"#)
    }

    #[test]
    fn test4() -> Result<()> {
        common_test(
            136,
            r#"#################
            #i.G..c...e..H.p#
            ########.########
            #j.A..b...f..D.o#
            ########@########
            #k.E..a...g..B.n#
            ########.########
            #l.F..d...h..C.m#
            #################"#)
    }

    #[test]
    fn test5() -> Result<()> {
        common_test(
            81,
            r#"########################
            #@..............ac.GI.b#
            ###d#e#f################
            ###A#B#C################
            ###g#h#i################
            ########################"#)
    }

    fn common_test(expected: usize, data: &str) -> Result<()> {
        let (map, pathes, start_pos, end_pos) = parse_map(&data)?;
        assert_eq!(expected, find_shortest_path(&map, &pathes, &start_pos, &end_pos)?);
        Ok(())
    }
}
