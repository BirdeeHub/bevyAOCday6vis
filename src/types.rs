use std::ops::{Deref, DerefMut};
use std::fmt::{Display, Formatter};
use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    asset::*,
    tasks::Task,
};

pub const CELL_SIZE: f32 = 20.0; // Define cell size in pixels
pub const SCALE_FACTOR: f32 = 1.0; // Scaling factor for cell size

pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
// Adjust the size and position
pub const SCALED_CELL_SIZE: f32 = CELL_SIZE * SCALE_FACTOR;

/// How quickly should the camera snap to the desired location.
pub const CAMERA_DECAY_RATE: f32 = 2.;

#[derive(Component)]
pub struct ToDelete(pub usize);
#[derive(Component)]
pub struct Obstacle(pub usize);

#[derive(Component)]
pub struct ComputeTrails(pub Task<CommandQueue>);

#[derive(Component)]
pub struct ProgressBarFill;
#[derive(Component)]
pub struct StateButton;
#[derive(Component)]
pub struct StateButtonText;

#[derive(Component)]
pub struct ErrorBox(pub String);

#[derive(Component)]
pub struct InputText(pub String);
#[derive(Resource)]
pub struct PendingText(pub String);
#[derive(Resource)]
pub struct CurrentError(pub String);

#[derive(Component)]
pub struct MenuParent;

