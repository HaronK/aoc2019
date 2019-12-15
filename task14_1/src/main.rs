use anyhow::{ensure, Result};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

type QuantityType = u64;
type Chemicals = HashMap<String, QuantityType>;

struct Reaction {
    quantity: QuantityType,
    chemicals: Chemicals,
}

impl Reaction {
    fn new(quantity: QuantityType, chemicals: Chemicals) -> Self {
        Self {
            quantity,
            chemicals,
        }
    }
}

struct NanoFactory {
    reactions: HashMap<String, Reaction>,
}

impl NanoFactory {
    fn new(reactions_data: &str) -> Result<Self> {
        let mut res = Self {
            reactions: HashMap::new(),
        };

        res.load_reactions(reactions_data)?;

        Ok(res)
    }

    fn load_reactions(&mut self, data: &str) -> Result<()> {
        for react_str in data.lines() {
            let r_str = react_str.trim();
            if r_str.is_empty() {
                break;
            }

            let r_pair: Vec<&str> = r_str.split("=>").map(|c| c.trim()).collect();
            ensure!(r_pair.len() == 2, "Wrong reaction format: {}", r_str);
            ensure!(!r_pair[0].is_empty(), "Wrong chemicals format: {}", r_str);
            ensure!(
                !r_pair[1].is_empty(),
                "Wrong resulting chemical format: {}",
                r_str
            );

            let res = NanoFactory::parse_chemical(r_pair[1])?;
            let ch_pairs: Vec<&str> = r_pair[0].split(',').map(|c| c.trim()).collect();

            let mut chemicals = HashMap::new();
            for ch_pair in ch_pairs {
                let ch = NanoFactory::parse_chemical(ch_pair)?;
                chemicals.insert(ch.0, ch.1);
            }

            self.reactions
                .insert(res.0, Reaction::new(res.1, chemicals));
        }

        Ok(())
    }

    fn parse_chemical(data: &str) -> Result<(String, QuantityType)> {
        let ch_pair: Vec<&str> = data.split(' ').map(|c| c.trim()).collect();
        ensure!(ch_pair.len() == 2, "Wrong reaction format: {}", data);
        ensure!(!ch_pair[0].is_empty(), "Chemical amount is empty: {}", data);
        ensure!(!ch_pair[1].is_empty(), "Chemical name is empty: {}", data);

        Ok((ch_pair[1].to_string(), ch_pair[0].parse()?))
    }

    fn calc_ore(&self, name: &str, quant: QuantityType) -> QuantityType {
        let mut resources = Chemicals::new();
        let mut storage = Chemicals::new();
        let mut workpad1 = Chemicals::new();
        workpad1.insert(name.to_string(), quant);

        while !workpad1.is_empty() {
            let mut workpad2 = Chemicals::new();

            for (ch_name, ch_quant) in workpad1.drain() {
                if self.reactions.contains_key(&ch_name) {
                    let st_remains = storage.entry(ch_name.clone()).or_insert(0);

                    if *st_remains >= ch_quant {
                        *st_remains -= ch_quant;
                    } else {
                        let req_quant = ch_quant - *st_remains;
                        let min_quant = self.reactions[&ch_name].quantity;

                        let mut react_count = req_quant / min_quant;
                        if req_quant % min_quant > 0 {
                            react_count += 1;
                        }

                        NanoFactory::add_chemical(
                            &ch_name,
                            react_count * min_quant,
                            &mut resources,
                        );

                        *st_remains = react_count * min_quant - req_quant;

                        for (ch_n, ch_q) in &self.reactions[&ch_name].chemicals {
                            NanoFactory::add_chemical(&ch_n, react_count * ch_q, &mut workpad2);
                        }
                    }
                } else {
                    NanoFactory::add_chemical(&ch_name, ch_quant, &mut resources);
                }
            }

            workpad1 = workpad2;
        }

        // println!("Storage: {:?}", storage);
        // println!("Resources: {:?}", resources);

        resources["ORE"]
    }

