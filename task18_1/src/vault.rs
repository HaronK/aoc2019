use anyhow::{ensure, Result};
use common::point::*;
use pathfinding::prelude::astar;
use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Debug)]
enum KeyInfo {
    WithDoor(KeyDoorPath),
    Single(PointU),
}

impl KeyInfo {
    fn to_single(&mut self) {
        *self = match std::mem::replace(self, Self::Single(Default::default())) {
            Self::WithDoor(path) => Self::Single(path.key_pos),
            v => v,
        }
    }

    fn key_pos(&self) -> PointU {
        match self {
            Self::WithDoor(path) => path.key_pos.clone(),
            Self::Single(pos) => pos.clone(),
        }
    }

    fn door_pos(&self) -> Option<PointU> {
        match self {
            Self::WithDoor(path) => Some(path.door_pos.clone()),
            Self::Single(_) => None,
        }
    }
}

#[derive(Clone)]
struct KeyDoorPath {
    key_pos: PointU,
    door_pos: PointU,
    /// Path from the key to the door including end points
    path: Vec<PointU>,
}

// impl KeyDoorPath {
//     fn contains(&self, point: &PointU) -> bool {
//         self.path.iter().any(|p| p == point)
//     }

//     fn dist_to_key(&self, pos: &PointU) -> usize {
//         self.path.iter().position(|p| p == pos).unwrap()
//     }
// }

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

#[derive(Clone)]
pub struct Route {
    start_pos: PointU,
    paths: HashMap<char, KeyInfo>,
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "start_pos: {:?}, paths[{}]: {:?}",
            self.start_pos,
            self.paths.len(),
            self.paths.keys()
        )?;

        // for path in &self.paths {
        //     write!(f, "\n  {:?}", path)?;
        // }

        Ok(())
    }
}

impl Route {
    fn new() -> Self {
        Self {
            start_pos: PointU::default(),
            paths: HashMap::new(),
        }
    }

    fn remove_key(&mut self, key: char) -> Option<KeyInfo> {
        self.paths.remove(&key)
    }

    fn format_nest(nest: usize, iter: usize) -> String {
        let mut res = String::from("");
        for _ in 0..nest {
            res += "  ";
        }
        res += &format!("[{},{}]", nest, iter);
        res
    }

    fn find_shortest_path(
        &mut self,
        nest: usize,
        map: &Map,
        cur_min: usize,
    ) -> Result<(usize, Vec<char>)> {
        let mut result = 0;
        let mut cur_pos = self.start_pos.clone();
        let mut keys = Vec::new();
        let mut iter = 0;

        println!("{} Start: {:?}", Self::format_nest(nest, iter), self);

        loop {
            let mut reachable_keys = self.get_reachable_keys(map, &cur_pos)?;
            // ensure!(!reachable_keys.is_empty(), "{} There are no reachable keys", format_nest(nest, iter));
            if reachable_keys.is_empty() {
                println!(
                    "{} There are no reachable keys",
                    Self::format_nest(nest, iter)
                );
                break;
            }

            println!(
                "{} {:?} Reachable keys[{}]: {:?}",
                Self::format_nest(nest, iter),
                cur_pos,
                reachable_keys.len(),
                reachable_keys
            );

            if reachable_keys.len() == 1 {
                let (ch, dist, additional_keys) = reachable_keys.pop().unwrap();
                ensure!(
                    additional_keys.is_empty(),
                    "Path should not contain additional keys: {:?}",
                    additional_keys
                );

                let key_info = self.remove_key(ch).unwrap();

                if result + dist >= cur_min {
                    // early exit
                    println!(
                        "{} {}: {} Early exit: {} > {}",
                        Self::format_nest(nest, iter),
                        ch,
                        dist,
                        result + dist,
                        cur_min
                    );
                    return Ok((result + dist, keys));
                }

                result += dist;
                keys.push(ch);
                cur_pos = key_info.key_pos();

                println!(
                    "{} {}: {:?} {}",
                    Self::format_nest(nest, iter),
                    ch,
                    cur_pos,
                    dist
                );
            } else {
                // More than 1 key is accessible. Find recursively the best to pick up first.
                // let mut closest_ch = ' ';
                let mut closest_dist = std::usize::MAX;
                let mut closest_keys = Vec::new();

                for (ch, dist, additional_keys) in reachable_keys {
                    println!(
                        "{} Try '{}' key path. Dist: {} ------------------",
                        Self::format_nest(nest, iter),
                        ch,
                        dist
                    );

                    if result + dist >= cur_min {
                        // early exit
                        println!(
                            "{} {}: {} Early exit: {} > {}",
                            Self::format_nest(nest, iter),
                            ch,
                            dist,
                            result + dist,
                            cur_min
                        );
                        continue;
                    }

                    let mut route_copy = self.clone();

                    let branch_path = route_copy.remove_key(ch).unwrap();

                    route_copy.start_pos = branch_path.key_pos();

                    for add_ch in &additional_keys {
                        route_copy.remove_key(*add_ch).unwrap();
                    }

                    let (branch_dist, mut branch_keys) =
                        route_copy.find_shortest_path(nest + 1, map, closest_dist)?;

                    println!(
                        "{} Dist[{}]: {}/{} Path: {:?}",
                        Self::format_nest(nest, iter),
                        ch,
                        branch_dist,
                        branch_dist + dist,
                        branch_keys
                    );

                    if closest_dist > branch_dist + dist {
                        closest_dist = branch_dist + dist;

                        println!(
                            "{} New min[{}]: {} {:?} {:?}",
                            Self::format_nest(nest, iter),
                            ch,
                            closest_dist,
                            additional_keys,
                            branch_keys
                        );

                        closest_keys = additional_keys;
                        closest_keys.push(ch);
                        closest_keys.append(&mut branch_keys);
                    }
                }

                if closest_dist != std::usize::MAX {
                    result += closest_dist;
                    keys.append(&mut closest_keys);
                } else if cur_min != std::usize::MAX {
                    result = cur_min + 1;
                } else {
                    result = cur_min;
                }

                break;
            }

            if self.paths.is_empty() {
                println!(
                    "{} Keys pick order: {:?}",
                    Self::format_nest(nest, iter),
                    keys
                );
                break;
            }

            iter += 1;
        }

        println!(
            "{} Result: {} Keys: {:?}",
            Self::format_nest(nest, iter),
            result,
            keys
        );

        Ok((result, keys))
    }

