use crate::types::*;
use bevy::prelude::*;
use bevy::ui::ZIndex;
use std::io::{self};
use bevy::asset::{AssetLoader, io::Reader, LoadContext};
use bevy::reflect::TypePath;
use bevy_egui::{egui, EguiContexts};
use egui::{Order, Id, Pos2};

pub fn add_examples(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut stateinfo: ResMut<StateInfo>,
) {
    if stateinfo.examples_loaded { return; };
    for i in 1..=4 {
        let file = asset_server.load("embedded://day6vis/examples/input".to_string() + &i.to_string() + ".txt");
        commands.spawn(TextHandle(file));
    }
    stateinfo.examples_loaded = true;
}

pub fn load_examples(
    mut commands: Commands,
    text_assets: Res<Assets<TextAsset>>,
    query: Query<(Entity, &TextHandle)>
) {
    for (ent, newtext) in query.iter() {
        if let Some(val) = text_assets.get(&newtext.0) {
            commands.spawn(InputText(val.0.clone()));
            commands.entity(ent).despawn();
        };
    }
}

#[derive(Asset, TypePath, Debug)]
pub struct TextAsset(pub String);

#[derive(Component)]
pub struct TextHandle(pub Handle<TextAsset>);

#[derive(Default)]
pub struct TextAssetLoader;

impl AssetLoader for TextAssetLoader {
    type Asset = TextAsset;
    type Settings = ();
    type Error = std::io::Error;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        // Convert bytes to String
        if let Ok(val) = String::from_utf8(bytes) {
            Ok(TextAsset(val))
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "UTF-8 error"))
        }
    }
    fn extensions(&self) -> &[&str] {
        &["txt"]
    }
}

pub fn handle_input(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut stateinfo: ResMut<StateInfo>,
    rooms: Res<AllRooms>,
    mut pending_text: ResMut<PendingText>,
    mut current_error: ResMut<CurrentError>,
    err_query: Query<(Entity, &ErrorBox)>,
) {
    egui::Area::new(Id::new("input_area")).order(Order::Background).show(contexts.ctx_mut(), |ui| {
        ui.vertical(|ui| {
            ui.label("Select a saved room, then click on Part 1:");
            for i in 0..rooms.len() {
                ui.radio_value(&mut stateinfo.room_idx, Some(i), i.to_string());
            }
        });
        ui.horizontal(|ui| {
            ui.button("New").clicked().then(|| {
                pending_text.0.clear();
            });
            ui.button("Edit").clicked().then(|| {
                for (room,_) in rooms.iter() {
                    if Some(room.index) == stateinfo.room_idx {
                        pending_text.0.clear();
                        pending_text.0 = format!("{}", room);
                    }
                }
            });
            ui.button("Submit").clicked().then(|| {
                commands.spawn(InputText(pending_text.0.clone()));
                pending_text.0.clear();
            });
        });
        ui.text_edit_multiline(&mut pending_text.0);
        for (ent, err) in err_query.iter() {
            current_error.0 = err.0.clone();
            commands.entity(ent).despawn();
        };
        ui.label(&current_error.0);
    });
}

pub fn setup_menu(mut commands: Commands) {
    commands.spawn((
        Node {
            // center button
            width: Val::Vw(100.),
            height: Val::Vh(100.),
            border: UiRect::axes(Val::Vw(5.), Val::Vh(5.)),
            justify_content: JustifyContent::End,
            align_items: AlignItems::Start,
            ..default()
        },
        ZIndex(100),
        MenuParent,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("Loops Found: 0"),
            Node {
                padding: UiRect::all(Val::Px(5.)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            TextFont {
                font_size: 33.0,
                ..default()
            },
            LoopBoard(0),
            Visibility::Hidden,
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ZIndex(99),
        ));
        parent.spawn((
            Button,
            Node {
                width: Val::Px(150.),
                height: Val::Px(65.),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            StateButton,
            ZIndex(99),
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
                ZIndex(100),
                StateButtonText,
            ));
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(10.0), // Progress bar height
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::End,
                    ..default()
                },
                ZIndex(100),
            )).with_children(|progress_parent| {
                // Progress bar fill
                progress_parent.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    Visibility::Hidden,
                    ZIndex(101),
                    BackgroundColor(Color::srgb(0.9, 0.9, 0.9)), // Bar color
                    ProgressBarFill, // Custom marker to identify the progress bar fill
                ));
            });
        });
    });
}

pub fn prog_update_system(
    stateinfo: Res<StateInfo>,
    rooms: Res<AllRooms>,
    tasks: Query<Entity, With<ComputeTrails>>,
    mut fill_bar: Query<(&mut Node, &mut Visibility), With<ProgressBarFill>>,
) {
    let Some((room, _)) = rooms.get_room(stateinfo.room_idx) else {
        return;
    };
    let num_tasks = tasks.iter().count() as f32;
    let total_tasks = room.to_check.len() as f32;
    for (mut node, mut vis) in &mut fill_bar {
        if num_tasks == 0.0 {
            *vis = Visibility::Hidden;
        } else {
            if *vis == Visibility::Hidden {
                *vis = Visibility::Visible;
            };
            node.width = Val::Percent(num_tasks / total_tasks * 100.);
        }
    }
}

pub fn guard_controls(
    mut contexts: EguiContexts,
    mut stateinfo: ResMut<StateInfo>,
    mut timer: ResMut<MoveTimer>,
    state: Res<State<AppState>>,
    rooms: Res<AllRooms>,
    st_but: Query<&GlobalTransform, With<StateButton>>,
) {
    let Some((_, guards)) = rooms.get_room(stateinfo.room_idx) else {
        return;
    };
    let Ok(global_transform) = st_but.get_single() else { return; };
    let newtran = global_transform.translation();
    egui::Area::new(Id::new("guard_select")).order(Order::Background).fixed_pos(Pos2::new(newtran.x, newtran.y+50.)).show(contexts.ctx_mut(), |ui| {
        let mut newtime = timer.0.duration().as_millis() as u64;
        ui.add(
            egui::Slider::new(&mut newtime, 0..=750)
                .text("Tick Rate").step_by(1.0),
        );
        timer.0.set_duration(std::time::Duration::from_millis(newtime));
        if *state.get() == AppState::Part2 {
            ui.add(
                egui::Slider::new(&mut stateinfo.camera_target, 0..=(guards.len() - 1))
                    .text("Focused Guard").step_by(1.0),
            );
        };
    });
}

pub fn menu(
    mut next_state: ResMut<NextState<AppState>>,
    state: Res<State<AppState>>,
    rooms: Res<AllRooms>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<StateButton>),
    >,
    stateinfo: Res<StateInfo>,
    mut button_text: Query<&mut Text, With<StateButtonText>>,
) {
    let p2loaded = if let Some((room, guards)) = rooms.get_room(stateinfo.room_idx) {
        StateInfo::p2_loaded(room, guards)
    } else {
        false
    };
    //if *state.get() == AppState::InputScreen { return; };
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
                        AppState::InputScreen => {
                            if rooms.get_room(stateinfo.room_idx).is_some() {
                                next_state.set(AppState::Part1);
                            }
                        }
                        AppState::Part1 => {
                            if p2loaded {
                                next_state.set(AppState::Part2)
                            }
                        }
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
