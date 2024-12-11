use std::io::Result;
use std::env;
use bevy::prelude::*;
mod part1and2;
mod types;
mod asset;
mod buttons;
mod camera;

use crate::types::*;

use crate::asset::{EmbeddedPlug, get_guard_sprite};

fn main() -> Result<()> {

    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(EmbeddedPlug)
        .init_state::<AppState>()
        .insert_resource(AllRooms::new())
        .insert_resource(StateInfo::new())
        .insert_resource(MoveTimer(Timer::from_seconds(0.05, TimerMode::Repeating)))
        .add_systems(Startup,(crate::camera::setup_camera,crate::buttons::setup_menu))
        .add_systems(Update,(calc_room,crate::buttons::menu))
        .add_systems(OnExit(AppState::InputScreen),load_room)
        .add_systems(OnEnter(AppState::Part1),(room_setup, guard_spawn).chain())
        .add_systems(Update,(render_trail,move_guard,crate::camera::update_camera).chain().run_if(in_state(AppState::Part1)))
        .add_systems(OnExit(AppState::Part1),(cleanup_guards, cleanup_room).chain())
        .add_systems(OnEnter(AppState::Part2),(room_setup, guard_spawn).chain())
        .add_systems(Update,(render_trail,move_guard,crate::camera::update_camera).chain().run_if(in_state(AppState::Part2)))
        .add_systems(OnExit(AppState::Part2),(cleanup_guards, cleanup_room).chain())
        .run();

    Ok(())
}

fn calc_room(
    mut allrooms: ResMut<AllRooms>,
) {
    for (room,guards) in &mut allrooms.0 {
        if StateInfo::p2_loaded(&room, &guards) { continue; }
        let Some(guard1) = guards.get(0) else { continue; };
        let init_is_loop = guard1.is_loop;
        for (i,(x,y)) in room.to_check.iter().enumerate() {
            println!("{} / {}",i,room.to_check.len());
            guards.push(crate::part1and2::part2(&mut room.clone(), init_is_loop, *x,*y, i));
        }
        let obstacles = crate::part1and2::deduplicate_vec(guards.iter().filter(|v|v.is_loop).collect()).len();
        println!("Part 2: possible obstacle locations for loop: {:?}",obstacles);
    }
}

fn load_room(
    mut allrooms: ResMut<AllRooms>,
    mut stateinfo: ResMut<StateInfo>,
) {
    // Get the Room and trails from your logic
    let args: Vec<String> = std::env::args().collect();
    let filepath = match args.get(1) {
        Some(filepath_arg) => filepath_arg.to_string(),
        _ => env::var("AOC_INPUT").expect("AOC_INPUT not set")
    };
    let Ok(filecontents) = crate::part1and2::read_file(&filepath) else { panic!("TESTFILEFAIL AOC_INPUT NOT SET") };

    let (board, guard1, visited) = crate::part1and2::part1(filecontents);

    let mut guards = AllGuards::new();
    guards.push(guard1.clone());

    allrooms.push((board,guards));
    stateinfo.room_idx = Some(0);
    println!("Part 1: total visited: {}", visited);
}

fn room_setup(
    mut commands: Commands,
    rooms: Res<AllRooms>,
    stateinfo: Res<StateInfo>,
    asset_server: Res<AssetServer>,
) {
    let Some((room, guards)) = rooms.get_room(stateinfo.room_idx) else { return; };
    for (x, row) in room.iter().enumerate() {
        for (y, cell) in row.iter().enumerate() {
            let sprite = match cell {
                RoomSpace::Obstacle => Sprite {
                    color: Color::srgb(0.0, 0.0, 0.0), // Black
                    custom_size: Some(Vec2::new(SCALED_CELL_SIZE, SCALED_CELL_SIZE)),
                    ..default()
                },
                _ => Sprite {
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
                GridEntity,
            ));
        }
    }
}

fn guard_spawn(
    mut commands: Commands,
    rooms: Res<AllRooms>,
    stateinfo: Res<StateInfo>,
    mut state: Res<State<AppState>>,
    asset_server: Res<AssetServer>,
) {
    let Some((room, guards)) = rooms.get_room(stateinfo.room_idx) else { return; };
    if ! StateInfo::p1_loaded(&guards) && ! StateInfo::p2_loaded(&room,&guards) { return; }
    for guard in &guards.0 {
        if let Some((dir,(x,y))) = guard.get_loc() {
            commands.spawn((
                Sprite::from_image(asset_server.load(get_guard_sprite(&dir,1))),
                Transform::from_translation(Vec3::new(
                    x as f32 * SCALED_CELL_SIZE + OFFSET_X,
                    y as f32 * -SCALED_CELL_SIZE + OFFSET_Y,
                    2.0,
                )),
                Visibility::default(),
                guard.clone(),
            ));
        }
        if *state == AppState::Part1 { break; }
    }
}

