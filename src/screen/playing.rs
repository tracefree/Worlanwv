//! The screen state for the main game loop.

use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    ui::Val::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use super::{PlayState, Screen};
use crate::game::{
    assets::SoundtrackKey, audio::soundtrack::PlaySoundtrack, logic::PromptText,
    spawn::level::SpawnLevel,
};
use crate::ui::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Playing), enter_playing);
    app.add_systems(OnExit(Screen::Playing), exit_playing);
    app.add_systems(OnEnter(PlayState::InMenu), enter_menu);
    app.add_systems(OnExit(PlayState::InMenu), exit_menu);
    app.enable_state_scoped_entities::<PlayState>();

    app.add_systems(
        Update,
        toggle_pause
            .run_if(in_state(Screen::Playing).and_then(input_just_pressed(KeyCode::Escape))),
    );
    app.add_systems(
        Update,
        handle_menu_action.run_if(in_state(PlayState::InMenu)),
    );
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
enum TitleAction {
    Play,
    Credits,
    /// Exit doesn't work well with embedded applications.
    #[cfg(not(target_family = "wasm"))]
    Exit,
}

fn enter_menu(mut commands: Commands, mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = windows.single_mut();
    primary_window.cursor.grab_mode = CursorGrabMode::None;
    primary_window.cursor.visible = true;
    commands
        .ui_root()
        .insert(StateScoped(PlayState::InMenu))
        .with_children(|root| {
            root.spawn(NodeBundle {
                style: Style {
                    width: Percent(30.0),
                    height: Percent(65.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    align_self: AlignSelf::Start,
                    flex_direction: FlexDirection::Column,
                    row_gap: Px(10.0),
                    position_type: PositionType::Absolute,
                    left: Px(0.0),
                    bottom: Px(0.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|container| {
                container.button("Play").insert(TitleAction::Play);
                container.button("Credits").insert(TitleAction::Credits);

                #[cfg(not(target_family = "wasm"))]
                container.button("Exit").insert(TitleAction::Exit);
            });
        });
}

fn exit_menu(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = windows.single_mut();
    primary_window.cursor.grab_mode = CursorGrabMode::Locked;
    primary_window.cursor.visible = false;
}

fn enter_playing(mut commands: Commands, mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    commands.trigger(SpawnLevel);
    commands.trigger(PlaySoundtrack::Key(SoundtrackKey::OceanAmbiance));
    commands.trigger(PlaySoundtrack::Key(SoundtrackKey::CycleOne));

    // Grab cursor
    let mut primary_window = windows.single_mut();
    primary_window.cursor.grab_mode = CursorGrabMode::Locked;
    primary_window.cursor.visible = false;

    // Spawn prompt
    commands.ui_root().with_children(|root| {
        root.spawn(NodeBundle {
            style: Style {
                // width: Percent(30.0),
                height: Percent(100.0),
                justify_content: JustifyContent::End,
                align_items: AlignItems::End,
                align_self: AlignSelf::Center,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Percent(5.0)),
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        })
        .with_children(|container| {
            container
                .spawn(TextBundle {
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font_size: 32.0,
                            ..default()
                        },
                    ),
                    ..default()
                })
                .insert(PromptText);
        });
    });
}

fn exit_playing(mut commands: Commands, mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    // We could use [`StateScoped`] on the sound playing entites instead.
    commands.trigger(PlaySoundtrack::Disable);

    // Release cursor
    let mut primary_window = windows.single_mut();
    primary_window.cursor.grab_mode = CursorGrabMode::None;
    primary_window.cursor.visible = true;
}

fn toggle_pause(
    current_state: Res<State<PlayState>>,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    next_state.set(match current_state.get() {
        PlayState::InGame => PlayState::InMenu,
        PlayState::InMenu => PlayState::InGame,
    });
}

fn handle_menu_action(
    mut next_screen: ResMut<NextState<PlayState>>,
    mut button_query: InteractionQuery<&TitleAction>,
    #[cfg(not(target_family = "wasm"))] mut app_exit: EventWriter<AppExit>,
) {
    for (interaction, action) in &mut button_query {
        if matches!(interaction, Interaction::Pressed) {
            match action {
                TitleAction::Play => next_screen.set(PlayState::InGame),
                TitleAction::Credits => {} //next_screen.set(Screen::Credits),
                #[cfg(not(target_family = "wasm"))]
                TitleAction::Exit => {
                    app_exit.send(AppExit::Success);
                }
            }
        }
    }
}
