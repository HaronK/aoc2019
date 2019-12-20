use crate::vault::*;
use anyhow::{anyhow, ensure, Result};
use common::point::*;
use pathfinding::prelude::astar;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{prelude::*, BufReader};

mod vault;

fn main() -> Result<()> {
    let file = File::open("input.txt")?;
    let reader = BufReader::new(file);
    let data = reader
        .lines()
        .nth(0)
        .ok_or_else(|| anyhow!("ERROR: Cannot read program string."))??;

    let mut vault = Vault::new(&data)?;
    let (dist, keys) = vault.find_shortest_path()?;

    println!("Shortest path[{}]: {:?}", dist, keys);

    Ok(())
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
        for line in data.lines() {
            println!("{}", line.trim());
        }

        let mut vault = Vault::new(&data)?;
        let (dist, keys) = vault.find_shortest_path()?;

        assert_eq!(*expected_keys, keys);
        assert_eq!(expected_dist, dist);

        Ok(())
    }
}
