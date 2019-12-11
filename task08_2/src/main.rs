use anyhow::{ensure, Result};
use std::fs::File;
use std::io::prelude::*;

fn main() -> Result<()> {
    let mut content = String::new();
    let mut file = File::open("input.txt")?;

    file.read_to_string(&mut content)?;

    let result = process_image(content, 25, 6)?;

    println!("Result:");
    for r in 0..6 {
        let row = &result[(r * 25)..((r + 1) * 25)];
        let row_str = row
            .iter()
            .map(|p| if *p == 1 { "#" } else { " " })
            .collect::<Vec<&str>>()
            .join("");
        println!("  {:?}", row_str);
    }

    Ok(())
}

fn process_image<S: AsRef<str>>(image: S, width: usize, height: usize) -> Result<Vec<u8>> {
    let layer_size = width * height;
    let image_data: Vec<u8> = image.as_ref().as_bytes().iter().map(|p| p - 48).collect();
    ensure!(
        image_data.len() % layer_size == 0,
        "ERROR: Image data is corrupted."
    );

    let mut result = vec![2; layer_size];
    let layers_count = image_data.len() / layer_size;
    let mut remaining_pixels = layer_size;

    for l in 0..layers_count {
        let layer_data = &image_data[(l * layer_size)..((l + 1) * layer_size)];
        // println!("Layer {}: {:?}", l, layer_data);

        for p in 0..layer_size {
            // println!("    rp: {}, lp: {}", result[p], layer_data[p]);
            if result[p] == 2 && layer_data[p] != 2 {
                result[p] = layer_data[p];
                // println!("      change: {}", result[p]);
                remaining_pixels -= 1;
            }
        }
        // println!("  Update: {:?}", result);

        if remaining_pixels == 0 {
            break;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add1() -> Result<()> {
        assert_eq!(vec![0, 1, 1, 0], process_image("0222112222120000", 2, 2)?);
        Ok(())
    }
}
