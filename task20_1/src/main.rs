use crate::maze::*;
use anyhow::Result;
use std::fs::File;
use std::io::prelude::*;

mod maze;

fn main() -> Result<()> {
    let content = get_file_content("input.txt")?;
    let maze = Maze::new(&content)?;

    // maze.dump_map();

    let dist = maze.find_shortest_path()?;

    println!("Shortest dist: {}", dist);

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
        common_test(23, "input1.txt")
    }

    #[test]
    fn test2() -> Result<()> {
        common_test(58, "input2.txt")
    }

    fn common_test(expected_dist: usize, path: &str) -> Result<()> {
        let content = get_file_content(path)?;
        let maze = Maze::new(&content)?;

        // maze.dump_map();

        let dist = maze.find_shortest_path()?;

        assert_eq!(expected_dist, dist);

        Ok(())
    }
}
