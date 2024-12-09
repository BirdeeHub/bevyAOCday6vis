use std::io::Result;
use std::env;
use bevy::prelude::*;
use bevy::asset::*;
use bevy::time::*;
mod part1and2;
mod types;
mod asset;

use crate::types::*;

use crate::asset::{EmbeddedPlug, get_guard_sprite};

const CELL_SIZE: f32 = 20.0; // Define cell size in pixels
const SCALE_FACTOR: f32 = 1.0; // Scaling factor for cell size
const OFFSET_X: f32 = -500.0; // Offset to move the grid horizontally
const OFFSET_Y: f32 = 500.0; // Offset to move the grid vertically

// Adjust the size and position
const SCALED_CELL_SIZE: f32 = CELL_SIZE * SCALE_FACTOR;

/// How quickly should the camera snap to the desired location.
const CAMERA_DECAY_RATE: f32 = 2.;

// Components to represent Room elements visually.
#[derive(Component)]
struct Space {
    x: usize,
    y: usize,
}

#[derive(Resource)]
struct MoveTimer(Timer);

#[derive(Component)]
struct Guard {
    direction: Direction,
    position: (usize, usize),
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
        .add_systems(Startup,(setup_camera,room_setup,guard_spawn))
        .add_systems(Update,(move_guard,update_camera).chain()) // Set up camera
        .run(); // Spawn Room entities

    Ok(())
}

// Set up a 2D camera
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn update_camera(
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<Guard>)>,
    player: Query<&Transform, (With<Guard>, Without<Camera2d>)>,
    time: Res<Time>,
) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    let Ok(player) = player.get_single() else {
        return;
    };

    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    // Applies a smooth effect to camera movement using stable interpolation
    // between the camera position and the player position on the x and y axes.
    camera
        .translation
        .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());
}

#[derive(Component)]
struct GridEntity;

fn room_setup(
    mut commands: Commands,
    mut room: ResMut<Room>, // Access to the room to modify it
    asset_server: Res<AssetServer>,
    //query: Query<Entity, With<GridEntity>>, // Query all entities with the GridEntity component
) {
    // Iterate over the Room grid and spawn new entities
    for (x, row) in room.iter().enumerate() {
        for (y, cell) in row.iter().enumerate() {
            let sprite = match cell {
                RoomSpace::Empty => Sprite {
                    color: Color::srgb(0.5, 0.5, 0.5), // Gray
                    custom_size: Some(Vec2::new(SCALED_CELL_SIZE, SCALED_CELL_SIZE)),
                    ..default()
                },
                RoomSpace::Obstacle => Sprite {
                    color: Color::srgb(0.0, 0.0, 0.0), // Black
                    custom_size: Some(Vec2::new(SCALED_CELL_SIZE, SCALED_CELL_SIZE)),
                    ..default()
                },
                RoomSpace::Visited => Sprite {
                    color: Color::srgb(0.0, 1.0, 0.0), // Green
                    custom_size: Some(Vec2::new(SCALED_CELL_SIZE, SCALED_CELL_SIZE)),
                    ..default()
                },
                RoomSpace::Guard(_) => Sprite {
                    color: Color::srgb(0.5, 0.5, 0.5), // Gray
                    custom_size: Some(Vec2::new(SCALED_CELL_SIZE, SCALED_CELL_SIZE)),
                    ..default()
                },
            };
            commands.spawn((
                sprite,
                Transform::from_translation(Vec3::new(
                    x as f32 * SCALED_CELL_SIZE + OFFSET_X,
                    y as f32 * -SCALED_CELL_SIZE + OFFSET_Y, // Use -scaled_cell_size for inverted Y
                    0.0,
                )),
                Visibility::default(),
                Space{x,y},
                GridEntity, // Tag the entity
            ));
        }
    }
}

fn guard_spawn(
    mut commands: Commands,
    mut room: ResMut<Room>, // Access to the room to modify it
    asset_server: Res<AssetServer>,
) {
    if let Some((dir,(x,y))) = room.get_guard_loc() {
        commands.spawn((
            get_guard_sprite(&dir,1,asset_server),
            Transform::from_translation(Vec3::new(
                x as f32 * SCALED_CELL_SIZE + OFFSET_X,
                y as f32 * -SCALED_CELL_SIZE + OFFSET_Y, // Use -scaled_cell_size for inverted Y
                2.0,
            )),
            Visibility::default(),
            Guard { direction: Direction::Up, position: (x,y) },
            GridEntity, // Tag the entity
        ));
    }
}

#[derive(Component)]
struct TrailEntity {
    index: usize,
}

impl TrailEntity {
    fn new(index: usize) -> TrailEntity {
        TrailEntity{index}
    }
}

fn move_guard(
    mut commands: Commands,
    mut room: ResMut<Room>, // Access to the room to modify it
    time: Res<Time>,
    mut timer: ResMut<MoveTimer>,
    asset_server: Res<AssetServer>,
    query: Query<Entity, With<Guard>>, // Query all entities with the GridEntity component
    querytrail: Query<(Entity, &TrailEntity)>, // Query all entities with the TrailEntity component AND their TrailEntity component
) {
    if timer.0.tick(time.delta()).just_finished() {
        room.advance();
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
        if let Some((dir,(x,y))) = room.get_guard_loc() {
            commands.spawn((
                get_guard_sprite(&dir,1,asset_server),
                Transform::from_translation(Vec3::new(
                    x as f32 * SCALED_CELL_SIZE + OFFSET_X,
                    y as f32 * -SCALED_CELL_SIZE + OFFSET_Y, // Use -scaled_cell_size for inverted Y
                    2.0,
                )),
                Visibility::default(),
                Guard { direction: dir, position: (x,y) },
                GridEntity, // Tag the entity
            ));
        }
        let mut final_idx = 0;
        for (entity, trailidx) in querytrail.iter() {
            if let Some(_) = room.trail.get(trailidx.index) {
                final_idx = trailidx.index;
            } else {
                commands.entity(entity).despawn();
            }
        }
        for (i, (dir,(x,y))) in room.get_current_trail().iter().enumerate() {
            if i > final_idx {
                commands.spawn((
                    Sprite {
                        color: Color::srgb(0.0, 1.0, 0.0), // Green
                        custom_size: Some(Vec2::new(SCALED_CELL_SIZE/2., SCALED_CELL_SIZE/2.)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(
                        *x as f32 * SCALED_CELL_SIZE + OFFSET_X,
                        *y as f32 * -SCALED_CELL_SIZE + OFFSET_Y, // Use -scaled_cell_size for inverted Y
                        1.0,
                    )),
                    Visibility::default(),
                    TrailEntity::new(i),
                    GridEntity, // Tag the entity
                ));
            }
        }
    }
}