fn move_guard(
    mut commands: Commands,
    rooms: Res<AllRooms>,
    stateinfo: Res<StateInfo>,
    time: Res<Time>,
    state: Res<State<AppState>>,
    asset_server: Res<AssetServer>,
    mut guardquery: Query<(Entity, &mut Transform, &mut Sprite, &mut Guard)>,
) {
    let Some((room, guards)) = rooms.get_room(stateinfo.room_idx) else { return; };
    if ! StateInfo::p1_loaded(&guards) && ! StateInfo::p2_loaded(&room,&guards) { return; }
    for (entity, mut tform, mut sprite, guard) in &mut guardquery {
        if let Some((dir,(x,y))) = guard.get_loc() {
            let mut direction = Vec3::ZERO;
            direction.x = (x as f32 * SCALED_CELL_SIZE + OFFSET_X) - tform.translation.x;
            direction.y = (y as f32 * -SCALED_CELL_SIZE + OFFSET_Y) - tform.translation.y;
            if direction != Vec3::ZERO {
                tform.translation += direction * SCALED_CELL_SIZE * time.delta_secs();
            }
            *sprite = Sprite::from_image(asset_server.load(get_guard_sprite(&dir,1)));
        }
    }
}

fn render_trail(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<MoveTimer>,
    asset_server: Res<AssetServer>,
    mut rooms: ResMut<AllRooms>,
    stateinfo: Res<StateInfo>,
    querytrail: Query<(Entity, &TrailEntity)>,
    mut guardquery: Query<(Entity, &mut Guard)>,
) {
    let Some((room, guards)) = rooms.get_room_mut(stateinfo.room_idx) else { return; };
    if timer.0.tick(time.delta()).just_finished() && StateInfo::p1_loaded(&guards) && StateInfo::p2_loaded(&room,&guards) {
        for (entity, mut guard) in guardquery.iter_mut() {
            guard.advance();
            let mut final_idx = 0;
            let mut has_zero = false;
            for (entity, trailidx) in querytrail.iter() {
                if trailidx.guard_index == guard.display_index {
                    if let Some(_) = guard.trail.get(trailidx.index) {
                        if trailidx.index == 0 { has_zero = true; }
                        final_idx = trailidx.index;
                    } else {
                        commands.entity(entity).despawn();
                    }
                }
            }
            if !has_zero {
                if let Some((dir,(x,y))) = guard.trail.get(0) {
                    commands.spawn((
                        Sprite {
                            color: Color::srgb(0.0, 1.0, 0.0), // Green
                            custom_size: Some(Vec2::new(SCALED_CELL_SIZE/2., SCALED_CELL_SIZE/2.)),
                            ..default()
                        },
                        Transform::from_translation(Vec3::new(
                            *x as f32 * SCALED_CELL_SIZE + OFFSET_X,
                            *y as f32 * -SCALED_CELL_SIZE + OFFSET_Y,
                            1.0,
                        )),
                        Visibility::default(),
                        TrailEntity::new(0, guard.display_index),
                    ));
                }
            }
            for i in (final_idx+1)..guard.get_current_trail().len() {
                if let Some((dir,(x,y))) = guard.trail.get(i) {
                    commands.spawn((
                        Sprite {
                            color: Color::srgb(0.0, 1.0, 0.0), // Green
                            custom_size: Some(Vec2::new(SCALED_CELL_SIZE/2., SCALED_CELL_SIZE/2.)),
                            ..default()
                        },
                        Transform::from_translation(Vec3::new(
                            *x as f32 * SCALED_CELL_SIZE + OFFSET_X,
                            *y as f32 * -SCALED_CELL_SIZE + OFFSET_Y,
                            1.0,
                        )),
                        Visibility::default(),
                        TrailEntity::new(i, guard.display_index),
                    ));
                }
            }
        }
    }
}

fn cleanup_room(mut commands: Commands, items: Query<Entity, With<GridEntity>>) {
    for entity in items.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn cleanup_guards(mut commands: Commands, guard: Query<Entity, With<Guard>>, trail: Query<Entity, With<TrailEntity>>) {
    for entity in guard.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in trail.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
