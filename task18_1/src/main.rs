use crate::point::*;
use anyhow::{anyhow, ensure, Result};
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

    let (map, pathes, start_pos, free_keys) = parse_map(&data)?;
    let (dist, keys) = find_shortest_path(&map, &pathes, &start_pos, &free_keys)?;

    println!("Shortest path[{}]: {:?}", dist, keys);

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

fn parse_map(
    data: &str,
) -> Result<(
    Vec<Vec<char>>,
    HashMap<char, KeyDoorPath>,
    PointU,
    HashMap<char, PointU>,
)> {
    let mut map: Vec<Vec<char>> = Vec::new();
    let mut pathes = HashMap::new();
    let mut start_pos = PointU::new(std::usize::MAX, std::usize::MAX);

    for line in data.lines() {
        let row = line.trim();

        if !map.is_empty() {
            ensure!(
                map[0].len() == row.len(),
                "Wrong row size. Expected {} but was {}",
                map[0].len(),
                row.len()
            );
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
    let mut free_key_ch = Vec::new();
    // let mut end_key = PointU::new(std::usize::MAX, std::usize::MAX);
    for (ch, path) in &mut pathes {
        if path.door_pos.x == std::usize::MAX && path.door_pos.y == std::usize::MAX {
            free_key_ch.push(*ch);
        } else {
            path.path = build_path(&map, &path.key_pos, &path.door_pos)?;
            println!("{}: {:?}", ch, path);
        }
    }

    let mut free_keys = HashMap::new();

    for ch in free_key_ch {
        let free_pos = pathes.remove(&ch).unwrap().key_pos;
        free_keys.insert(ch, free_pos);
    }
    println!("free_keys: {:?}", free_keys);

    Ok((map, pathes, start_pos, free_keys))
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
    ensure!(
        p1.x < width && p1.y < height,
        "Point is out of map. p1: {:?} > [{}, {}]",
        p1,
        width,
        height
    );
    ensure!(
        p2.x < width && p2.y < height,
        "Point is out of map. p2: {:?} > [{}, {}]",
        p2,
        width,
        height
    );

    Ok(astar(
        p1,
        |pos| neighbors(map, &pos, "#").into_iter().map(|p| (p, 1)),
        |pos| (pos.x as isize - p2.x as isize).abs() + (pos.y as isize - p2.y as isize).abs(),
        |pos| *p2 == *pos,
    )
    .unwrap_or((Vec::new(), 0))
    .0)
}

fn find_shortest_path(
    map: &Vec<Vec<char>>,
    kd_pathes: &HashMap<char, KeyDoorPath>,
    start_pos: &PointU,
    free_keys: &HashMap<char, PointU>,
) -> Result<(usize, Vec<char>)> {
    find_shortest_subpath(0, map, kd_pathes, start_pos, free_keys)
}

fn find_shortest_subpath(
    iter: usize,
    map: &Vec<Vec<char>>,
    kd_pathes: &HashMap<char, KeyDoorPath>,
    start_pos: &PointU,
    free_keys: &HashMap<char, PointU>,
) -> Result<(usize, Vec<char>)> {
    let mut result = 0;
    let mut cur_pos = start_pos.clone();
    let mut keys = Vec::new();
    let mut pathes = kd_pathes.clone();
    let mut process_free_keys = true;

    loop {
        let mut reachable_keys = get_reachable_keys(map, &pathes, &cur_pos)?;
        // ensure!(!reachable_keys.is_empty(), "[{}] There are no reachable keys", iter);
        if reachable_keys.is_empty() {
            println!("[{}] There are no reachable keys", iter);
            break;
        }

        println!(
            "[{}] {:?} Reachable keys: {:?}",
            iter, cur_pos, reachable_keys
        );

        if reachable_keys.len() == 1 {
            let (ch, dist, _additional_keys) = reachable_keys.pop().unwrap();
            let path = pathes.remove(&ch).unwrap();
            println!("[{}] {}: {:?} {}", iter, ch, path.key_pos, dist);
            result += dist;
            cur_pos = path.key_pos;
            keys.push(ch);
        } else {
            // More than 1 key is accessible. Find recursively the best to pick up first.
            let mut closest_ch = ' ';
            let mut closest_dist = std::usize::MAX;
            let mut closest_keys = Vec::new();

            for (ch, dist, additional_keys) in reachable_keys {
                println!(
                    "[{}] Try '{}' key path. Dist: {} ------------------",
                    iter, ch, dist
                );

                let mut pathes_copy = pathes.clone();

                let branch_path = pathes_copy.remove(&ch).unwrap();

                for add_ch in &additional_keys {
                    pathes_copy.remove(add_ch).unwrap();
                }

                let (branch_dist, mut branch_keys) = find_shortest_subpath(
                    iter + 1,
                    map,
                    &pathes_copy,
                    &branch_path.key_pos,
                    free_keys,
                )?;
                println!("[{}] Dist: {} Path: {:?}", iter, branch_dist, branch_keys);

                if closest_dist > branch_dist + dist {
                    closest_ch = ch;
                    closest_dist = branch_dist + dist;
                    println!(
                        "[{}] New min: {} {} {:?} {:?}",
                        iter, closest_dist, ch, additional_keys, branch_keys
                    );
                    closest_keys = additional_keys;
                    closest_keys.push(ch);
                    closest_keys.append(&mut branch_keys);
                }
            }

            result += closest_dist;
            // keys.push(closest_ch);
            keys.append(&mut closest_keys);
            process_free_keys = false;
            break;
        }

        if pathes.is_empty() {
            println!("[{}] Keys pick order: {:?}", iter, keys);
            break;
        }
    }

    if process_free_keys {
        let end_path = build_path(map, &cur_pos, end_pos)?;
        println!(
            "[{}] {}: {:?} -> {:?} {}",
            iter,
            '+',
            cur_pos,
            end_pos,
            end_path.len() - 1
        );
        keys.push(map[end_pos.y][end_pos.x]);
        result += end_path.len() - 1;
    }

    println!("[{}] Result: {} Keys: {:?}", iter, result, keys);

    Ok((result, keys))
}

fn find_closest_key(
    map: &Vec<Vec<char>>,
    cur_pos: &PointU,
    keys: &HashMap<char, PointU>,
) -> (char, Vec<PointU>) {
}

/// Returns (key, distance to key, additional keys on a path)
fn get_reachable_keys(
    map: &Vec<Vec<char>>,
    pathes: &HashMap<char, KeyDoorPath>,
    cur_pos: &PointU,
) -> Result<Vec<(char, usize, Vec<char>)>> {
    let mut keys = Vec::new();

    'p2: for (ch, path) in pathes {
        let cur_path = build_path(map, cur_pos, &path.key_pos)?;

        for (_ch1, path1) in pathes {
            if cur_path.contains(&path1.door_pos) {
                continue 'p2;
            }
        }

        let mut additional_keys = Vec::new();

        if cur_path.len() > 2 {
            for i in 1..cur_path.len() - 1 {
                let add_ch = map[cur_path[i].y][cur_path[i].x];
                if *ch != add_ch && pathes.contains_key(&add_ch) {
                    additional_keys.push(add_ch);
                }
            }
        }

        keys.push((*ch, cur_path.len() - 1, additional_keys));
    }

    Ok(keys)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() -> Result<()> {
        common_test(
            8,
            &vec!['a', 'b'],
            r#"#########
            #b.A.@.a#
            #########"#,
        )
    }

    #[test]
    fn test2() -> Result<()> {
        common_test(
            86,
            &vec!['a', 'b', 'c', 'd', 'e', 'f'],
            r#"########################
            #f.D.E.e.C.b.A.@.a.B.c.#
            ######################.#
            #d.....................#
            ########################"#,
        )
    }

    #[test]
    fn test3() -> Result<()> {
        common_test(
            132,
            &vec!['b', 'a', 'c', 'd', 'f', 'e', 'g'],
            r#"########################
            #...............b.C.D.f#
            #.######################
            #.....@.a.B.c.d.A.e.F.g#
            ########################"#,
        )
    }

    #[test]
    fn test4() -> Result<()> {
        common_test(
            136,
            &vec![
                'a', 'f', 'b', 'j', 'g', 'n', 'h', 'd', 'l', 'o', 'e', 'p', 'c', 'i', 'k', 'm',
            ],
            r#"#################
            #i.G..c...e..H.p#
            ########.########
            #j.A..b...f..D.o#
            ########@########
            #k.E..a...g..B.n#
            ########.########
            #l.F..d...h..C.m#
            #################"#,
        )
    }

    #[test]
    fn test5() -> Result<()> {
        common_test(
            81,
            &vec!['a', 'c', 'f', 'i', 'd', 'g', 'b', 'e', 'h'],
            r#"########################
            #@..............ac.GI.b#
            ###d#e#f################
            ###A#B#C################
            ###g#h#i################
            ########################"#,
        )
    }

    fn common_test(expected_dist: usize, expected_keys: &Vec<char>, data: &str) -> Result<()> {
        let (map, pathes, start_pos, free_keys) = parse_map(&data)?;
        let (dist, keys) = find_shortest_path(&map, &pathes, &start_pos, &free_keys)?;
        assert_eq!(*expected_keys, keys);
        assert_eq!(expected_dist, dist);
        Ok(())
    }
}