    /// Returns (key, distance to key, additional keys on a path)
    fn get_reachable_keys(
        &self,
        map: &Map,
        cur_pos: &PointU,
    ) -> Result<Vec<(char, usize, Vec<char>)>> {
        let mut keys = Vec::new();

        'p2: for (ch, key_info) in &self.paths {
            let pos = key_info.key_pos();

            // check if there are no other doors on the path
            let cur_path = map.build_path(cur_pos, &pos)?;
            let contains_door = self.paths.iter().any(|(_ch, key_info1)| {
                if let Some(door_pos) = key_info1.door_pos() {
                    return cur_path.contains(&door_pos);
                }
                false
            });

            if contains_door {
                continue 'p2;
            }

            let mut additional_keys = Vec::new();

            if cur_path.len() > 2 {
                for p in cur_path.iter().take(cur_path.len() - 1).skip(1) {
                    let add_ch = map.value(&p);
                    if *ch != add_ch && self.paths.contains_key(&add_ch) {
                        additional_keys.push(add_ch);
                    }
                }
            }

            keys.push((*ch, cur_path.len() - 1, additional_keys));
        }

        Ok(keys)
    }
}

pub struct Map {
    map: Vec<Vec<char>>,
}

impl Map {
    fn new() -> Self {
        Self { map: Vec::new() }
    }

    fn size(&self) -> PointU {
        let ysize = self.map.len();
        let xsize = if ysize > 0 { self.map[0].len() } else { 0 };

        PointU::new(xsize, ysize)
    }

    fn value(&self, pos: &PointU) -> char {
        self.map[pos.y][pos.x]
    }

    fn neighbors(&self, pos: &PointU, barriers: &str) -> Vec<PointU> {
        let mut result = Vec::new();

        let mut p1 = PointU::new(pos.x, pos.y - 1);
        if !barriers.contains(self.value(&p1)) {
            result.push(p1);
        }

        p1 = PointU::new(pos.x, pos.y + 1);
        if !barriers.contains(self.value(&p1)) {
            result.push(p1);
        }

        p1 = PointU::new(pos.x - 1, pos.y);
        if !barriers.contains(self.value(&p1)) {
            result.push(p1);
        }

        p1 = PointU::new(pos.x + 1, pos.y);
        if !barriers.contains(self.value(&p1)) {
            result.push(p1);
        }

        result
    }

    fn build_path(&self, p1: &PointU, p2: &PointU) -> Result<Vec<PointU>> {
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
            p1,
            |pos| self.neighbors(&pos, "#").into_iter().map(|p| (p, 1)),
            |pos| (pos.x as isize - p2.x as isize).abs() + (pos.y as isize - p2.y as isize).abs(),
            |pos| *p2 == *pos,
        )
        .unwrap_or((Vec::new(), 0))
        .0)
    }
}

pub struct Vault {
    map: Map,
    route: Route,
}

impl Vault {
    pub fn new(data: &str) -> Result<Self> {
        let mut res = Self {
            map: Map::new(),
            route: Route::new(),
        };

        res.load_map(data)?;

        Ok(res)
    }

    fn load_map(&mut self, data: &str) -> Result<()> {
        for line in data.lines() {
            let row = line.trim();

            let map_size = self.map.size();
            if map_size.y > 0 {
                ensure!(
                    map_size.x == row.len(),
                    "Wrong row size. Expected {} but was {}",
                    map_size.x,
                    row.len()
                );
            }

            for (x, c) in row.chars().enumerate() {
                if c == '@' {
                    self.route.start_pos.set(x, map_size.y);
                } else if c.is_alphabetic() {
                    let key = c.to_lowercase().to_string().chars().nth(0).unwrap();
                    let key_info = self
                        .route
                        .paths
                        .entry(key)
                        .or_insert_with(|| KeyInfo::WithDoor(KeyDoorPath::default()));

                    if let KeyInfo::WithDoor(path) = key_info {
                        if c.is_lowercase() {
                            (*path).key_pos.set(x, map_size.y);
                        } else {
                            (*path).door_pos.set(x, map_size.y);
                        }
                    } else {
                        unreachable!();
                    }
                }
            }

            self.map.map.push(row.chars().collect());
        }

        println!("Map: {:?}", self.map.size());

        // convert single keys
        for key_info in &mut self.route.paths.values_mut() {
            if let Some(door_pos) = key_info.door_pos() {
                if door_pos.x == std::usize::MAX && door_pos.y == std::usize::MAX {
                    key_info.to_single();
                }
            } else {
                unreachable!();
            }
        }

        // build paths
        for (ch, key_info) in &mut self.route.paths {
            if let KeyInfo::WithDoor(path) = key_info {
                path.path = self.map.build_path(&path.key_pos, &path.door_pos)?;

                println!("{}: {:?}", ch, path);
            }
        }

        Ok(())
    }

    pub fn find_shortest_path(&mut self) -> Result<(usize, Vec<char>)> {
        self.route.find_shortest_path(0, &self.map, std::usize::MAX)
    }
}