    fn calc_fuel(&self, avail_ore: QuantityType) -> QuantityType {
        let ore_quant = self.calc_ore("FUEL", 1);
        let mut fuel_quant = avail_ore / ore_quant;

        loop {
            let ore_q = self.calc_ore("FUEL", fuel_quant);

            if ore_q > avail_ore {
                break;
            }

            let extra_fuel = (avail_ore - ore_q) / ore_quant;
            fuel_quant += if extra_fuel > 0 { extra_fuel } else { 1 };
        }

        fuel_quant - 1
    }

    fn add_chemical(name: &str, quant: QuantityType, dest: &mut Chemicals) {
        let d = dest.entry(name.to_string()).or_insert(0);
        *d += quant;
    }
}

impl fmt::Display for NanoFactory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Reactions:")?;
        for (name, react) in &self.reactions {
            write!(f, "  {}<{}>:", name, react.quantity)?;

            for (ch_name, quant) in &react.chemicals {
                write!(f, " {}<{}>", ch_name, quant)?;
            }

            writeln!(f)?;
        }
        // writeln!(f, "Storage: {:?}", self.storage)
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut content = String::new();
    let mut file = File::open("input.txt")?;

    file.read_to_string(&mut content)?;

    let factory = NanoFactory::new(&content)?;

    // println!("{}", factory);

    let ore_quant = factory.calc_ore("FUEL", 1);

    println!("ORE quantity: {}", ore_quant);

    let fuel_quant = factory.calc_fuel(1_000_000_000_000);

    println!("FUEL quantity: {}", fuel_quant);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() -> Result<()> {
        common_test(
            31,
            34482758620,
            r#"10 ORE => 10 A
            1 ORE => 1 B
            7 A, 1 B => 1 C
            7 A, 1 C => 1 D
            7 A, 1 D => 1 E
            7 A, 1 E => 1 FUEL"#,
        )
    }

    #[test]
    fn test2() -> Result<()> {
        common_test(
            165,
            6323777403,
            r#"9 ORE => 2 A
            8 ORE => 3 B
            7 ORE => 5 C
            3 A, 4 B => 1 AB
            5 B, 7 C => 1 BC
            4 C, 1 A => 1 CA
            2 AB, 3 BC, 4 CA => 1 FUEL"#,
        )
    }

    #[test]
    fn test3() -> Result<()> {
        common_test(
            13312,
            82892753,
            r#"157 ORE => 5 NZVS
            165 ORE => 6 DCFZ
            44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
            12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
            179 ORE => 7 PSHF
            177 ORE => 5 HKGWZ
            7 DCFZ, 7 PSHF => 2 XJWVT
            165 ORE => 2 GPVTF
            3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT"#,
        )
    }

    #[test]
    fn test4() -> Result<()> {
        common_test(
            180697,
            5586022,
            r#"2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
            17 NVRVD, 3 JNWZP => 8 VPVL
            53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
            22 VJHF, 37 MNCFX => 5 FWMGM
            139 ORE => 4 NVRVD
            144 ORE => 7 JNWZP
            5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
            5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
            145 ORE => 6 MNCFX
            1 NVRVD => 8 CXFTF
            1 VJHF, 6 MNCFX => 4 RFSQX
            176 ORE => 6 VJHF"#,
        )
    }

    #[test]
    fn test5() -> Result<()> {
        common_test(
            2210736,
            460664,
            r#"171 ORE => 8 CNZTR
            7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
            114 ORE => 4 BHXH
            14 VRPVC => 6 BMBT
            6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
            6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
            15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
            13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
            5 BMBT => 4 WPTQ
            189 ORE => 9 KTJDG
            1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
            12 VRPVC, 27 CNZTR => 2 XDBXC
            15 KTJDG, 12 BHXH => 5 XCVML
            3 BHXH, 2 VRPVC => 7 MZWV
            121 ORE => 7 VRPVC
            7 XCVML => 6 RJRHP
            5 BHXH, 4 VRPVC => 5 LTCX"#,
        )
    }

    fn common_test(
        expected_ore: QuantityType,
        expected_fuel: QuantityType,
        react_str: &str,
    ) -> Result<()> {
        let factory = NanoFactory::new(react_str)?;
        assert_eq!(expected_ore, factory.calc_ore("FUEL", 1));
        assert_eq!(expected_fuel, factory.calc_fuel(1_000_000_000_000));
        Ok(())
    }
}
