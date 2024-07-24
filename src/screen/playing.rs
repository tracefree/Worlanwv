//! The screen state for the main game loop.

use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use super::{PlayState, Screen};
use crate::game::{
    assets::SoundtrackKey, audio::soundtrack::PlaySoundtrack, spawn::level::SpawnLevel,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Playing), enter_playing);
    app.add_systems(OnExit(Screen::Playing), exit_playing);
    app.add_systems(OnEnter(PlayState::InMenu), enter_menu);
    app.add_systems(OnExit(PlayState::InMenu), exit_menu);

    app.add_systems(
        Update,
        toggle_pause
            .run_if(in_state(Screen::Playing).and_then(input_just_pressed(KeyCode::Escape))),
    );
}

fn enter_menu(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = windows.single_mut();
    primary_window.cursor.grab_mode = CursorGrabMode::None;
    primary_window.cursor.visible = true;
}

fn exit_menu(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = windows.single_mut();
    primary_window.cursor.grab_mode = CursorGrabMode::Locked;
    primary_window.cursor.visible = false;
}

fn enter_playing(mut commands: Commands, mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    commands.trigger(SpawnLevel);
    commands.trigger(PlaySoundtrack::Key(SoundtrackKey::OceanAmbiance));

    // Grab cursor
    let mut primary_window = windows.single_mut();
    primary_window.cursor.grab_mode = CursorGrabMode::Locked;
    primary_window.cursor.visible = false;
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
