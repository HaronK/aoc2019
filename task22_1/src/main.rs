use anyhow::{bail, Result};
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

fn find_position(deck_size: usize, card: usize, techniques: &Vec<Technique>) -> usize {
    let mut result = card;

    for tech in techniques {
        match tech {
            Technique::NewStack => result = deck_size - result - 1,
            Technique::Cut(m) => {
                result = ((deck_size + result) as isize - *m).abs() as usize % deck_size
            }
            Technique::Incr(m) => result = result * m % deck_size,
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

    let pos = find_position(10007, 2019, &techniques);

    println!("Position: {}", pos);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        assert_eq!(6, find_position(10, 3, &vec![Technique::NewStack]));
    }

    #[test]
    fn test2() {
        assert_eq!(0, find_position(10, 3, &vec![Technique::Cut(3)]));
        assert_eq!(3, find_position(10, 6, &vec![Technique::Cut(3)]));
        assert_eq!(7, find_position(10, 3, &vec![Technique::Cut(-4)]));
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
        assert_eq!(9, find_position(10, 3, &vec![Technique::Incr(3)]));
        assert_eq!(4, find_position(10, 8, &vec![Technique::Incr(3)]));
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
            result[find_position(deck_size, i, &techniques)] = i;
        }

        result
    }
}
