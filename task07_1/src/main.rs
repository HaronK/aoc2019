use crate::intcode_comp::*;
use anyhow::{anyhow, ensure, Result};
use std::fs::File;
use std::io::{prelude::*, BufReader};

mod intcode_comp;

fn main() -> Result<()> {
    let file = File::open("input.txt")?;
    let reader = BufReader::new(file);
    let prog_str = reader
        .lines()
        .nth(0)
        .ok_or_else(|| anyhow!("ERROR: Cannot read program string."))??;

    let mut phase_settings = vec![0, 1, 2, 3, 4];
    let mut output = run_amplifier(&prog_str, &phase_settings)?;

    while next_set(&mut phase_settings) {
        let o = run_amplifier(&prog_str, &phase_settings)?;

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

    return true;
}

fn run_amplifier(prog_str: &String, phase_settings: &Vec<i32>) -> Result<i32> {
    let steps = phase_settings.len();
    ensure!(steps > 0, "ERROR: No phase settings are set.");

    let mut result = 0;
    let mut comps = Vec::new();

    for _i in 0..steps {
        comps.push(IntcodeComp::new(prog_str)?);
    }

    for i in 0..steps {
        let output = comps[i].run(vec![phase_settings[i], result])?;
        ensure!(
            output.len() == 1,
            "ERROR: Comp {}. Expected 1 output but was {}.",
            i,
            output.len()
        );

        result = output[0];
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() -> Result<()> {
        let prog_str = "3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0".to_owned();
        let phase_settings = vec![4, 3, 2, 1, 0];

        assert_eq!(43210, run_amplifier(&prog_str, &phase_settings)?);

        Ok(())
    }

    #[test]
    fn test2() -> Result<()> {
        let prog_str =
            "3,23,3,24,1002,24,10,24,1002,23,-1,23,101,5,23,23,1,24,23,23,4,23,99,0,0".to_owned();
        let phase_settings = vec![0, 1, 2, 3, 4];

        assert_eq!(54321, run_amplifier(&prog_str, &phase_settings)?);

        Ok(())
    }

    #[test]
    fn test3() -> Result<()> {
        let prog_str = "3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0".to_owned();
        let phase_settings = vec![1, 0, 4, 3, 2];

        assert_eq!(65210, run_amplifier(&prog_str, &phase_settings)?);

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
