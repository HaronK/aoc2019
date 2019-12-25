use anyhow::{bail, ensure, Result};
use common::dynamic_map::*;
use common::intcode_comp::*;
use common::log::*;
use common::point::*;
use std::fmt;
use std::io;
use std::io::Write;
use std::{thread, time};
use termion;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

#[derive(Clone, PartialEq)]
enum Cell {
    Undefined,
    Empty,
    Wall,
    Item(String),
    Pressure,
}

impl CellDisplay for Cell {
    fn display(&self) -> char {
        match self {
            Self::Undefined => ' ',
            Self::Empty => '░',
            Self::Wall => '█',
            Self::Item(_) => '●',
            Self::Pressure => 'P',
        }
    }

    fn start(&self) -> Option<char> {
        Some('★')
    }

    fn current(&self) -> Option<char> {
        Some('⛑')
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::Undefined
    }
}

struct OutputData {
    text: String,
    doors: Vec<Direction>,
    items: Vec<String>,
    cmd: bool,
    go_back: bool,
}

impl OutputData {
    fn new(data: &Vec<&str>) -> Result<Self> {
        let mut read_doors = false;
        let mut read_items = false;
        let mut text = String::new();
        let mut doors = Vec::new();
        let mut items = Vec::new();
        let mut cmd = false;
        let mut go_back = false;

        for line in data {
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            if read_doors && line[0..2] == *"- " {
                let dir = match &line[2..] {
                    "north" => Direction::North,
                    "south" => Direction::South,
                    "west" => Direction::West,
                    "east" => Direction::East,
                    _ => bail!("Unknown direction: '{}'", &line[2..]),
                };
                doors.push(dir);
            } else if read_items && line[0..2] == *"- " {
                items.push(line[2..].to_string());
            } else if line == "Doors here lead:" {
                read_doors = true;
                read_items = false;
            } else if line == "Items here:" {
                read_items = true;
                read_doors = false;
            } else if line == "== Pressure-Sensitive Floor ==" {
                go_back = true;
            } else if line == "Command?" {
                cmd = true;
                break;
            } else {
                text += line;
                text += "\n";
                read_items = false;
                read_doors = false;
            }
        }

        Ok(Self { text: text.trim().to_string(), doors, items, cmd, go_back })
    }
}

pub struct Droid<'l> {
    comp: IntcodeComp<'l>,
    map: DynamicMap<Cell>,
}

impl<'l> Droid<'l> {
    pub fn new(prog: &str, log: &'l Log) -> Result<Self> {
        let mut comp = IntcodeComp::new(Vec::new(), log);

        comp.load_prog(prog)?;

        Ok(Self {
            comp,
            map: DynamicMap::new(),
        })
    }

    fn run_iter(&mut self) -> Result<OutputData> {
        self.comp.run()?;

        let output = self.comp.get_output();
        let output_str = Self::output2str(&output);
        let output_vec: Vec<&str> = output_str.trim().lines().map(|l| l.trim()).collect();

        // println!("Output:\n{}", output_str);

        Ok(OutputData::new(&output_vec)?)
    }

