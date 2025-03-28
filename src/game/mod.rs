//! Game mechanics and content.

use bevy::prelude::*;

pub mod animation;
pub mod assets;
pub mod audio;
pub mod logic;
mod movement;
pub mod spawn;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        animation::plugin,
        audio::plugin,
        assets::plugin,
        movement::plugin,
        spawn::plugin,
        logic::plugin,
    ));
}
