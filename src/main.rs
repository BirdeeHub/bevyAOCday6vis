use std::io::Result;
use std::env;
use bevy::prelude::*;
use bevy::asset::*;
use bevy::time::*;
mod part1and2;
mod types;

use crate::types::*;

// Components to represent Room elements visually.
#[derive(Component)]
struct Space;

#[derive(Resource)]
struct MoveTimer(Timer);

struct EmbeddedPlug;
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
    }
}

fn main() -> Result<()> {
    // Get the Room and trails from your logic
    let args: Vec<String> = std::env::args().collect();
    let filepath = match args.get(1) {
        Some(filepath_arg) => filepath_arg.to_string(),
        _ => env::var("AOC_INPUT").expect("AOC_INPUT not set")
    };
    let (room, chkrooms) = part1and2::run(&filepath)?;
    let testroom = room.clone();
    let checkrooms = chkrooms.clone();

    // Initialize Bevy App
    let mut app = App::new();
    app.add_plugins(DefaultPlugins) // Default plugins for window and rendering
        .add_plugins(EmbeddedPlug)
        .insert_resource(testroom) // Insert Room as a resource to access in systems
        .insert_resource(checkrooms) // Insert Room as a resource to access in systems
        .insert_resource(MoveTimer(Timer::from_seconds(0.25, TimerMode::Repeating))) // Add the timer resource
        .add_systems(Startup,setup_camera) // Set up camera
        .add_systems(Update,move_system)
        .run(); // Spawn Room entities

    Ok(())
}

// Set up a 2D camera
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// Update system that runs once every 250ms
fn move_system(mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<MoveTimer>,
    mut room: ResMut<Room>, // Access to the room to modify it
    asset_server: Res<AssetServer>
) {
    // Tick the timer
    if timer.0.tick(time.delta()).just_finished() {
        let cell_size = 20.0; // Define cell size in pixels
        let scale_factor = 1.0; // Scaling factor for cell size
        let offset_x = -500.0; // Offset to move the grid horizontally
        let offset_y = 500.0; // Offset to move the grid vertically

        // Adjust the size and position
        let scaled_cell_size = cell_size * scale_factor;

        room.advance();

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
                    RoomSpace::Guard(Direction::Up) => Sprite::from_image(asset_server.load("embedded://day6vis/sprites/Up1.png")),
                    RoomSpace::Guard(Direction::Left) => Sprite::from_image(asset_server.load("embedded://day6vis/sprites/Left1.png")),
                    RoomSpace::Guard(Direction::Right) => Sprite::from_image(asset_server.load("embedded://day6vis/sprites/Right1.png")),
                    RoomSpace::Guard(Direction::Down) => Sprite::from_image(asset_server.load("embedded://day6vis/sprites/Down1.png")),
                };
                commands.spawn((
                    sprite,
                    Transform::from_translation(Vec3::new(
                        x as f32 * scaled_cell_size + offset_x,
                        y as f32 * -scaled_cell_size + offset_y, // Use -scaled_cell_size for inverted Y
                        0.0,
                    )),
                    Visibility::default(),
                    Space,
                ));
            }
        }

    }
}
