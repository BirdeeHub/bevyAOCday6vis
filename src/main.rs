use std::io::Result;
use bevy::prelude::*;
mod part1and2;
mod types;

use crate::types::*;

// Components to represent Room elements visually.
#[derive(Component)]
struct Cell;

fn main() -> Result<()> {
    // Get the Room and trails from your logic
    let (room, trail, chktrails) = part1and2::run()?;

    // Initialize Bevy App
    App::new()
        .add_plugins(DefaultPlugins) // Default plugins for window and rendering
        .insert_resource(room) // Insert Room as a resource to access in systems
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
fn spawn_room(mut commands: Commands, room: Res<Room>) {
    let cell_size = 20.0; // Define cell size in pixels

    // Iterate over the Room grid
    for (x, row) in room.iter().enumerate() {
        for (y, cell) in row.iter().enumerate() {
            let color = match cell {
                RoomSpace::Empty => Color::srgb(0.5, 0.5, 0.5), // Gray
                RoomSpace::Obstacle => Color::srgb(0.0, 0.0, 0.0), // Black
                RoomSpace::Visited => Color::srgb(0.0, 1.0, 0.0), // Green
                RoomSpace::Guard(_) => Color::srgb(0.0, 0.0, 1.0), // Blue
            };

            // Spawn a rectangle for each cell
            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(cell_size, cell_size)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(
                    x as f32 * cell_size,
                    y as f32 * - cell_size,
                    0.0,
                )),
                Visibility::default(),
                Cell,
            ));
        }
    }
}
