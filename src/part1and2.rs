use std::io::{self};

use crate::types::*;

pub fn run(filepath: &str) -> io::Result<(Room, Trail, Vec<(usize, usize, Trail)>)> {
    let room = Room::from_file(filepath)?;

    let mut board = room.clone();
    let boardx = board.len();
    let boardy = board[0].len();
    let mut trail = Trail::new();
    if check_for_loop(&mut board, &mut trail, boardx, boardy).is_some() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Board has loop in initial state!"));
    };
    
    let visited = board.iter().flat_map(|row| row.iter()).filter(|&cell| cell == &RoomSpace::Visited).count();

    let mut chktrails = Vec::new();

    let mut obstacles = Vec::new();
    let trlclone = trail.clone();
    let tocheck = deduplicate_vec(trlclone.iter().map(|(_,pos)|pos).collect());
    for (i,(x,y)) in tocheck.iter().enumerate().skip(1) {
        println!("{} / {}",i,trail.len()-1);
        let mut chktrl = Trail::new();
        if let Some(obs) = check_for_loop(&mut room.clone(), &mut chktrl, *x,*y) {
            obstacles.push(obs);
        }
        chktrails.push((*x, *y, chktrl))
    }
    obstacles = deduplicate_vec(obstacles);

    println!("Part 1: total visited: {}", visited);

    println!("Part 2: possible obstacle locations for loop: {:?}",obstacles.len());

    Ok((room,trail,chktrails))
}

fn check_for_loop(room: &mut Room, trail: &mut Trail, obsx: usize, obsy: usize) -> Option<(usize,usize)> {
    if obsx < room.len() && obsy < room[0].len() {
        if room[obsx][obsy] == RoomSpace::Obstacle {
            return None;
        }
        room.add_obstacle(obsx,obsy);
    }
    let mut continue_moving = true;
    let mut checkpoints = Vec::new();
    while continue_moving {
        continue_moving = move_guard(room, trail);
        if continue_moving && checkpoints.contains(trail.last().unwrap()) {
            return Some((obsx,obsy))
        }
        if continue_moving {
            checkpoints.push(trail.last().unwrap().clone());
        }
    }
    None
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

fn deduplicate_vec<T: Eq + std::hash::Hash>(vec: Vec<T>) -> Vec<T> {
    let mut result = Vec::new();
    for item in vec {
        if !result.contains(&item) {
            result.push(item);
        }
    }
    result
}
