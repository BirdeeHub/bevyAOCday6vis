use std::io::Result;
use std::env;
use std::task::{Context, Poll};
mod part1and2;
mod types;
mod controls;
mod camera;

use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    tasks::{futures_lite::FutureExt, AsyncComputeTaskPool, Task},
};

use crate::types::*;

fn main() -> Result<()> {

    App::new().add_plugins(DefaultPlugins)
        .add_plugins(EmbeddedPlug)
        .init_state::<AppState>()
        .insert_resource(AllRooms::new())
        .insert_resource(StateInfo::new())
        .insert_resource(MoveTimer(Timer::from_seconds(0.05, TimerMode::Repeating)))
        .add_systems(Startup,(crate::camera::setup_camera,crate::controls::setup_menu))
        .add_systems(Update,crate::controls::menu)
        .add_systems(Update,handle_calc_tasks)
        .add_systems(OnExit(AppState::InputScreen),(load_room, spawn_calc_tasks).chain())
        .add_systems(OnEnter(AppState::Part1),(room_setup, guard_spawn).chain())
        .add_systems(Update,(render_trail,move_guard,crate::camera::update_camera).chain().run_if(in_state(AppState::Part1)))
        .add_systems(OnExit(AppState::Part1),(cleanup_guards, cleanup_room).chain())
        .add_systems(OnEnter(AppState::Part2),(room_setup, sort_guards, guard_spawn).chain())
        .add_systems(Update,(render_trail,move_guard,crate::camera::update_camera,cleanup_non_looping).chain().run_if(in_state(AppState::Part2)))
        .add_systems(OnExit(AppState::Part2),(cleanup_guards, cleanup_room).chain())
        .run();

    Ok(())
}

// TODO: make an input screen, make this happen conditionally when they do that
// give a second button to choose which input they should use by setting StateInfo.room_idx
// and then fetch that and allguards here when input phase exits
// should also give instructions on how to format the input
// use Display impl for room to allow new boards based on old ones
fn load_room(
    mut allrooms: ResMut<AllRooms>,
    mut stateinfo: ResMut<StateInfo>,
) {
    let args: Vec<String> = std::env::args().collect();
    let filepath = match args.get(1) {
        Some(filepath_arg) => filepath_arg.to_string(),
        _ => env::var("AOC_INPUT").expect("AOC_INPUT not set")
    };
    let Ok(filecontents) = crate::part1and2::read_file(&filepath) else { panic!("TESTFILEFAIL AOC_INPUT NOT SET") };

    let Ok((board, guard1, visited)) = crate::part1and2::part1(filecontents) else { panic!("Invalid room!!!"); };

    let mut guards = AllGuards::new();
    guards.push(guard1.clone());

    allrooms.push((board,guards));
    stateinfo.room_idx = Some(0);
    println!("Part 1: total visited: {}", visited);
}

#[derive(Component)]
struct ComputeTrails(Task<CommandQueue>);

fn spawn_calc_tasks(
    mut commands: Commands,
    rooms: Res<AllRooms>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    for (room_index, (room,guards)) in rooms.0.iter().enumerate() {
        if StateInfo::p2_loaded(&room, &guards) { return; }
        let Some(guard1) = guards.get(0) else { return; };
        let init_is_loop = guard1.is_loop;
        let to_check = room.to_check.clone();
        for (i,(x,y)) in to_check.iter().enumerate() {
            let mut room = room.clone();
            let obsx = *x;
            let obsy = *y;
            let task = thread_pool.spawn(async move {
                let newguard = crate::part1and2::part2(&mut room, init_is_loop, obsx,obsy, i+1);
                let mut command_queue = CommandQueue::default();
                // we use a raw command queue to pass a FnOnce(&mut World) back to be applied in a deferred manner.
                command_queue.push(move |world: &mut World| {
                    let Some(mut allrooms) = world.get_resource_mut::<AllRooms>() else { return; };
                    let Some((_, ref mut guards)) = allrooms.get_room_mut(Some(room_index)) else { return; };
                    guards.push(newguard);
                });
                command_queue
            });
            commands.spawn(ComputeTrails(task));
        }
    }
}