    pub fn interactive(&mut self) -> Result<()> {
        let mut stdout = io::stdout().into_raw_mode()?;
        let mut stdin = io::stdin().keys();

        self.begin_show(&mut stdout)?;

        // self.show(&mut stdout)?;
        // stdout.lock().flush()?;

        let mut inventory = Vec::new();
        let mut prev_dir = Direction::North;
        let mut prev_inv = false;
        let mut prev_doors = Vec::new();
        let mut prev_items = Vec::new();

        'iter: loop {
            let mut output = self.run_iter()?;

            if prev_inv {
                output.doors = prev_doors.clone();
                output.items = prev_items.clone();
                prev_inv = false;
            }

            for dir in &output.doors {
                if self.map.get_cell_dir(dir) == Cell::Undefined {
                    self.map.set_cell_dir(dir, Cell::Empty);
                }
            }

            if output.go_back {
                self.map.set_cell(Cell::Pressure);
                self.map.do_move(&prev_dir.opposite());

            } else if !output.items.is_empty() {
                self.map.set_cell(Cell::Item(output.items[0].clone()));
            }

            self.show(&mut stdout, &format!("{}\nDoors: {:?}\nItems: {:?}\nInventory: {:?}",
                output.text, output.doors, output.items, inventory))?;
            stdout.lock().flush()?;

            // println!("[{}] Doors: {:?} Items: {:?}", output.text, output.doors, output.items);

            if !output.cmd {
                println!("Exit");
                break;
            }

            let mut correct_action = false;

            while !correct_action {
                // Read input (if any)
                let input = stdin.next();

                // If a key was pressed
                if let Some(c) = input {
                    correct_action = match c? {
                        Key::Up => {
                            if output.doors.iter().any(|d| *d == Direction::North) {
                                self.comp.add_input_vec(&mut Self::str2input("north"));
                                self.map.do_move(&Direction::North);
                                prev_dir = Direction::North;
                                true
                            } else {
                                false
                            }
                        }
                        Key::Down => {
                            if output.doors.iter().any(|d| *d == Direction::South) {
                                self.comp.add_input_vec(&mut Self::str2input("south"));
                                self.map.do_move(&Direction::South);
                                prev_dir = Direction::South;
                                true
                            } else {
                                false
                            }
                        }
                        Key::Left => {
                            if output.doors.iter().any(|d| *d == Direction::West) {
                                self.comp.add_input_vec(&mut Self::str2input("west"));
                                self.map.do_move(&Direction::West);
                                prev_dir = Direction::West;
                                true
                            } else {
                                false
                            }
                        }
                        Key::Right => {
                            if output.doors.iter().any(|d| *d == Direction::East) {
                                self.comp.add_input_vec(&mut Self::str2input("east"));
                                self.map.do_move(&Direction::East);
                                prev_dir = Direction::East;
                                true
                            } else {
                                false
                            }
                        }
                        Key::Char('i') => {
                            self.comp.add_input_vec(&mut Self::str2input("inv"));
                            prev_doors = output.doors.clone();
                            prev_items = output.items.clone();
                            prev_inv = true;
                            true
                        }
                        Key::Char('t') => {
                            if !output.items.is_empty() {
                                let item = output.items.remove(0);
                                let take = format!("take {}", item);
                                inventory.push(item);
                                self.comp.add_input_vec(&mut Self::str2input(&take));
                                self.map.set_cell(Cell::Empty);
                                prev_doors = output.doors.clone();
                                prev_items = output.items.clone();
                                prev_inv = true;
                                true
                            } else {
                                false
                            }
                        }
                        Key::Char('d') => {
                            let inv_idx = if !inventory.is_empty() {
                                if inventory.len() == 1 {
                                    Some (0)
                                } else {
                                    print!("Choose item[0..{}]: ", inventory.len() - 1);

                                    let inv_sel = stdin.next();

                                    if let Some(sel) = inv_sel {
                                        match sel? {
                                            Key::Char(ch) => {
                                                let idx = ch as usize - '0' as usize;
                                                if idx < inventory.len() {
                                                    Some(idx)
                                                } else {
                                                    None
                                                }
                                            }
                                            _ => None,
                                        }
                                    } else {
                                        None
                                    }
                                }
                            } else {
                                None
                            };

                            if let Some(idx) = inv_idx {
                                let item = inventory.remove(idx);
                                let drop = format!("drop {}", item);
                                self.comp.add_input_vec(&mut Self::str2input(&drop));
                                self.map.set_cell(Cell::Item(drop));
                                prev_doors = output.doors.clone();
                                prev_items = output.items.clone();
                                prev_inv = true;
                                true
                            } else {
                                false
                            }
                        }
                        Key::Esc => {
                            break 'iter;
                        }
                        _ => false,
                    };
                }
            }
        }

        self.end_show(&mut stdout)?;

        Ok(())
    }

    fn begin_show(&self, f: &mut dyn io::Write) -> Result<()> {
        write!(f, "{}", termion::cursor::Hide)?;

        Ok(())
    }

    fn show(&self, f: &mut dyn io::Write, msg: &String) -> Result<()> {
        write!(f, "{}", termion::clear::All)?;

        self.map.show_with_msg(f, msg)?;

        // let (_, ys) = self.map.size();
        // write!(f, "{}Current cell: '{}'\n", termion::cursor::Goto(1, ys as u16 + 1), self.map.get_cell().display())?;
        // write!(f, "{}Arrows to move, Esc to exit.", termion::cursor::Goto(1, ys as u16 + 2))?;
        // println!();

        Ok(())
    }

    fn end_show(&self, f: &mut dyn io::Write) -> Result<()> {
        write!(f, "{}", termion::cursor::Show)?;

        Ok(())
    }

    fn str2input(data: &str) -> Vec<DataType> {
        let mut result: Vec<DataType> = data.chars().map(|c| c as u8 as DataType).collect();

        result.push(10);

        result
    }

    fn output2str(data: &[DataType]) -> String {
        let mut res = String::new();

        for v in data {
            if *v == 10 {
                res.push('\n');
            } else {
                res.push(*v as u8 as char);
            }
        }

        res
    }
}