use anyhow::{bail, Result};
use num_bigint::*;
use num_traits::cast::ToPrimitive;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
enum Technique {
    NewStack,
    Cut(isize),
    Incr(usize),
}

fn load_techniques(data: &str) -> Result<Vec<Technique>> {
    let mut result = Vec::new();

    for tech_str in data.lines() {
        let t_str = tech_str.trim();
        if t_str.is_empty() {
            break;
        }

        if t_str == "deal into new stack" {
            result.push(Technique::NewStack);
        } else if &t_str[0..4] == "cut " {
            result.push(Technique::Cut(t_str[4..].parse()?));
        } else if &t_str[0..20] == "deal with increment " {
            result.push(Technique::Incr(t_str[20..].parse()?));
        } else {
            bail!("Unsupported technique: '{}'", t_str);
        }
    }

    Ok(result)
}

fn find_position(deck_size: u128, card: u128, times: u128, techniques: &Vec<Technique>) -> u128 {
    let mut result = card;

    for tech in techniques {
        match tech {
            Technique::NewStack => {
                result = if times & 1 == 1 {
                    (deck_size as i128 - result as i128 - 1) as u128 % deck_size
                } else {
                    result
                }
            }
            Technique::Cut(m) => {
                result = (((deck_size as i128 - *m as i128) * times as i128) + result as i128).abs()
                    as u128
                    % deck_size
            }
            Technique::Incr(m) => {
                let m_b = m.to_biguint().unwrap();
                let times_b = times.to_biguint().unwrap();
                let deck_size_b = deck_size.to_biguint().unwrap();
                let m_pow = m_b.modpow(&times_b, &deck_size_b).to_u128().unwrap();

                result = result * m_pow % deck_size
            }
        }
    }

    result
}

fn main() -> Result<()> {
    let mut content = String::new();
    let mut file = File::open("input.txt")?;

    file.read_to_string(&mut content)?;

    let techniques = load_techniques(&content)?;

    // println!("Techniques: {:?}", techniques);

    let pos = find_position(10007, 2019, 1, &techniques);

    println!("Position: {}", pos);

    let mut card = 2020;
    let shuffle_times: u128 = 2; //101_741_582_076_661;
    let deck_size: u128 = 119_315_717_514_047;

    for i in 0..shuffle_times {
        let old_card = card;
        card = find_position(deck_size, card, 1, &techniques);
        // println!("{:7}: {:15} {:17}", i, card, card as i128 - old_card as i128);
    }
    println!("Loop: {}", card);

    card = find_position(deck_size, 2020, shuffle_times, &techniques);

    println!("Smart: {}", card);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        assert_eq!(6, find_position(10, 3, 1, &vec![Technique::NewStack]));
    }

    #[test]
    fn test2() {
        assert_eq!(0, find_position(10, 3, 1, &vec![Technique::Cut(3)]));
        assert_eq!(3, find_position(10, 6, 1, &vec![Technique::Cut(3)]));
        assert_eq!(7, find_position(10, 3, 1, &vec![Technique::Cut(-4)]));
        assert_eq!(
            vec![6, 7, 8, 9, 0, 1, 2, 3, 4, 5],
            apply2deck(10, &vec![Technique::Cut(-4)])
        );
        assert_eq!(
            vec![3, 4, 5, 6, 7, 8, 9, 0, 1, 2],
            apply2deck(10, &vec![Technique::Cut(3)])
        );
    }

    #[test]
    fn test3() {
        assert_eq!(9, find_position(10, 3, 1, &vec![Technique::Incr(3)]));
        assert_eq!(4, find_position(10, 8, 1, &vec![Technique::Incr(3)]));
        assert_eq!(
            vec![0, 7, 4, 1, 8, 5, 2, 9, 6, 3],
            apply2deck(10, &vec![Technique::Incr(3)])
        );
    }

    #[test]
    fn test4() {
        assert_eq!(
            vec![0, 3, 6, 9, 2, 5, 8, 1, 4, 7],
            apply2deck(
                10,
                &vec![Technique::Incr(7), Technique::NewStack, Technique::NewStack]
            )
        );
    }

    #[test]
    fn test5() {
        assert_eq!(
            vec![3, 0, 7, 4, 1, 8, 5, 2, 9, 6],
            apply2deck(
                10,
                &vec![Technique::Cut(6), Technique::Incr(7), Technique::NewStack]
            )
        );
    }

    #[test]
    fn test6() {
        assert_eq!(
            vec![6, 3, 0, 7, 4, 1, 8, 5, 2, 9],
            apply2deck(
                10,
                &vec![Technique::Incr(7), Technique::Incr(9), Technique::Cut(-2)]
            )
        );
    }

    #[test]
    fn test7() {
        assert_eq!(
            vec![9, 2, 5, 8, 1, 4, 7, 0, 3, 6],
            apply2deck(
                10,
                &vec![
                    Technique::NewStack,
                    Technique::Cut(-2),
                    Technique::Incr(7),
                    Technique::Cut(8),
                    Technique::Cut(-4),
                    Technique::Incr(7),
                    Technique::Cut(3),
                    Technique::Incr(9),
                    Technique::Incr(3),
                    Technique::Cut(-1)
                ]
            )
        );
    }

    fn apply2deck(deck_size: usize, techniques: &Vec<Technique>) -> Vec<usize> {
        let mut result = vec![0; deck_size];

        for i in 0..deck_size {
            result[find_position(deck_size as u128, i as u128, 1, &techniques) as usize] = i;
        }

        result
    }
}
