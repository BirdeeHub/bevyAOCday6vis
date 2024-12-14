use std::task::{Context, Poll};

use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    tasks::{futures_lite::FutureExt, AsyncComputeTaskPool},
};
use bevy_egui::EguiPlugin;

use crate::types::*;
use crate::camera::*;
use crate::controls::*;

pub fn run() {
    App::new().add_plugins(DefaultPlugins)
        .add_plugins(EmbeddedPlug)
        .add_plugins(EguiPlugin)
        .init_state::<AppState>()
        .init_asset::<TextAsset>() // Register the custom asset type
        .init_asset_loader::<TextAssetLoader>() // Register the custom loader
        .insert_resource(AllRooms::new())
        .insert_resource(StateInfo::new())
        .insert_resource(PendingText(String::new()))
        .insert_resource(CurrentError(String::new()))
        .insert_resource(MoveTimer(Timer::from_seconds(0.05, TimerMode::Repeating)))
        .add_systems(Startup,(setup_camera,setup_menu))
        .add_systems(Update,menu)
        .add_systems(Update,handle_calc_tasks)
        .add_systems(OnEnter(AppState::InputScreen),add_examples)
        .add_systems(Update,(load_inputs,handle_input,load_examples).run_if(in_state(AppState::InputScreen)))
        .add_systems(OnExit(AppState::InputScreen),(spawn_calc_tasks).chain())
        .add_systems(OnEnter(AppState::Part1),(room_setup, guard_spawn).chain())
        .add_systems(Update,(
            render_trail,
            move_guard,
            update_camera,
        ).chain().run_if(in_state(AppState::Part1)))
        .add_systems(Update,(guard_controls,prog_update_system,resize_trails).run_if(in_state(AppState::Part1)))
        .add_systems(OnExit(AppState::Part1),cleanup_room)
        .add_systems(OnEnter(AppState::Part2),(room_setup, sort_guards, guard_spawn).chain())
        .add_systems(Update,(
            render_trail,
            move_guard,
            update_camera,
            cleanup_non_looping,
        ).chain().run_if(in_state(AppState::Part2)))
        .add_systems(Update,(guard_controls,resize_trails).run_if(in_state(AppState::Part2)))
        .add_systems(OnExit(AppState::Part2),cleanup_room)
        .run();
}

pub fn load_inputs(
    mut commands: Commands,
    mut rooms: ResMut<AllRooms>,
    input_text: Query<(Entity,&InputText)>,
) {
    for (ent, text) in &input_text {
        match crate::part1and2::part1(text.0.clone()) {
            Ok(mut new_room) => {
                new_room.0.index = rooms.0.len();
                rooms.push(new_room);
            }
            Err(err) => {
                let final_msg = format!("Error: {}. Caused by input:\n{}", err, text.0.clone());
                println!("{}", final_msg);
                commands.spawn(ErrorBox(final_msg));
            }
        }
        commands.entity(ent).despawn();
    }
}

