use bevy::prelude::*;
use crate::types::*;

pub fn setup_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands
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
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    StateButton,
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
                    parent.spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(10.0), // Progress bar height
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::End,
                        ..default()
                    })
                    .with_children(|progress_parent| {
                        // Progress bar fill
                        progress_parent.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            Visibility::Hidden,
                            BackgroundColor(Color::srgb(0.9, 0.9, 0.9)), // Bar color
                            ProgressBarFill, // Custom marker to identify the progress bar fill
                        ));
                    });
                });
        });
}

pub fn prog_update_system(
    mut commands: Commands,
    stateinfo: Res<StateInfo>,
    rooms: Res<AllRooms>,
    tasks: Query<Entity, With<ComputeTrails>>,
    mut fill_bar: Query<(&mut Node, &mut Visibility), With<ProgressBarFill>>
) {
    let Some((room, guards)) = rooms.get_room(stateinfo.room_idx) else {
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
    };
}

//TODO: add a slider for speed.
//TODO: add a slider with optional number input/display to select/see which guard to follow in part 2.

pub fn menu(
    mut next_state: ResMut<NextState<AppState>>,
    state: Res<State<AppState>>,
    rooms: Res<AllRooms>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<StateButton>),
    >,
    stateinfo: Res<StateInfo>,
    mut button_text: Query<&mut Text, With<StateButtonText>>
) {
    let p2loaded = if let Some((room, guards)) = rooms.get_room(stateinfo.room_idx) {
        StateInfo::p2_loaded(room, guards)
    } else { false };
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
                        AppState::Part1 => {
                            if p2loaded {
                                next_state.set(AppState::Part2)
                            }
                        },
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

