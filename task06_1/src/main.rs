use anyhow::{ensure, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

fn main() -> Result<()> {
    let mut content = String::new();
    let mut file = File::open("input.txt")?;

    file.read_to_string(&mut content)?;

    let orbits = parse(&content)?;
    let total_orbits = get_total_orbits(&orbits)?;

    println!("Total orbits: {}", total_orbits);

    Ok(())
}

fn parse(content: &String) -> Result<HashMap<String, String>> {
    let mut orbits = HashMap::new();
    for line in content.split("\n") {
        let orbit: Vec<&str> = line.trim().split(")").collect();
        ensure!(
            orbit.len() == 2,
            format!(
                "ERROR: Expected 2 objects per orbit but was {}.",
                orbit.len()
            )
        );
        ensure!(
            !orbits.contains_key(&orbit[1].to_owned()),
            format!(
                "ERROR: Trying to add {} orbit but {} is already on orbit around {}.",
                line,
                orbit[1],
                orbits[&orbit[1].to_owned()]
            )
        );

        orbits.insert(orbit[1].to_owned(), orbit[0].to_owned());
    }
    Ok(orbits)
}

fn get_total_orbits(orbits: &HashMap<String, String>) -> Result<u32> {
    let mut result = 0;

    for (_k, v) in orbits {
        result += get_indirect_orbits(v, orbits)? + 1;
    }

    Ok(result)
}

fn get_indirect_orbits(object: &String, orbits: &HashMap<String, String>) -> Result<u32> {
    let mut result = 0;
    let mut obj = object;

    while let Some(o) = orbits.get(obj) {
        result += 1;
        obj = o;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add1() -> Result<()> {
        let orbits = parse(&"COM)B\nB)C\nC)D\nD)E\nE)F\nB)G\nG)H\nD)I\nE)J\nJ)K\nK)L".to_owned())?;
        assert_eq!(42, get_total_orbits(&orbits)?);
        Ok(())
    }
}
