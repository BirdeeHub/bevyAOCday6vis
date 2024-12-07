use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::time::Duration;
use std::thread;

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

#[derive(Debug, PartialEq, Clone)]
pub struct Room(Vec<Vec<RoomSpace>>);

impl Room {
    pub fn new() -> Room {
        Room(Vec::new())
    }
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
