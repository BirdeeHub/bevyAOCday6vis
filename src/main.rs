use std::io::Result;
use std::env;
use bevy::prelude::*;
mod part1and2;
mod types;
mod asset;
mod buttons;

use crate::types::*;

use crate::asset::{EmbeddedPlug, get_guard_sprite};

fn main() -> Result<()> {
    // Get the Room and trails from your logic
    let args: Vec<String> = std::env::args().collect();
    let filepath = match args.get(1) {
        Some(filepath_arg) => filepath_arg.to_string(),
        _ => env::var("AOC_INPUT").expect("AOC_INPUT not set")
    };
    let filecontents = crate::part1and2::read_file(&filepath)?;

    let (board, guard1, visited) = crate::part1and2::part1(filecontents);

    let mut guards = AllGuards::new();
    guards.push(guard1.clone());

    for (i,(x,y)) in board.to_check.iter().enumerate() {
        println!("{} / {}",i,board.to_check.len());
        guards.push(crate::part1and2::part2(&mut board.clone(), guard1.is_loop, *x,*y, i));
    }
    let obstacles = crate::part1and2::deduplicate_vec(guards.iter().filter(|v|v.is_loop).collect()).len();

    println!("Part 1: total visited: {}", visited);

    println!("Part 2: possible obstacle locations for loop: {:?}",obstacles);

    // Initialize Bevy App
    let mut app = App::new();
    app.add_plugins(DefaultPlugins) // Default plugins for window and rendering
        .add_plugins(EmbeddedPlug)
        .init_state::<AppState>()
        .insert_resource(board) // Insert Room as a resource to access in systems
        .insert_resource(guards)
        .insert_resource(CameraTarget(0))
        .insert_resource(MoveTimer(Timer::from_seconds(0.05, TimerMode::Repeating))) // Add the timer resource
        .add_systems(Startup,(setup_camera,room_setup))
        .add_systems(Startup,crate::buttons::setup_menu)
        .add_systems(Update,crate::buttons::menu)
        .add_systems(OnEnter(AppState::Part1),guard_spawn)
        .add_systems(Update,(render_trail,move_guard,update_camera).chain().run_if(in_state(AppState::Part1))) // Set up camera
        .add_systems(OnExit(AppState::Part1),cleanup_p1)
        .run(); // Spawn Room entities

    Ok(())
}

// Set up a 2D camera
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn update_camera(
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<Guard>)>,
    target: Res<CameraTarget>,
    guards: Query<(&Transform, &Guard), Without<Camera2d>>,
    time: Res<Time>,
) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    let mut guard = None;

    for (e, g) in &guards {
        if g.display_index == target.0 {
            guard = Some(e);
        }
    };

    if let Some(g) = guard {
        let Vec3{ x, y, .. } = g.translation;
        let direction = Vec3::new(x, y, camera.translation.z);

        // Applies a smooth effect to camera movement using stable interpolation
        // between the camera position and the player position on the x and y axes.
        camera
            .translation
            .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());
    }
}

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
    mut room: Res<Room>, // Access to the room to modify it
    mut guards: ResMut<AllGuards>,
    mut state: Res<State<AppState>>,
    asset_server: Res<AssetServer>,
) {
    for guard in &guards.0 {
        if let Some((dir,(x,y))) = guard.get_loc() {
            commands.spawn((
                Sprite::from_image(asset_server.load(get_guard_sprite(&dir,1))),
                Transform::from_translation(Vec3::new(
                    x as f32 * SCALED_CELL_SIZE + OFFSET_X,
                    y as f32 * -SCALED_CELL_SIZE + OFFSET_Y, // Use -scaled_cell_size for inverted Y
                    2.0,
                )),
                Visibility::default(),
                guard.clone(),
                GridEntity, // Tag the entity
            ));
        }
        if *state.get() != AppState::Part2 { break; }
    }
}

fn move_guard(
    mut commands: Commands,
    mut room: ResMut<Room>, // Access to the room to modify it
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut guardquery: Query<(Entity, &mut Transform, &mut Sprite), With<Guard>>, // Query all entities with the GridEntity component
) {
    for (entity, mut tform, mut sprite) in &mut guardquery {
        if let Some((dir,(x,y))) = room.get_guard_loc() {
            let mut direction = Vec3::ZERO;
            direction.x = (x as f32 * SCALED_CELL_SIZE + OFFSET_X) - tform.translation.x;
            direction.y = (y as f32 * -SCALED_CELL_SIZE + OFFSET_Y) - tform.translation.y; // Use -scaled_cell_size for inverted Y
            if direction != Vec3::ZERO {
                tform.translation += direction * SCALED_CELL_SIZE * time.delta_secs();
            }
            *sprite = Sprite::from_image(asset_server.load(get_guard_sprite(&dir,1)));
        }
    }
}

fn render_trail(
    mut commands: Commands,
    mut room: ResMut<Room>, // Access to the room to modify it
    time: Res<Time>,
    mut timer: ResMut<MoveTimer>,
    asset_server: Res<AssetServer>,
    querytrail: Query<(Entity, &TrailEntity)>, // Query all entities with the TrailEntity component AND their TrailEntity component
) {
    if timer.0.tick(time.delta()).just_finished() {
        room.advance();
        let mut final_idx = 0;
        let mut has_zero = false;
        for (entity, trailidx) in querytrail.iter() {
            if let Some(_) = room.trail.get(trailidx.index) {
                if trailidx.index == 0 { has_zero = true; }
                final_idx = trailidx.index;
            } else {
                commands.entity(entity).despawn();
            }
        }
        if !has_zero {
            if let Some((dir,(x,y))) = room.trail.get(0) {
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
                    TrailEntity::new(0),
                    GridEntity, // Tag the entity
                ));
            }
        }
        for i in (final_idx+1)..room.get_current_trail().len() {
            if let Some((dir,(x,y))) = room.trail.get(i) {
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

fn cleanup_p1(mut commands: Commands, mut room: ResMut<Room>, guard: Query<Entity, With<Guard>>, trail: Query<Entity, With<TrailEntity>>) {
    room.reset();
    for entity in guard.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in trail.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
