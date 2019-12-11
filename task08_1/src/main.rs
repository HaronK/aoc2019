use anyhow::{ensure, Result};
use std::fs::File;
use std::io::prelude::*;

fn main() -> Result<()> {
    let mut content = String::new();
    let mut file = File::open("input.txt")?;

    file.read_to_string(&mut content)?;

    let result = process_image(content, 25, 6)?;

    println!("Result: {}", result);

    Ok(())
}

fn process_image<S: AsRef<str>>(image: S, width: usize, height: usize) -> Result<u32> {
    let layer_size = width * height;
    let image_data = image.as_ref().as_bytes();
    ensure!(
        image_data.len() % layer_size == 0,
        "ERROR: Image data is corrupted."
    );

    let layers_count = image_data.len() / layer_size;
    let mut digit0_min_count = std::u32::MAX;
    let mut min_layer_idx = 0;

    for i in 0..layers_count {
        let digit0_count =
            get_digits_count(&image_data[(i * layer_size)..((i + 1) * layer_size)], 0);
        // println!("Layer: {}, zeros: {}", i, digit0_count);

        if digit0_count < digit0_min_count {
            digit0_min_count = digit0_count;
            min_layer_idx = i;
        }
    }

    let min_layer = &image_data[(min_layer_idx * layer_size)..((min_layer_idx + 1) * layer_size)];
    let digit1_count = get_digits_count(min_layer, 1);
    let digit2_count = get_digits_count(min_layer, 2);
    // println!("Min layer{}: {:?}, ones: {}, twos: {}", min_layer_idx, min_layer, digit1_count, digit2_count);

    Ok(digit1_count * digit2_count)
}

fn get_digits_count(layer: &[u8], digit: u8) -> u32 {
    let mut result = 0;

    for pixel in layer {
        if *pixel - 48 == digit {
            result += 1;
        }
    }

    return result;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add1() -> Result<()> {
        assert_eq!(1, process_image("123456789012", 3, 2)?);
        Ok(())
    }
}