fn handle_calc_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ComputeTrails)>,
) {
    for (entity, mut task) in &mut tasks {
        let waker = futures::task::noop_waker();
        let mut context = Context::from_waker(&waker);
        if let Poll::Ready(mut commands_queue) = task.0.poll(&mut context) {
            commands.append(&mut commands_queue);
            commands.entity(entity).despawn();
        }
    }
}

fn room_setup(
    mut commands: Commands,
    rooms: Res<AllRooms>,
    mut stateinfo: ResMut<StateInfo>,
    state: Res<State<AppState>>,
    asset_server: Res<AssetServer>,
) {
    let Some((room, _)) = rooms.get_room(stateinfo.room_idx) else { return; };
    if *state.get() == AppState::Part1 && stateinfo.camera_target != 0 {
        stateinfo.camera_target = 0;
    }
    if *state.get() == AppState::Part2 && stateinfo.camera_target == 0 {
        stateinfo.camera_target = 1;
    }
    for (x, row) in room.iter().enumerate() {
        for (y, cell) in row.iter().enumerate() {
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.5, 0.5, 0.5), // Gray
                    custom_size: Some(Vec2::new(SCALED_CELL_SIZE, SCALED_CELL_SIZE)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(
                    x as f32 * SCALED_CELL_SIZE,
                    y as f32 * -SCALED_CELL_SIZE, // Use -scaled_cell_size for inverted Y
                    0.0,
                )),
                Visibility::default(),
                Space{x,y},
            ));
            if *cell == RoomSpace::Obstacle {
                let mut obssprite = Sprite::from_image(asset_server.load(random_obstacle()));
                obssprite.custom_size = Some(Vec2::new(SCALED_CELL_SIZE, SCALED_CELL_SIZE));
                commands.spawn((
                    obssprite,
                    Transform::from_translation(Vec3::new(
                        x as f32 * SCALED_CELL_SIZE,
                        y as f32 * -SCALED_CELL_SIZE, // Use -scaled_cell_size for inverted Y
                        0.1,
                    )),
                    Visibility::default(),
                    Space{x,y},
                ));
            }
        }
    }
}

fn sort_guards(mut rooms: ResMut<AllRooms>) {
    for (_, ref mut guards) in rooms.iter_mut() {
        guards.sort_by_idx();
    }
}

//TODO: extra ghost obstacles as X in trail color
fn guard_spawn(
    mut commands: Commands,
    rooms: Res<AllRooms>,
    stateinfo: Res<StateInfo>,
    state: Res<State<AppState>>,
    asset_server: Res<AssetServer>,
) {
    let Some((_, guards)) = rooms.get_room(stateinfo.room_idx) else { return; };
    for guard in &guards.0 {
        if *state.get() == AppState::Part1 && guard.display_index != 0 { continue; }
        if let Some((_,(x,y))) = guard.get_loc() {
            commands.spawn((
                Sprite::from_image(asset_server.load(guard.get_sprite())),
                Transform::from_translation(Vec3::new(
                    x as f32 * SCALED_CELL_SIZE,
                    y as f32 * -SCALED_CELL_SIZE,
                    2.0,
                )),
                Visibility::default(),
                guard.clone(),
            ));
        }
    }
}

fn move_guard(
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut guards: Query<(&mut Transform, &mut Sprite, &Guard)>,
) {
    for (mut tform, mut sprite, guard) in &mut guards {
        if let Some((_,(x,y))) = guard.get_loc() {
            let mut direction = Vec3::ZERO;
            direction.x = x as f32 * SCALED_CELL_SIZE;
            direction.y = y as f32 * -SCALED_CELL_SIZE;
            direction.z = tform.translation.z;
            let scalefactor = direction.distance(tform.translation) * SCALED_CELL_SIZE/2.;
            tform.translation.smooth_nudge(&direction, scalefactor, time.delta_secs());
            *sprite = Sprite::from_image(asset_server.load(guard.get_sprite()));
        }
    }
}

