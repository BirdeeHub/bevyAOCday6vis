use std::io::Result;
use std::env;
use bevy::prelude::*;
mod part1and2;
mod types;

use crate::types::*;

// Components to represent Room elements visually.
#[derive(Component)]
struct Cell;

fn main() -> Result<()> {
    // Get the Room and trails from your logic
    let (room, trail, chktrails) = part1and2::run(&env::var("AOC_INPUT").expect("AOC_INPUT not set"))?;

    let mut testroom = room.clone();
    testroom.apply_trail(&trail, true);

    // Initialize Bevy App
    App::new()
        .add_plugins(DefaultPlugins) // Default plugins for window and rendering
        .insert_resource(testroom) // Insert Room as a resource to access in systems
        .add_systems(Startup,setup_camera) // Set up camera
        .add_systems(Startup,spawn_room) // Spawn Room entities
        .run();

    Ok(())
}

// Set up a 2D camera
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// Spawn Room cells as entities
fn spawn_room(mut commands: Commands, room: Res<Room>,asset_server: Res<AssetServer>) {
    let cell_size = 20.0; // Define cell size in pixels
    let scale_factor = 1.0; // Scaling factor for cell size
    let offset_x = -500.0; // Offset to move the grid horizontally
    let offset_y = 500.0; // Offset to move the grid vertically

    // Adjust the size and position
    let scaled_cell_size = cell_size * scale_factor;

    // Iterate over the Room grid
    for (x, row) in room.iter().enumerate() {
        for (y, cell) in row.iter().enumerate() {
            let sprite = match cell {
                RoomSpace::Empty => Sprite {
                    color: Color::srgb(0.5, 0.5, 0.5), // Gray
                    custom_size: Some(Vec2::new(scaled_cell_size, scaled_cell_size)),
                    ..default()
                },
                RoomSpace::Obstacle => Sprite {
                    color: Color::srgb(0.0, 0.0, 0.0), // Black
                    custom_size: Some(Vec2::new(scaled_cell_size, scaled_cell_size)),
                    ..default()
                },
                RoomSpace::Visited => Sprite {
                    color: Color::srgb(0.0, 1.0, 0.0), // Green
                    custom_size: Some(Vec2::new(scaled_cell_size, scaled_cell_size)),
                    ..default()
                },
                //RoomSpace::Guard(_) => Sprite {
                //    color: Color::srgb(1.0, 0.0, 0.0), // Red
                //    custom_size: Some(Vec2::new(scaled_cell_size, scaled_cell_size)),
                //    ..default()
                //},
                RoomSpace::Guard(Direction::Up) => Sprite::from_image(asset_server.load("Up1.png")),
                RoomSpace::Guard(Direction::Left) => Sprite::from_image(asset_server.load("Left1.png")),
                RoomSpace::Guard(Direction::Right) => Sprite::from_image(asset_server.load("Right1.png")),
                RoomSpace::Guard(Direction::Down) => Sprite::from_image(asset_server.load("Down1.png")),
            };
            commands.spawn((
                sprite,
                Transform::from_translation(Vec3::new(
                    x as f32 * scaled_cell_size + offset_x,
                    y as f32 * -scaled_cell_size + offset_y, // Use -scaled_cell_size for inverted Y
                    0.0,
                )),
                Visibility::default(),
                Cell,
            ));
        }
    }
}
