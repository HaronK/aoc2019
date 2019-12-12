use anyhow::Result;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::ops::*;
// use termion;

fn main() -> Result<()> {
    let mut content = String::new();
    let mut file = File::open("input.txt")?;

    file.read_to_string(&mut content)?;

    let mut moons = parse(content)?;
    let period = get_full_turn_period(&mut moons);
    let total = total_energy(1000, &mut moons);

    println!("Moons:");
    dump_moons(&moons);
    println!("Period: {}", period);
    println!("Total energy: {}", total);

    Ok(())
}

fn dump_moons(moons: &[Moon]) {
    for moon in moons {
        println!("  {}", moon);
    }
}

fn total_energy(steps: usize, moons: &mut Vec<Moon>) -> usize {
    for _s in 0..steps {
        // println!("Step {}", s);
        // dump_moons(&moons);

        // println!("{}{}Step: ={}=", termion::clear::All, termion::cursor::Goto(1, 1), s);
        // std::io::stdout().flush().unwrap();

        process_once(moons);
    }

    // println!("After final step");
    // dump_moons(&moons);

    moons.iter().map(|m| m.energy()).sum()
}

fn get_full_turn_period(moons: &mut Vec<Moon>) -> usize {
    let mut period: usize = 0;
    let mut moons_period = vec![0; 3];
    let orig = moons.clone();

    loop {
        period += 1;
        process_once(moons);

        if moons_period[0] == 0 && moons.iter().enumerate().all(|(i, m)| m.cmp_x(&orig[i])) {
            moons_period[0] = period;
        }

        if moons_period[1] == 0 && moons.iter().enumerate().all(|(i, m)| m.cmp_y(&orig[i])) {
            moons_period[1] = period;
        }

        if moons_period[2] == 0 && moons.iter().enumerate().all(|(i, m)| m.cmp_z(&orig[i])) {
            moons_period[2] = period;
        }

        // println!("{}Step: {} Periods: {:?}", termion::cursor::Goto(1, 1), period, moons_period);
        // std::io::stdout().flush().unwrap();

        if moons_period.iter().all(|&p| p != 0) {
            break;
        }
    }

    let lcm1 = lcm(moons_period[0] + 1, moons_period[1] + 1);
    lcm(lcm1, moons_period[2] + 1)
}

fn process_once(moons: &mut Vec<Moon>) {
    let moons_len = moons.len();
    for i in 1..moons_len {
        let (left, right) = moons.split_at_mut(i);
        // println!("left={} right={}", left.len(), right.len());
        let this = &mut left[left.len() - 1];
        for mut other in right {
            this.update_vel(&mut other);
        }
        this.apply_vel();
    }
    moons[moons_len - 1].apply_vel();
}

fn lcm(v1: usize, v2: usize) -> usize {
    v1 * v2 / gcd(v1, v2)
    // while v1 != 0 {
    //     let old_v1 = v1;
    //     v1 = v2 % v1;
    //     v2 = old_v1;
    // }
    // v2
}

fn gcd(mut v1: usize, mut v2: usize) -> usize {
    while v1 != 0 {
        let old_v1 = v1;
        v1 = v2 % v1;
        v2 = old_v1;
    }
    v2
}

type CoordinateType = i64;

#[derive(Clone, PartialEq)]
struct Node3D {
    x: CoordinateType,
    y: CoordinateType,
    z: CoordinateType,
}

impl Node3D {
    fn new(x: CoordinateType, y: CoordinateType, z: CoordinateType) -> Self {
        Self { x, y, z }
    }
}

impl Default for Node3D {
    fn default() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }
}

impl AddAssign for Node3D {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        };
    }
}

impl Neg for Node3D {
    type Output = Node3D;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl fmt::Display for Node3D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}, {}, {}>", self.x, self.y, self.z)
    }
}

#[derive(Clone, PartialEq)]
struct Moon {
    pos: Node3D,
    vel: Node3D,
}

impl Moon {
    fn new(x: CoordinateType, y: CoordinateType, z: CoordinateType) -> Self {
        Self {
            pos: Node3D::new(x, y, z),
            vel: Node3D::default(),
        }
    }

    fn update_vel(&mut self, other: &mut Moon) {
        if self.pos.x < other.pos.x {
            self.vel.x += 1;
            other.vel.x -= 1;
        } else if self.pos.x > other.pos.x {
            self.vel.x -= 1;
            other.vel.x += 1;
        }
        if self.pos.y < other.pos.y {
            self.vel.y += 1;
            other.vel.y -= 1;
        } else if self.pos.y > other.pos.y {
            self.vel.y -= 1;
            other.vel.y += 1;
        }
        if self.pos.z < other.pos.z {
            self.vel.z += 1;
            other.vel.z -= 1;
        } else if self.pos.z > other.pos.z {
            self.vel.z -= 1;
            other.vel.z += 1;
        }
    }

    fn apply_vel(&mut self) {
        self.pos += self.vel.clone();
    }

    fn potential(&self) -> usize {
        (self.pos.x.abs() + self.pos.y.abs() + self.pos.z.abs()) as usize
    }

    fn kinetic(&self) -> usize {
        (self.vel.x.abs() + self.vel.y.abs() + self.vel.z.abs()) as usize
    }

    fn energy(&self) -> usize {
        self.potential() * self.kinetic()
    }

    fn cmp_x(&self, other: &Moon) -> bool {
        self.pos.x == other.pos.x
    }

    fn cmp_y(&self, other: &Moon) -> bool {
        self.pos.y == other.pos.y
    }

    fn cmp_z(&self, other: &Moon) -> bool {
        self.pos.z == other.pos.z
    }
}

impl fmt::Display for Moon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pos={}, vel={}", self.pos, self.vel)
    }
}

fn parse<S: AsRef<str>>(positions: S) -> Result<Vec<Moon>> {
    let mut moons = Vec::new();

    for position in positions.as_ref().lines() {
        let pos_str = position.trim();
        if pos_str.is_empty() {
            break;
        }

        let coord_str = &pos_str[1..pos_str.len() - 1];
        let coord_vec: Vec<&str> = coord_str.split(',').map(|c| c.trim()).collect();
        let x: CoordinateType = coord_vec[0][2..].parse()?;
        let y: CoordinateType = coord_vec[1][2..].parse()?;
        let z: CoordinateType = coord_vec[2][2..].parse()?;

        moons.push(Moon::new(x, y, z));
    }

    Ok(moons)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() -> Result<()> {
        common_test(
            179,
            2772,
            10,
            r#"<x=-1, y=0, z=2>
        <x=2, y=-10, z=-7>
        <x=4, y=-8, z=8>
        <x=3, y=5, z=-1>"#,
        )
    }

    #[test]
    fn test2() -> Result<()> {
        common_test(
            1940,
            4_686_774_924,
            100,
            r#"<x=-8, y=-10, z=0>
            <x=5, y=5, z=10>
            <x=2, y=-7, z=3>
            <x=9, y=-8, z=-3>"#,
        )
    }

    fn common_test(
        expected_total: usize,
        expected_period: usize,
        steps: usize,
        moons_str: &str,
    ) -> Result<()> {
        let mut moons = parse(moons_str)?;
        assert_eq!(expected_period, get_full_turn_period(&mut moons));
        moons = parse(moons_str)?;
        assert_eq!(expected_total, total_energy(steps, &mut moons));
        Ok(())
    }
}
