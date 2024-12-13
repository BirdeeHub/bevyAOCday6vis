use bevy::prelude::*;
use std::env;
use crate::types::*;

pub fn setup_input(
    mut commands: Commands,
) {
    let args: Vec<String> = std::env::args().collect();
    let filepath = match args.get(1) {
        Some(filepath_arg) => filepath_arg.to_string(),
        _ => env::var("AOC_INPUT").expect("AOC_INPUT not set")
    };
    let Ok(filecontents) = crate::part1and2::read_file(&filepath) else { panic!("TESTFILEFAIL AOC_INPUT NOT SET") };
    commands.spawn(InputText(filecontents));
}

pub fn handle_input(
    mut commands: Commands,
    mut stateinfo: ResMut<StateInfo>,
) {
    stateinfo.room_idx = Some(0);
}

// TODO: make a text input screen, on new submissions call crate::part1and2::part1(inputtext),
// push result if Ok to AllRooms, otherwise print the error message somewhere on the screen and reject it.
// make a way to choose which input they should use (sets stateinfo.room_idx, which corresponds with their AllRooms.0[index])
// Should be able to click them in the input screen, and use the Display implementation of the Room type to
// print it back into the text input box to edit and save as a new room.
// should also have a space for instructions on how to format the input