#[derive(Resource, Clone, Copy, PartialEq)]
pub struct StateInfo{
    pub room_idx: Option<usize>,
    pub camera_target: usize,
}
impl StateInfo {
    pub fn new() -> StateInfo {
        StateInfo{camera_target:0,room_idx:None,}
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

#[derive(Debug, PartialEq, Clone)]
pub struct Room {
    grid: Vec<Vec<RoomSpace>>,
    initial_guard_pos: Option<(Direction,(usize,usize))>,
    pub to_check: Vec<(usize,usize)>,
    pub visited: usize,
    pub index: usize,
}

pub enum RoomError {
    NoGuards,
    ManyGuards,
    Uneven,
}
impl Display for RoomError {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        fmt.write_str(match self {
            RoomError::NoGuards => "Room has no guards.",
            RoomError::ManyGuards => "Room has more than one guard.",
            RoomError::Uneven => "Room is not rectangular.",
        })
    }
}

impl Room {
    pub fn new() -> Room {
        Room{to_check: Vec::new(), grid: Vec::new(), visited:0, initial_guard_pos:None, index:0}
    }
    pub fn from_string(input: String) -> Result<Room,RoomError> {
        let mut rawout:Vec<Vec<RoomSpace>> = Vec::new();

        let mut guard_found = false;

        for line in input.lines() {
            let mut row:Vec<RoomSpace> = Vec::new();
            for c in line.chars() {
                let value = match c {
                    '^' => RoomSpace::Guard(Direction::Up),
                    '<' => RoomSpace::Guard(Direction::Left),
                    '>' => RoomSpace::Guard(Direction::Right),
                    'v' => RoomSpace::Guard(Direction::Down),
                    '#' => RoomSpace::Obstacle,
                    _ => RoomSpace::Empty,
                };
                if let RoomSpace::Guard(_) = value {
                    if guard_found {
                        return Err(RoomError::ManyGuards);
                    } else {
                        guard_found = true;
                    }
                };
                row.push(value);
            }
            rawout.push(row);
        }
        if !guard_found {
            return Err(RoomError::NoGuards);
        }
        let mut mutlock = true;
        let mut rowlen = 0;
        for row in &rawout {
            if mutlock {
                rowlen = row.len();
                mutlock = false;
            } else if row.len() != rowlen {
                return Err(RoomError::Uneven);
            }
        }
        // fix x and y...
        let mut newroom = Room::new();
        for i in 0..rawout[0].len() {
            let mut newrow = Vec::new();
            rawout.iter().for_each(|row|newrow.push(row[i].clone()));
            newroom.push(newrow);
        };
        newroom.initial_guard_pos = newroom.find_guard();
        Ok(newroom)
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
        };
        if let Some((dir,(x,y))) = self.initial_guard_pos.clone() {
            self.add_guard(x,y,&dir);
        };
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

impl Display for Room {
    fn fmt(&self, fmt:&mut Formatter) -> std::fmt::Result {
        let mut resultstr = String::new();
        let num_cols = self.len();
        let num_rows = self[0].len();
        for col in 0..num_rows {
            let row: String = (0..num_cols)
                .map(|row| self[row][col].to_string())
                .collect();
                resultstr.push_str(&row);
                resultstr.push('\n');
        }
        fmt.write_str(&resultstr)
    }
}

impl Display for RoomSpace {
    fn fmt(&self, fmt:&mut Formatter) -> std::fmt::Result {
        fmt.write_str(match self {
            RoomSpace::Guard(dir) => match dir {
                Direction::Up => "^",
                Direction::Down => "v",
                Direction::Left => "<",
                Direction::Right => ">",
            },
            RoomSpace::Obstacle => "#",
            RoomSpace::Visited => ".",
            RoomSpace::Empty => ".",
        })
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

#[derive(Debug, PartialEq, Clone)]
pub struct AllGuards(pub Vec<Guard>);

impl AllGuards {
    pub fn new() -> AllGuards {
        AllGuards(Vec::new())
    }
    pub fn sort_by_idx(&mut self) {
        self.0.sort_by(|a, b| a.display_index.cmp(&b.display_index));
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
    pub fn get_sprite(&self) -> String {
        let index = self.display_index % 3;
        let direction = if let Some((d,_)) = self.get_loc() { d } else { Direction::Down };
        (match direction {
            Direction::Up => match index {
                1 => "embedded://day6vis/sprites/Up2.png",
                2 => "embedded://day6vis/sprites/Up3.png",
                _ => "embedded://day6vis/sprites/Up1.png",
            },
            Direction::Down => match index {
                1 => "embedded://day6vis/sprites/Down2.png",
                2 => "embedded://day6vis/sprites/Down3.png",
                _ => "embedded://day6vis/sprites/Down1.png",
            },
            Direction::Left => match index {
                1 => "embedded://day6vis/sprites/Left2.png",
                2 => "embedded://day6vis/sprites/Left3.png",
                _ => "embedded://day6vis/sprites/Left1.png",
            },
            Direction::Right => match index {
                1 => "embedded://day6vis/sprites/Right2.png",
                2 => "embedded://day6vis/sprites/Right3.png",
                _ => "embedded://day6vis/sprites/Right1.png",
            },
        }).to_string()
    }
}

#[derive(Component)]
pub struct TrailEntity {
    pub index: usize,
    pub guard_index: usize,
}

impl TrailEntity {
    pub fn new(index:usize, guard_index: usize) -> TrailEntity {
        TrailEntity{index,guard_index}
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
            for mut entry in &mut self.0 {
                if entry.0.index == room_idx { return Some(entry); }
            };
            return None;
        } else { return None; }
    }
    pub fn get_room(&self,room_idx:Option<usize>) -> Option<&(Room,AllGuards)> {
        if let Some(room_idx) = room_idx {
            for entry in &self.0 {
                if entry.0.index == room_idx { return Some(entry); }
            };
            return None;
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

pub struct EmbeddedPlug;
impl Plugin for EmbeddedPlug {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "sprites/Up1.png");
        embedded_asset!(app, "sprites/Up2.png");
        embedded_asset!(app, "sprites/Up3.png");
        embedded_asset!(app, "sprites/Right1.png");
        embedded_asset!(app, "sprites/Right2.png");
        embedded_asset!(app, "sprites/Right3.png");
        embedded_asset!(app, "sprites/Down1.png");
        embedded_asset!(app, "sprites/Down2.png");
        embedded_asset!(app, "sprites/Down3.png");
        embedded_asset!(app, "sprites/Left1.png");
        embedded_asset!(app, "sprites/Left2.png");
        embedded_asset!(app, "sprites/Left3.png");
        embedded_asset!(app, "sprites/Obstacle0.png");
        embedded_asset!(app, "sprites/Obstacle1.png");
        embedded_asset!(app, "sprites/Obstacle2.png");
        embedded_asset!(app, "sprites/Obstacle3.png");
        embedded_asset!(app, "sprites/Obstacle4.png");
        embedded_asset!(app, "sprites/Obstacle5.png");
        embedded_asset!(app, "sprites/Obstacle6.png");
        embedded_asset!(app, "sprites/Obstacle7.png");
        embedded_asset!(app, "sprites/Obstacle8.png");
        embedded_asset!(app, "sprites/Obstacle9.png");
        embedded_asset!(app, "sprites/Obstacle10.png");
        embedded_asset!(app, "sprites/Obstacle11.png");
        embedded_asset!(app, "sprites/Obstacle12.png");
    }
}

pub fn random_obstacle() -> String {
    "embedded://day6vis/sprites/Obstacle".to_string() + &(rand::random::<u8>() % 13).to_string() + ".png"
}

pub fn color_from_idx(idx: usize) -> Color {
    Color::hsv((idx as f32 * 10. + 100.) % 360., 1., 1.)
}
