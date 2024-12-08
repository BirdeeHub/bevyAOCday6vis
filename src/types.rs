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
pub struct Room {
    pub name: String,
    trail_idx: usize,
    grid: Vec<Vec<RoomSpace>>,
    pub trail: Trail,
}

impl Room {
    pub fn new(name: String, trail: Option<Trail>) -> Room {
        if let Some(trail) = trail {
            Room{name, trail_idx: 0, trail: trail.clone(), grid: Vec::new()}
        } else {
            Room{name, trail_idx: 0, trail: Trail::new(), grid: Vec::new()}
        }
    }
    pub fn from_file<P: AsRef<Path>>(filepath: P, name: String) -> io::Result<Room> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);

        let mut rawout:Vec<Vec<RoomSpace>> = Vec::new();

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
            rawout.push(row);
        }
        // fix x and y...
        let mut newroom = Room::new(name, None);
        for i in 0..rawout[0].len() {
            let mut newrow = Vec::new();
            rawout.iter().for_each(|row|newrow.push(row[i].clone()));
            newroom.push(newrow);
        };
        Ok(newroom)
    }
    pub fn retreat(&mut self) {
        if self.trail.get(self.trail_idx).is_some() && self.trail_idx > 0 {
            let pos = self.trail[self.trail_idx].clone();
            self[pos.1.0][pos.1.1] = RoomSpace::Empty;
            self.trail_idx -= 1;
            self.add_guard(pos.1.0,pos.1.1,&pos.0)
        }
    }
    pub fn advance(&mut self) {
        if self.trail.get(self.trail_idx).is_some() {
            let pos = self.trail[self.trail_idx].clone();
            self.trail_idx += 1;
            if self.trail.get(self.trail_idx).is_some() {
                self.visit_space(pos.1.0,pos.1.1);
                let (dir,(x,y)) = self.trail[self.trail_idx].clone();
                self.add_guard(x,y,&dir)
            }
        }
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
        &self.grid
    }
}

impl DerefMut for Room {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.grid
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

#[derive(Debug, PartialEq, Clone, Resource)]
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

#[derive(Debug, PartialEq, Clone, Resource)]
pub struct CheckRooms(Vec<Room>);

impl CheckRooms {
    pub fn new() -> CheckRooms {
        CheckRooms(Vec::new())
    }
}

impl Deref for CheckRooms {
    type Target = Vec<Room>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CheckRooms {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
