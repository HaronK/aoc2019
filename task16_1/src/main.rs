use anyhow::{anyhow, Result};
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

    let mut input2 = input.clone();
    input2.append(&mut input.clone());

    let output2 = apply_fft(100, &input2);

    println!("Output 2x: {:?}", output2);

    Ok(())
}

type SignalType = u8;
type PatternType = i8;

fn parse_signal(data: &str) -> Result<Vec<SignalType>> {
    let mut signal = Vec::new();
    let signal_chars: Vec<String> = data.chars().map(|c| c.to_string()).collect();

    for c in signal_chars {
        signal.push(c.parse()?);
    }

    Ok(signal)
}

fn get_pattern(pos: usize, len: usize) -> Vec<PatternType> {
    let base_pattern = vec![0, 1, 0, -1];
    let base_len = base_pattern.len();
    let mut pattern = Vec::new();

    for i in 0..len {
        let idx = base_pattern[((i + 1) / (pos + 1)) % base_len];
        pattern.push(idx);
    }

    pattern
}

fn apply_fft(phases: usize, input: &Vec<SignalType>) -> Vec<SignalType> {
    let mut result = input.clone();
    let len = input.len();

    for p in 0..phases {
        let mut buf = Vec::new();

        for i in 0..len {
            let pattern = get_pattern(i, result.len());
            let mut value = 0;

            for (j, sig) in result.iter().enumerate() {
                value += *sig as i32 * pattern[j] as i32;
            }

            buf.push((value.abs() % 10) as SignalType);
        }

        result = buf;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        assert_eq!(vec![1, 0, -1, 0, 1, 0, -1, 0], get_pattern(0, 8));
        assert_eq!(vec![0, 1, 1, 0, 0, -1, -1, 0], get_pattern(1, 8));
        assert_eq!(vec![0, 0, 1, 1, 1, 0, 0, 0], get_pattern(2, 8));
        assert_eq!(vec![0, 0, 0, 1, 1, 1, 1, 0], get_pattern(3, 8));
        assert_eq!(vec![0, 0, 0, 0, 1, 1, 1, 1], get_pattern(4, 8));
        assert_eq!(vec![0, 0, 0, 0, 0, 1, 1, 1], get_pattern(5, 8));
        assert_eq!(vec![0, 0, 0, 0, 0, 0, 1, 1], get_pattern(6, 8));
    }

    #[test]
    fn test2() {
        assert_eq!(vec![0,1,0,2,9,4,9,8], apply_fft(4, &vec![1,2,3,4,5,6,7,8]));
    }

    #[test]
    fn test3() -> Result<()> {
        let input = parse_signal("80871224585914546619083218645595")?;
        assert_eq!([2,4,1,7,6,1,7,6], apply_fft(100, &input)[0..8]);
        Ok(())
    }

    #[test]
    fn test4() -> Result<()> {
        let input = parse_signal("19617804207202209144916044189917")?;
        assert_eq!([7,3,7,4,5,4,1,8], apply_fft(100, &input)[0..8]);
        Ok(())
    }

    #[test]
    fn test5() -> Result<()> {
        let input = parse_signal("69317163492948606335995924319873")?;
        assert_eq!([5,2,4,3,2,1,3,3], apply_fft(100, &input)[0..8]);
        Ok(())
    }
}
