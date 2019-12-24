use crate::biome_field::*;
use anyhow::Result;
use std::fs::File;
use std::io::prelude::*;

mod biome_field;

fn main() -> Result<()> {
    let content = get_file_content("input.txt")?;
    let mut field = BiomeField::new(&content)?;
    let bugs_count = field.bugs_count(200);

    println!("Bugs count: {}", bugs_count);

    Ok(())
}

fn get_file_content(path: &str) -> Result<String> {
    let mut content = String::new();
    let mut file = File::open(path)?;

    file.read_to_string(&mut content)?;

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() -> Result<()> {
        common_test(
            99,
            10,
            r#"....#
            #..#.
            #..##
            ..#..
            #...."#,
        )
    }

    fn common_test(expected_bugs: usize, steps: usize, data: &str) -> Result<()> {
        let mut field = BiomeField::new(data)?;
        let bugs_count = field.bugs_count(steps);

        assert_eq!(expected_bugs, bugs_count);

        Ok(())
    }
}
