use std::io::Result;
use std::env;
use bevy::prelude::*;
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

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
// Adjust the size and position
const SCALED_CELL_SIZE: f32 = CELL_SIZE * SCALE_FACTOR;

/// How quickly should the camera snap to the desired location.
const CAMERA_DECAY_RATE: f32 = 2.;

fn main() -> Result<()> {
    // Get the Room and trails from your logic
    let args: Vec<String> = std::env::args().collect();
    let filepath = match args.get(1) {
        Some(filepath_arg) => filepath_arg.to_string(),
        _ => env::var("AOC_INPUT").expect("AOC_INPUT not set")
    };
    let (room, chktrails) = part1and2::run(&filepath)?;
    let testroom = room.clone();
    let checktrails = chktrails.clone();

    // Initialize Bevy App
    let mut app = App::new();
    app.add_plugins(DefaultPlugins) // Default plugins for window and rendering
        .add_plugins(EmbeddedPlug)
        .init_state::<AppState>()
        .insert_resource(testroom) // Insert Room as a resource to access in systems
        .insert_resource(checktrails) // Insert Room as a resource to access in systems
        .insert_resource(MoveTimer(Timer::from_seconds(0.05, TimerMode::Repeating))) // Add the timer resource
        .add_systems(Startup,(setup_camera,room_setup))
        .add_systems(Startup,setup_menu)
        .add_systems(Update,menu)
        .add_systems(OnEnter(AppState::Part1),guard_spawn)
        .add_systems(Update,(render_trail,move_guard,update_camera).chain().run_if(in_state(AppState::Part1))) // Set up camera
        .add_systems(OnExit(AppState::Part1),cleanup_p1)
        .run(); // Spawn Room entities

    Ok(())
}

// Components to represent Room elements visually.
#[derive(Component)]
struct Space {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    InputScreen,
    Part1,
    Part2
}

#[derive(Resource)]
struct MoveTimer(Timer);

#[derive(Component)]
struct Guard {
    pathindex: usize,
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
            Sprite::from_image(asset_server.load(get_guard_sprite(&dir,1))),
            Transform::from_translation(Vec3::new(
                x as f32 * SCALED_CELL_SIZE + OFFSET_X,
                y as f32 * -SCALED_CELL_SIZE + OFFSET_Y, // Use -scaled_cell_size for inverted Y
                2.0,
            )),
            Visibility::default(),
            Guard { pathindex: 0, },
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

#[derive(Component)]
struct StateButtonText;
fn setup_menu(mut commands: Commands) {
    let button_entity = commands
        .spawn(Node {
            // center button
            width: Val::Vw(100.),
            height: Val::Vh(100.),
            border: UiRect::axes(Val::Vw(5.), Val::Vh(5.)),
            justify_content: JustifyContent::End,
            align_items: AlignItems::Start,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(150.),
                        height: Val::Px(65.),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Input"),
                        TextFont {
                            font_size: 33.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        StateButtonText,
                    ));
                });
        })
        .id();
}

fn menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut state: ResMut<State<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut button_text: Query<&mut Text, With<StateButtonText>>
) {
    for (interaction, mut color) in &mut interaction_query {
        for mut text in &mut button_text {
            *text = Text::new(match state.get() {
                AppState::InputScreen => "Part 1",
                AppState::Part1 => "Part 2",
                AppState::Part2 => "Input",
            });
            match *interaction {
                Interaction::Pressed => {
                    *color = PRESSED_BUTTON.into();
                    match state.get() {
                        AppState::InputScreen => next_state.set(AppState::Part1),
                        AppState::Part1 => next_state.set(AppState::Part2),
                        AppState::Part2 => next_state.set(AppState::InputScreen),
                    }
                }
                Interaction::Hovered => {
                    *color = HOVERED_BUTTON.into();
                }
                Interaction::None => {
                    *color = NORMAL_BUTTON.into();
                }
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
