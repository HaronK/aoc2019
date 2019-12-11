use anyhow::{bail, Result};

fn main() -> Result<()> {
    let count = pass_count("402328-864247")?;
    println!("Result: {}", count);

    Ok(())
}

fn pass_count(range: &str) -> Result<u32> {
    let range_val_str: Vec<&str> = range.split('-').collect();
    if range_val_str.len() != 2 {
        bail!(
            "ERROR: Expected 2 values of range data but got {}",
            range_val_str.len()
        );
    }

    let r1: u32 = range_val_str[0].parse()?;
    let r2: u32 = range_val_str[1].parse()?;
    if r1 > r2 {
        bail!(
            "ERROR: Left range value {} is bigger then right one {}",
            r1,
            r2
        );
    }
    if r1 > 999_999 || r2 > 999_999 {
        bail!("ERROR: Only 6 digits numbers are allowed.");
    }

    let mut result: u32 = 0;
    for i in r1..=r2 {
        if check_pass(i) {
            result += 1;
        // println!("  + {}", i);
        } else {
            // println!("  - {}", i);
        }
    }

    Ok(result)
}

fn check_pass(pass: u32) -> bool {
    let mut p = pass;
    let mut prev_d: u32 = 0;
    let mut double_num_exists = false;
    let mut matching_count: u32 = 1;

    // println!("check_pass({})", pass);

    for i in 0..6 {
        let d = p % 10;
        // println!("  d={}", d);
        if i > 0 {
            if d > prev_d {
                // println!("  false: {} > {}", d, prev_d);
                return false;
            }

            if !double_num_exists {
                if prev_d == d {
                    matching_count += 1;
                // println!("  matching_count={}", matching_count);
                } else if matching_count > 1 {
                    if matching_count == 2 {
                        // println!("  true/2");
                        double_num_exists = true;
                    }
                    matching_count = 1;
                }
            }
        }
        prev_d = d;
        p /= 10;
    }

    // if double_num_exists || matching_count == 2 {
    //     println!("  + {}", pass);
    // }
    // else {
    //     println!("  - {}", pass);
    // }

    double_num_exists || matching_count == 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        assert_eq!(true, check_pass(112233));
    }

    #[test]
    fn test2() {
        assert_eq!(false, check_pass(123444));
    }

    #[test]
    fn test3() {
        assert_eq!(true, check_pass(111122));
    }

    #[test]
    fn test4() {
        assert_eq!(true, check_pass(788999));
    }

    #[test]
    fn test5() {
        assert_eq!(true, check_pass(445555));
    }
}
