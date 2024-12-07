use bevy::prelude::*;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::ops::{Deref, DerefMut};
use std::time::Duration;
use std::thread;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug, PartialEq, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RoomSpace {
    Guard(Direction),
    Obstacle,
    Visited,
    Empty,
}

impl Display for RoomSpace {
    fn fmt(&self, fmt:&mut Formatter) -> Result<(), std::fmt::Error> {
        fmt.write_str(match self {
            RoomSpace::Guard(dir) => match dir {
                Direction::Up => "^",
                Direction::Down => "v",
                Direction::Left => "<",
                Direction::Right => ">",
            },
            RoomSpace::Obstacle => "#",
            RoomSpace::Visited => ".",
            RoomSpace::Empty => " ",
        })
    }
}

#[derive(Debug, PartialEq, Resource, Clone)]
pub struct Room(Vec<Vec<RoomSpace>>);

impl Room {
    pub fn new() -> Room {
        Room(Vec::new())
    }
    pub fn from_file<P: AsRef<Path>>(filepath: P) -> io::Result<Room> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);

        let mut room = Room::new();

        for line in reader.lines() {
            let line = line?;
            let mut row:Vec<RoomSpace> = Vec::new();
            for c in line.chars() {
                row.push(match c {
                    '^' => RoomSpace::Guard(Direction::Up),
                    '#' => RoomSpace::Obstacle,
                    _ => RoomSpace::Empty,
                });
            }
            room.push(row);
        }
        // fix x and y...
        let mut newroom = Room::new();
        for i in 0..room[0].len() {
            let mut newrow = Vec::new();
            room.iter().for_each(|row|newrow.push(row[i].clone()));
            newroom.push(newrow);
        };
        Ok(newroom)
    }
    pub fn apply_trail(&self, trail: &Trail, with_guard: bool) -> Room {
        let mut newroom = self.clone();
        let mut newtrail: Trail = trail.clone();
        if let Some((dir,(gx,gy))) = &newtrail.pop() {
            for (_,(x,y)) in newtrail.iter() {
                newroom.visit_space(*x,*y);
            };
            if with_guard {
                newroom.add_guard(*gx,*gy, dir);
            } else {
                newroom.visit_space(*gx,*gy);
            };
        };
        newroom
    }
    pub fn add_obstacle(&mut self, x:usize, y:usize) {
        self[x][y] = RoomSpace::Obstacle;
    }
    pub fn add_guard(&mut self, x:usize, y:usize, d:&Direction) {
        self[x][y] = RoomSpace::Guard(d.clone());
    }
    pub fn visit_space(&mut self, x:usize, y:usize) {
        self[x][y] = RoomSpace::Visited;
    }
    pub fn find_guard(&self) -> Option<(Direction,(usize,usize))> {
        for (i, _) in self.iter().enumerate() {
            for (j, item) in self[i].iter().enumerate() {
                match item {
                    RoomSpace::Guard(dir) => {
                        return Some((dir.clone(),(i,j)));
                    },
                    _ => continue,
                }
            }
        }
        None
    }
    #[allow(dead_code)]
    pub fn print(&self, delay:u64) {
        thread::sleep(Duration::from_millis(delay));
        println!("{}", self);
    }
}

impl Deref for Room {
    type Target = Vec<Vec<RoomSpace>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Room {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Room {
    fn fmt(&self, fmt:&mut Formatter) -> Result<(), std::fmt::Error> {
        if self.is_empty() {
            return fmt.write_str("");
        }
        let mut resultstr = String::new();
        resultstr.push_str(&"-".repeat(self[0].len()));
        resultstr.push('\n');

        let num_cols = self.len();
        let num_rows = self[0].len();

        for col in 0..num_rows {
            let row: String = (0..num_cols)
                .map(|row| self[row][col].to_string())
                .collect();
                resultstr.push_str(&row);
                resultstr.push('\n');
        }
        resultstr.push_str(&"-".repeat(self[0].len()));
        resultstr.push('\n');
        fmt.write_str(&resultstr)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Trail(Vec<(Direction,(usize,usize))>);

impl Trail {
    pub fn new() -> Trail {
        Trail(Vec::new())
    }
}

impl Deref for Trail {
    type Target = Vec<(Direction,(usize,usize))>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Trail {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
