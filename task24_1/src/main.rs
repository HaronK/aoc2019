use crate::biome_field::*;
use anyhow::Result;
use std::fs::File;
use std::io::prelude::*;

mod biome_field;

fn main() -> Result<()> {
    let content = get_file_content("input.txt")?;
    let mut field = BiomeField::new(&content)?;
    let rating = field.get_rating();

    println!("Rating: {}", rating);

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
        common_test(2129920,
            r#"....#
            #..#.
            #..##
            ..#..
            #...."#)
    }

    fn common_test(expected_rating: u64, data: &str) -> Result<()> {
        let mut field = BiomeField::new(data)?;
        let rating = field.get_rating();

        assert_eq!(expected_rating, rating);

        Ok(())
    }
}
