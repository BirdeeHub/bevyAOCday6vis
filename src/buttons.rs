use bevy::prelude::*;
use crate::types::*;

#[derive(Component)]
pub struct StateButtonText;
pub fn setup_menu(mut commands: Commands) {
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

pub fn menu(
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

