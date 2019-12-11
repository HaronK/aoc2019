use crate::amplifier::*;
use crate::log::*;
use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{prelude::*, BufReader};

mod amplifier;
pub mod intcode_comp;
pub mod log;

fn main() -> Result<()> {
    let log = Log::new(false);
    let file = File::open("input.txt")?;
    let reader = BufReader::new(file);
    let prog_str = reader
        .lines()
        .nth(0)
        .ok_or_else(|| anyhow!("ERROR: Cannot read program string."))??;

    let mut phase_settings = vec![5, 6, 7, 8, 9];
    let mut amplifier = Amplifier::new(&prog_str, &log)?;
    let mut output = amplifier.run(&phase_settings)?;

    while next_set(&mut phase_settings) {
        let o = amplifier.run(&phase_settings)?;

        if o > output {
            output = o;
        }
    }

    println!("Output: {}", output);

    Ok(())
}

fn next_set(values: &mut Vec<i32>) -> bool {
    let mut i = (values.len() - 2) as i32;

    while i >= 0 && values[i as usize] > values[(i + 1) as usize] {
        i -= 1;
    }

    if i < 0 {
        return false;
    }

    let mut j = i + 1;
    let mut k = (values.len() - 1) as i32;

    while j < k {
        values.swap(j as usize, k as usize);
        j += 1;
        k -= 1;
    }

    j = i + 1;
    while values[j as usize] < values[i as usize] {
        j += 1;
    }

    values.swap(i as usize, j as usize);

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() -> Result<()> {
        let log = Log::new(false);
        let prog_str =
            "3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5"
                .to_owned();
        let phase_settings = vec![9, 8, 7, 6, 5];
        let mut amplifier = Amplifier::new(&prog_str, &log)?;

        assert_eq!(139629729, amplifier.run(&phase_settings)?);

        Ok(())
    }

    #[test]
    fn test4() -> Result<()> {
        let mut data = vec![0, 1, 2];

        while next_set(&mut data) {
            println!("{:?}", data);
        }

        Ok(())
    }
}
