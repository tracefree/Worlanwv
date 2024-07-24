//! The game's main screen states and transitions between them.

mod credits;
mod loading;
mod playing;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();
    app.add_sub_state::<PlayState>();
    app.enable_state_scoped_entities::<Screen>();

    app.add_plugins((loading::plugin, credits::plugin, playing::plugin));
}

/// The game's main screen states.
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum Screen {
    #[default]
    Loading,
    Playing,
    Credits,
}

#[derive(SubStates, Debug, Hash, PartialEq, Eq, Clone, Default)]
#[source(Screen = Screen::Playing)]
pub enum PlayState {
    #[default]
    InMenu,
    InGame,
}
