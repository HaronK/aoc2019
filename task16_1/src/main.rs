use anyhow::{anyhow, Result};
use rayon::prelude::*;
use std::fs::File;
use std::io::{prelude::*, BufReader};

fn main() -> Result<()> {
    let file = File::open("input.txt")?;
    let reader = BufReader::new(file);
    let data = reader
        .lines()
        .nth(0)
        .ok_or_else(|| anyhow!("ERROR: Cannot read program string."))??;

    let input = parse_signal(&data)?;
    let output = apply_fft(100, &input);

    println!("Output: {:?}", &output[0..8]);

    let output10k = apply_fft_dup(100, 10000, &input);

    println!("Output 10k: {:?}", &output10k[0..8]);

    Ok(())
}

type SignalType = i32;
type PatternType = i32;

fn parse_signal(data: &str) -> Result<Vec<SignalType>> {
    let mut signal = Vec::new();
    let signal_chars: Vec<String> = data.chars().map(|c| c.to_string()).collect();

    for c in signal_chars {
        signal.push(c.parse()?);
    }

    Ok(signal)
}

fn calc_offset(digits: &[SignalType]) -> usize {
    let mut offset: usize = 0;

    for d in digits {
        offset = offset * 10 + *d as usize;
    }

    offset
}

fn apply_fft_dup(phases: usize, count: usize, input: &Vec<SignalType>) -> Vec<SignalType> {
    let offset = calc_offset(&input[0..7]);
    let mut buf = Vec::with_capacity(input.len() * count);

    for _i in 0..count {
        buf.append(&mut input.clone());
    }

    let start_pos = buf.len() - offset;
    let start_pos2 = start_pos * 2;
    let mut input10k: Vec<SignalType> = buf[(buf.len() - start_pos2)..].iter().copied().collect();

    for _p in 0..phases {
        for i in (0..input10k.len() - 1).rev() {
            input10k[i] = (input10k[i] + input10k[i + 1]) % 10;
        }
    }

    input10k[start_pos..].iter().copied().collect()
}

fn apply_fft(phases: usize, input: &[SignalType]) -> Vec<SignalType> {
    let mut result: Vec<SignalType> = input.iter().copied().collect();
    let len = input.len();

    for _p in 0..phases {
        result = (0..len)
            .into_par_iter()
            .map(|i| {
                let mut value = 0;
                let mut pat_val: PatternType = 1;
                let count = i + 1;
                let mut j = i;

                while j < len {
                    let min_len = std::cmp::min(len, j + count);
                    for prev in result.iter().take(min_len).skip(j) {
                        value += prev * pat_val;
                    }
                    j += count * 2;
                    pat_val *= -1;
                }

                value.abs() % 10
            })
            .collect();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fft1() {
        assert_eq!(
            vec![0, 1, 0, 2, 9, 4, 9, 8],
            apply_fft(4, &vec![1, 2, 3, 4, 5, 6, 7, 8])
        );
    }

    #[test]
    fn test_fft2() -> Result<()> {
        let input = parse_signal("80871224585914546619083218645595")?;
        assert_eq!([2, 4, 1, 7, 6, 1, 7, 6], apply_fft(100, &input)[0..8]);
        Ok(())
    }

    #[test]
    fn test_fft3() -> Result<()> {
        let input = parse_signal("19617804207202209144916044189917")?;
        assert_eq!([7, 3, 7, 4, 5, 4, 1, 8], apply_fft(100, &input)[0..8]);
        Ok(())
    }

    #[test]
    fn test_fft4() -> Result<()> {
        let input = parse_signal("69317163492948606335995924319873")?;
        assert_eq!([5, 2, 4, 3, 2, 1, 3, 3], apply_fft(100, &input)[0..8]);
        Ok(())
    }

    #[test]
    fn test6() {
        assert_eq!(12345, calc_offset(&[1, 2, 3, 4, 5]));
    }

    #[test]
    fn test_dup1() -> Result<()> {
        let input = parse_signal("03036732577212944063491565474664")?;
        assert_eq!(
            [8, 4, 4, 6, 2, 0, 2, 6],
            apply_fft_dup(100, 10000, &input)[0..8]
        );
        Ok(())
    }

    #[test]
    fn test_dup2() -> Result<()> {
        let input = parse_signal("02935109699940807407585447034323")?;
        assert_eq!(
            [7, 8, 7, 2, 5, 2, 7, 0],
            apply_fft_dup(100, 10000, &input)[0..8]
        );
        Ok(())
    }

    #[test]
    fn test_dup3() -> Result<()> {
        let input = parse_signal("03081770884921959731165446850517")?;
        assert_eq!(
            [5, 3, 5, 5, 3, 7, 3, 1],
            apply_fft_dup(100, 10000, &input)[0..8]
        );
        Ok(())
    }
}
