use bevy::prelude::*;
use std::ops::{Deref, DerefMut};

pub const CELL_SIZE: f32 = 20.0; // Define cell size in pixels
pub const SCALE_FACTOR: f32 = 1.0; // Scaling factor for cell size
pub const OFFSET_X: f32 = -500.0; // Offset to move the grid horizontally
pub const OFFSET_Y: f32 = 500.0; // Offset to move the grid vertically

pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
// Adjust the size and position
pub const SCALED_CELL_SIZE: f32 = CELL_SIZE * SCALE_FACTOR;

/// How quickly should the camera snap to the desired location.
pub const CAMERA_DECAY_RATE: f32 = 2.;

#[derive(Resource,Clone, Copy, PartialEq)]
pub struct StateInfo{
    pub room_idx: Option<usize>,
    pub camera_target: usize,
    pub trace_only: bool,
    pub seq: bool,
}
impl StateInfo {
    pub fn new() -> StateInfo {
        StateInfo{camera_target:0,trace_only:true,seq:true,room_idx:None,}
    }
    pub fn p1_loaded(guards:&AllGuards) -> bool {
        ! guards.is_empty()
    }
    pub fn p2_loaded(room:&Room,guards:&AllGuards) -> bool {
        room.to_check.len() <= guards.0.len()
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
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

#[derive(Debug, PartialEq, Resource, Clone)]
pub struct Room {
    grid: Vec<Vec<RoomSpace>>,
    pub to_check: Vec<(usize,usize)>,
}

impl Room {
    pub fn new() -> Room {
        Room{to_check: Vec::new(), grid: Vec::new()}
    }
    pub fn from_string(input: String) -> Room {
        let mut rawout:Vec<Vec<RoomSpace>> = Vec::new();

        for line in input.lines() {
            let mut row:Vec<RoomSpace> = Vec::new();
            for c in line.chars() {
                row.push(match c {
                    '^' => RoomSpace::Guard(Direction::Up),
                    '<' => RoomSpace::Guard(Direction::Left),
                    '>' => RoomSpace::Guard(Direction::Right),
                    'v' => RoomSpace::Guard(Direction::Down),
                    '#' => RoomSpace::Obstacle,
                    _ => RoomSpace::Empty,
                });
            }
            rawout.push(row);
        }
        // fix x and y...
        let mut newroom = Room::new();
        for i in 0..rawout[0].len() {
            let mut newrow = Vec::new();
            rawout.iter().for_each(|row|newrow.push(row[i].clone()));
            newroom.push(newrow);
        };
        newroom
    }
    pub fn reset(&mut self) {
        // Iterate through the grid and reset RoomSpace values
        for row in &mut self.grid {
            for cell in row {
                *cell = match cell {
                    RoomSpace::Visited => RoomSpace::Empty, // Clear visited spaces
                    RoomSpace::Guard(_) => RoomSpace::Empty, // Clear guards
                    _ => cell.clone(), // Leave other spaces unchanged
                };
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

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
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
pub struct AllGuards(pub Vec<Guard>);

impl AllGuards {
    pub fn new() -> AllGuards {
        AllGuards(Vec::new())
    }
}

impl Deref for AllGuards {
    type Target = Vec<Guard>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AllGuards {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    InputScreen,
    Part1,
    Part2
}

// Components to represent Room elements visually.
#[derive(Component)]
pub struct Space {
    pub x: usize,
    pub y: usize,
}

#[derive(Resource)]
pub struct MoveTimer(pub Timer);

#[derive(Component, Debug, PartialEq, Hash, Eq, Clone)]
pub struct Guard {
    pub trail: Trail,
    pub obstacle: Option<(usize, usize)>,
    pub is_loop: bool,
    pub trail_idx: usize,
    pub display_index: usize,
}

impl Guard {
    pub fn new(trail: Trail, obstacle: Option<(usize,usize)>, is_loop: bool, display_index: usize) -> Guard {
        Guard { trail, obstacle, is_loop, trail_idx: 0, display_index, }
    }
    pub fn retreat(&mut self) -> Option<(Direction,(usize,usize))> {
        if self.trail.get(self.trail_idx).is_some() && self.trail_idx > 0 {
            self.trail_idx -= 1;
            let pos = self.trail[self.trail_idx].clone();
            Some(pos)
        } else { None }
    }
    pub fn advance(&mut self) -> Option<(Direction,(usize,usize))> {
        if self.trail.get(self.trail_idx).is_some() {
            let pos = self.trail[self.trail_idx].clone();
            self.trail_idx += 1;
            if self.trail.get(self.trail_idx).is_some() {
                let (dir,(x,y)) = self.trail[self.trail_idx].clone();
                Some((dir,(x,y)))
            } else {
                Some(pos)
            }
        } else {
            None
        }
    }
    pub fn get_current_trail(&mut self) -> Trail {
        let mut ret = Trail::new();
        for i in 0..self.trail_idx {
            ret.push(self.trail[i].clone());
        }
        ret
    }
    pub fn get_loc(&self) -> Option<(Direction,(usize,usize))> {
        if let Some((dir,(x,y))) = self.trail.get(self.trail_idx) {
            Some((dir.clone(),(*x,*y)))
        } else if self.trail_idx >= self.trail.len() {
            if let Some(ret) = self.trail.last() {
                Some(ret.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn reset(&mut self) {
        self.trail_idx = 0;
    }
}

#[derive(Component)]
pub struct GridEntity;

#[derive(Component)]
pub struct TrailEntity {
    index: usize,
}

impl TrailEntity {
    pub fn new(index: usize) -> TrailEntity {
        TrailEntity{index}
    }
}

#[derive(Debug, Clone, Resource)]
pub struct AllRooms(pub Vec<(Room,AllGuards)>);

impl AllRooms {
    pub fn new() -> AllRooms {
        AllRooms(Vec::new())
    }
    pub fn get_room_mut(&mut self,room_idx:Option<usize>) -> Option<&mut (Room,AllGuards)> {
        if let Some(room_idx) = room_idx {
            self.0.get_mut(room_idx)
        } else { return None; }
    }
    pub fn get_room(&self,room_idx:Option<usize>) -> Option<&(Room,AllGuards)> {
        if let Some(room_idx) = room_idx {
            self.0.get(room_idx)
        } else { return None; }
    }
}

impl Deref for AllRooms {
    type Target = Vec<(Room,AllGuards)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AllRooms {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
