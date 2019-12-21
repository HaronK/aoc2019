use crate::spring_droid::*;
use anyhow::{anyhow, Result};
use common::log::*;
use std::fs::File;
use std::io::{prelude::*, BufReader};

mod spring_droid;

fn main() -> Result<()> {
    let log = Log::new(false);
    let file = File::open("input.txt")?;
    let reader = BufReader::new(file);
    let prog_str = reader
        .lines()
        .nth(0)
        .ok_or_else(|| anyhow!("ERROR: Cannot read program string."))??;

    let mut droid = SpringDroid::new(&prog_str, &log)?;

    // Task 1
    // let dust_count = droid.move_droid(
    //     &["OR A J",
    //     "AND C J",
    //     "NOT J J",
    //     "AND D J",
    //     "WALK"]
    // )?;

    // Task 2
    let dust_count = droid.move_droid(&[
        "OR B J", "AND C J", "NOT J J", "AND D J", "AND H J", "NOT A T", "OR T J", "RUN",
    ])?;

    println!("Dust count: {}", dust_count);

    // let comb1 = vec![
    //     "AND A T", "AND A J", "AND B T", "AND B J", "AND C T", "AND C J", "AND D T", "AND D J", "AND T T", "AND T J", "AND J T", "AND J J",
    //     "OR A T", "OR A J", "OR B T", "OR B J", "OR C T", "OR C J", "OR D T", "OR D J", "OR T J", "OR J T", // "OR T T", "OR J J", - these are equivalent to AND versions
    //     "NOT A T", "NOT A J", "NOT B T", "NOT B J", "NOT C T", "NOT C J", "NOT D T", "NOT D J", "NOT T T", "NOT T J", "NOT J T", "NOT J J"];
    // // let comb1 = gen_combinations(&["AND", "OR", "NOT"], &["A", "B", "C", "D", "T", "J"], &["T", "J"]);

    // println!("Comb1[{}]: {:?}", comb1.len(), comb1);

    // let comb2 = vec![
    //     "AND A T", "AND A J", "AND B T", "AND B J", "AND C T", "AND C J", "AND D T", "AND D J", "AND E T", "AND E J", "AND F T", "AND F J", "AND G T", "AND G J", "AND H T", "AND H J", "AND I T", "AND I J", "AND T T", "AND T J", "AND J T", "AND J J",
    //     "OR A T", "OR A J", "OR B T", "OR B J", "OR C T", "OR C J", "OR D T", "OR D J", "OR E T", "OR E J", "OR F T", "OR F J", "OR G T", "OR G J", "OR H T", "OR H J", "OR I T", "OR I J", "OR T T", "OR T J", "OR J T", "OR J J",
    //     "NOT A T", "NOT A J", "NOT B T", "NOT B J", "NOT C T", "NOT C J", "NOT D T", "NOT D J", "NOT E T", "NOT E J", "NOT F T", "NOT F J", "NOT G T", "NOT G J", "NOT H T", "NOT H J", "NOT I T", "NOT I J", "NOT T T", "NOT T J", "NOT J T", "NOT J J"];
    // // let comb2 = gen_combinations(&["AND", "OR", "NOT"], &["A", "B", "C", "D", "E", "F", "G", "H", "I", "T", "J"], &["T", "J"]);

    // println!("Comb2[{}]: {:?}", comb2.len(), comb2);

    Ok(())
}

fn gen_combinations(v1: &[&str], v2: &[&str], v3: &[&str]) -> Vec<String> {
    let mut res = Vec::new();

    for s1 in v1 {
        for s2 in v2 {
            for s3 in v3 {
                let mut s = String::new();

                s += s1;
                s.push(' ');
                s += s2;
                s.push(' ');
                s += s3;

                res.push(s);
            }
        }
    }

    res
}