//TODO: Rainbow trail with increasing height and decreasing sizes
//TODO: Despawn non-loop ghost obstacles and trail entities upon reaching their end
fn render_trail(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<MoveTimer>,
    state: Res<State<AppState>>,
    mut guards: Query<&mut Guard>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for mut guard in guards.iter_mut() {
            let color = color_from_idx(guard.display_index);
            if guard.trail_idx == 0 && *state.get() == AppState::Part1 {
                if let Some((_,(x,y))) = guard.trail.get(0) {
                    commands.spawn((
                        Sprite {
                            color, // Green
                            custom_size: Some(Vec2::new(SCALED_CELL_SIZE/2., SCALED_CELL_SIZE/2.)),
                            ..default()
                        },
                        Transform::from_translation(Vec3::new(
                            *x as f32 * SCALED_CELL_SIZE,
                            *y as f32 * -SCALED_CELL_SIZE,
                            (guard.display_index as f32)/10.,
                        )),
                        Visibility::default(),
                        TrailEntity::new(guard.trail_idx,guard.display_index),
                    ));
                }
            }
            if let Some((_,(x,y))) = guard.advance() {
                commands.spawn((
                    Sprite {
                        color, // Green
                        custom_size: Some(Vec2::new(SCALED_CELL_SIZE/2., SCALED_CELL_SIZE/2.)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(
                        x as f32 * SCALED_CELL_SIZE,
                        y as f32 * -SCALED_CELL_SIZE,
                        1.,
                    )),
                    Visibility::default(),
                    TrailEntity::new(guard.trail_idx,guard.display_index),
                ));
            } else if *state.get() == AppState::Part2 && !guard.is_loop {
                commands.spawn(ToDelete(guard.display_index));
            }
            if let Some((x,y)) = guard.obstacle {
                if *state.get() == AppState::Part2 {
                    commands.spawn((
                        Sprite {
                            color,
                            custom_size: Some(Vec2::new(SCALED_CELL_SIZE, SCALED_CELL_SIZE/6.)), // Same size
                            ..Default::default()
                        },
                        Transform {
                            rotation: Quat::from_rotation_z(-std::f32::consts::FRAC_PI_4),
                            translation: Vec3::new(x as f32 * SCALED_CELL_SIZE, y as f32 * -SCALED_CELL_SIZE, 1.,),
                            ..Default::default()
                        },
                        Visibility::default(),
                        Obstacle(guard.display_index),
                    ));
                    commands.spawn((
                        Sprite {
                            color,
                            custom_size: Some(Vec2::new(SCALED_CELL_SIZE, SCALED_CELL_SIZE/6.)), // Same size
                            ..Default::default()
                        },
                        Transform {
                            rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_4),
                            translation: Vec3::new(x as f32 * SCALED_CELL_SIZE, y as f32 * -SCALED_CELL_SIZE, 1.,),
                            ..Default::default()
                        },
                        Visibility::default(),
                        Obstacle(guard.display_index),
                    ));
                }
            }
        }
    }
}

fn color_from_idx(idx: usize) -> Color {
    Color::hsv((idx as f32 * 10.) % 360.0, 1., 1.)
}

#[derive(Component)]
struct ToDelete(usize);
#[derive(Component)]
struct Obstacle(usize);

fn cleanup_non_looping(
    mut commands: Commands,
    non_looping: Query<(Entity, &ToDelete)>,
    guards: Query<(Entity, &Guard)>,
    trails: Query<(Entity, &TrailEntity)>,
    obstacles: Query<(Entity, &Obstacle)>,
) {
    let mut to_delete = Vec::new();
    for (entity, id) in non_looping.iter() {
        to_delete.push(id.0);
        commands.entity(entity).despawn_recursive();
    }
    for (entity, guard) in guards.iter() {
        if to_delete.contains(&guard.display_index) {
            commands.entity(entity).despawn_recursive();
        }
    }
    for (entity, trail) in trails.iter() {
        if to_delete.contains(&trail.guard_index) {
            commands.entity(entity).despawn_recursive();
        }
    }
    for (entity, obs) in obstacles.iter() {
        if to_delete.contains(&obs.0) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn cleanup_room(mut commands: Commands, items: Query<Entity, With<Space>>) {
    for entity in items.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn cleanup_guards(
    mut commands: Commands,
    guard: Query<Entity, With<Guard>>,
    trail: Query<Entity, With<TrailEntity>>,
    obstacles: Query<Entity, With<Obstacle>>,
) {
    for entity in guard.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in trail.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in obstacles.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