fn spawn_calc_tasks(
    mut commands: Commands,
    rooms: Res<AllRooms>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    for (room,guards) in rooms.0.iter() {
        if StateInfo::p2_loaded(room, guards) { continue; }
        let Some(guard1) = guards.first() else { continue; };
        let init_is_loop = guard1.is_loop;
        let to_check = room.to_check.clone();
        for (i,(x,y)) in to_check.iter().enumerate() {
            let room = room.clone();
            let obsx = *x;
            let obsy = *y;
            let task = thread_pool.spawn(async move {
                let newguard = crate::part1and2::part2(&room, init_is_loop, obsx,obsy, i+1);
                let mut command_queue = CommandQueue::default();
                // we use a raw command queue to pass a FnOnce(&mut World) back to be applied in a deferred manner.
                command_queue.push(move |world: &mut World| {
                    let Some(mut allrooms) = world.get_resource_mut::<AllRooms>() else { return; };
                    let Some((_, ref mut guards)) = allrooms.get_room_mut(Some(room.index)) else { return; };
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
    mut loopquery: Query<&mut Visibility, With<LoopBoard>>,
) {
    let Some((room, _)) = rooms.get_room(stateinfo.room_idx) else { return; };
    if *state.get() == AppState::Part1 && stateinfo.camera_target != 0 {
        stateinfo.camera_target = 0;
    }
    if *state.get() == AppState::Part2 && stateinfo.camera_target == 0 {
        stateinfo.camera_target = 1;
        for mut vis in &mut loopquery {
            *vis = Visibility::default()
        }
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
        let color = color_from_idx(guard.display_index);
        if let Some((x,y)) = guard.obstacle {
            if *state.get() == AppState::Part2 {
                commands.spawn((
                    Sprite {
                        color,
                        custom_size: Some(Vec2::new(SCALED_CELL_SIZE, SCALED_CELL_SIZE/6.)),
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
                        custom_size: Some(Vec2::new(SCALED_CELL_SIZE, SCALED_CELL_SIZE/6.)),
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

fn resize_trails(
    stateinfo: Res<StateInfo>,
    mut trailent: Query<(&TrailEntity, &mut Sprite), Without<Obstacle>>,
    mut obstacles: Query<(&Obstacle, &mut Sprite)>
) {
    for (entobj, mut sprite) in trailent.iter_mut() {
        let custom_size = if entobj.guard_index == stateinfo.camera_target {
            Some(Vec2::new(SCALED_CELL_SIZE/2., SCALED_CELL_SIZE/2.))
        } else {
            Some(Vec2::new(SCALED_CELL_SIZE/4., SCALED_CELL_SIZE/4.))
        };
        sprite.custom_size = custom_size;
    };
    for (entobj, mut sprite) in obstacles.iter_mut() {
        let custom_size = if entobj.0 == stateinfo.camera_target {
            Some(Vec2::new(SCALED_CELL_SIZE, SCALED_CELL_SIZE/6.))
        } else {
            Some(Vec2::new(SCALED_CELL_SIZE/2., SCALED_CELL_SIZE/6.))
        };
        sprite.custom_size = custom_size;
    };
}

fn render_trail(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<MoveTimer>,
    state: Res<State<AppState>>,
    stateinfo: Res<StateInfo>,
    mut guards: Query<&mut Guard>,
    mut loopquery: Query<(&mut Text, &mut LoopBoard)>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for mut guard in guards.iter_mut() {
            let custom_size = if guard.display_index == stateinfo.camera_target {
                Some(Vec2::new(SCALED_CELL_SIZE/2., SCALED_CELL_SIZE/2.))
            } else {
                Some(Vec2::new(SCALED_CELL_SIZE/4., SCALED_CELL_SIZE/4.))
            };
            let color = color_from_idx(guard.display_index);
            if guard.trail_idx == 0 && *state.get() == AppState::Part1 {
                if let Some((_,(x,y))) = guard.trail.first() {
                    commands.spawn((
                        Sprite {
                            color,
                            custom_size,
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
                        color,
                        custom_size,
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
            } else if *state.get() == AppState::Part2 {
                if guard.is_loop && !guard.counted {
                    guard.counted = true;
                    for (mut looptext, mut loopboard) in loopquery.iter_mut() {
                        loopboard.0 += 1;
                        *looptext = Text::new(format!("Loops Found: {}", loopboard.0));
                    }
                } else {
                    commands.spawn(ToDelete(guard.display_index));
                }
            }
        }
    }
}

fn cleanup_non_looping(
    mut commands: Commands,
    non_looping: Query<(Entity, &ToDelete)>,
    guards: Query<(Entity, &Guard)>,
    trails: Query<(Entity, &TrailEntity)>,
    obstacles: Query<(Entity, &Obstacle)>,
) {
    for (entity, id) in non_looping.iter() {
        commands.entity(entity).despawn_recursive();
        for (entity, guard) in guards.iter() {
            if guard.display_index == id.0 {
                commands.entity(entity).despawn_recursive();
            }
        }
        for (entity, trail) in trails.iter() {
            if trail.guard_index == id.0 {
                commands.entity(entity).despawn_recursive();
            }
        }
        for (entity, obs) in obstacles.iter() {
            if obs.0 == id.0 {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn cleanup_room(
    mut commands: Commands,
    items: Query<Entity, With<Space>>,
    guard: Query<Entity, With<Guard>>,
    trail: Query<Entity, With<TrailEntity>>,
    obstacles: Query<Entity, With<Obstacle>>,
    mut loopquery: Query<(&mut Text, &mut LoopBoard, &mut Visibility)>,
) {
    for entity in items.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in guard.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in trail.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in obstacles.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for (mut text,mut loopboard,mut vis) in loopquery.iter_mut() {
        *text = Text::new("Loops Found: 0");
        loopboard.0 = 0;
        *vis = Visibility::Hidden;
    }
}
