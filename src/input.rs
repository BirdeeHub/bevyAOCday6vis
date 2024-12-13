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
    //TODO: set up a UI for inputs here
    commands.spawn(InputText(filecontents));
}

//TODO: make a system that handles the UI inputs and makes them into InputText

//TODO: make a system that displays any existing ErrorBox entities and deletes them

pub fn handle_input(
    mut commands: Commands,
    mut stateinfo: ResMut<StateInfo>,
) {
    // TODO: make selector for this
    stateinfo.room_idx = Some(0);
}
