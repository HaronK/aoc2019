use anyhow::{ensure, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

fn main() -> Result<()> {
    let mut content = String::new();
    let mut file = File::open("input.txt")?;

    file.read_to_string(&mut content)?;

    let orbits = parse(&content)?;
    let total_transfers = get_min_orbit_transfers(&orbits)?;

    println!("Total transfers: {}", total_transfers);

    Ok(())
}

fn parse(content: &String) -> Result<HashMap<String, String>> {
    let mut orbits = HashMap::new();
    for line in content.split("\n") {
        let orbit: Vec<&str> = line.trim().split(")").collect();
        ensure!(
            orbit.len() == 2,
            "ERROR: Expected 2 objects per orbit but was {}.",
            orbit.len()
        );
        ensure!(
            !orbits.contains_key(&orbit[1].to_owned()),
            "ERROR: Trying to add {} orbit but {} is already on orbit around {}.",
            line,
            orbit[1],
            orbits[&orbit[1].to_owned()]
        );

        orbits.insert(orbit[1].to_owned(), orbit[0].to_owned());
    }
    Ok(orbits)
}

fn get_min_orbit_transfers(orbits: &HashMap<String, String>) -> Result<usize> {
    ensure!(
        orbits.contains_key(&"SAN".to_owned()),
        "ERROR: Cannot find SAN object on the any orbit."
    );
    ensure!(
        orbits.contains_key(&"YOU".to_owned()),
        "ERROR: Cannot find YOU object on the any orbit."
    );

    let san_orbits = get_indirect_orbits(&"SAN".to_owned(), orbits)?;
    let you_orbits = get_indirect_orbits(&"YOU".to_owned(), orbits)?;
    let mut i = 0;
    let n = std::cmp::min(san_orbits.len(), you_orbits.len());

    println!("SAN orbits: {:?}", san_orbits);
    println!("YOU orbits: {:?}", you_orbits);

    while i < n && san_orbits[i] == you_orbits[i] {
        i += 1;
    }

    Ok(san_orbits.len() - i + you_orbits.len() - i)
}

fn get_indirect_orbits(object: &String, orbits: &HashMap<String, String>) -> Result<Vec<String>> {
    let mut result = Vec::new();
    let mut obj = object;

    while let Some(o) = orbits.get(obj) {
        result.push(o.clone());
        obj = o;
    }

    result.reverse();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add1() -> Result<()> {
        let orbits = parse(
            &"COM)B\nB)C\nC)D\nD)E\nE)F\nB)G\nG)H\nD)I\nE)J\nJ)K\nK)L\nK)YOU\nI)SAN".to_owned(),
        )?;
        assert_eq!(4, get_min_orbit_transfers(&orbits)?);
        Ok(())
    }
}
