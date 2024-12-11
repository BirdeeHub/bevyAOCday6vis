use std::io::{self};
use std::fs::File;
use std::io::Read;

use crate::types::*;

pub fn read_file(file_path: &str) -> io::Result<String> {
    let mut contents = String::new();
    File::open(file_path)?.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn part1(input: String) -> Result<(Room,Guard,usize), String> {
    let mut board = Room::from_string(input)?;
    let boardx = board.len();
    let boardy = board[0].len();
    let mut trail = Trail::new();
    let is_loop = check_for_loop(&mut board, &mut trail, false, boardx, boardy);
    let mut to_check = deduplicate_vec(trail.clone().iter().map(|(_,pos)|pos.clone()).collect());
    to_check.remove(0);
    let visited = board.iter().flat_map(|row| row.iter()).filter(|&cell| cell == &RoomSpace::Visited).count();
    board.reset();
    board.to_check = to_check;
    return Ok((board, Guard::new(trail.clone(),None,is_loop,0),visited));
}

pub fn part2(room: &Room, initial_path_is_loop: bool, obsx: usize, obsy: usize, index: usize) -> Guard {
    let mut newroom = room.clone();
    let mut trail = Trail::new();
    let is_loop = check_for_loop(&mut newroom, &mut trail, initial_path_is_loop, obsx,obsy);
    Guard::new(trail, Some((obsx, obsy)), is_loop, index) 
}

fn check_for_loop(room: &mut Room, trail: &mut Trail, initial_path_is_loop: bool, obsx: usize, obsy: usize) -> bool {
    if obsx < room.len() && obsy < room[0].len() {
        if room[obsx][obsy] == RoomSpace::Obstacle {
            return initial_path_is_loop;
        }
        room.add_obstacle(obsx,obsy);
    }
    let mut continue_moving = true;
    let mut checkpoints = Vec::new();
    while continue_moving {
        continue_moving = move_guard(room, trail);
        if continue_moving && checkpoints.contains(trail.last().unwrap()) {
            return true;
        }
        if continue_moving {
            checkpoints.push(trail.last().unwrap().clone());
        }
    }
    false
}

fn move_guard(room: &mut Room, trail: &mut Trail) -> bool {
    if let Some((direction,guard_pos)) = room.find_guard() {
        room.visit_space(guard_pos.0,guard_pos.1);
        trail.push((direction.clone(),guard_pos));
        if let Some((dir,newspace)) = get_newspace_with_obstacle(room, guard_pos, &direction) {
            if dir == direction {
                room.add_guard(newspace.0,newspace.1,&dir);
            } else {
                room.add_guard(guard_pos.0,guard_pos.1,&dir);
            }
            true
        } else {
            false
        }
    } else {
        false
    }
}

fn get_newspace_with_obstacle(room: &Room, pos: (usize,usize), direction: &Direction) -> Option<(Direction,(usize, usize))> {
    if let Some(newplace) = get_newspace(room, pos, direction) {
        if room[newplace.0][newplace.1] == RoomSpace::Obstacle {
            get_newspace_with_obstacle(room, pos, &turn_right(direction))
        } else {
            Some((direction.clone(),newplace))
        }
    } else {
        None
    }
}

fn get_newspace(room: &Room, pos: (usize,usize), direction: &Direction) -> Option<(usize, usize)> {
    match direction {
        Direction::Up => {
            if pos.1 > 0 { Some((pos.0, pos.1 - 1)) } else { None }
        },
        Direction::Down => {
            if pos.1 < room[pos.0].len() -1 { Some((pos.0, pos.1 + 1)) } else { None }
        },
        Direction::Right => {
            if pos.0 < room.len() - 1 { Some((pos.0 + 1, pos.1)) } else { None }
        },
        Direction::Left => {
            if pos.0 > 0 { Some((pos.0 - 1, pos.1)) } else { None }
        },
    }
}

fn turn_right(direction: &Direction) -> Direction {
    match direction {
        Direction::Up => Direction::Right,
        Direction::Right => Direction::Down,
        Direction::Down => Direction::Left,
        Direction::Left => Direction::Up,
    }
}

pub fn deduplicate_vec<T: Eq + std::hash::Hash>(vec: Vec<T>) -> Vec<T> {
    let mut result = Vec::new();
    for item in vec {
        if !result.contains(&item) {
            result.push(item);
        }
    }
    result
}
