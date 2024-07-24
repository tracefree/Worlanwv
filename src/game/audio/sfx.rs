use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};
use rand::seq::SliceRandom;

use crate::game::assets::{HandleMap, SfxKey};

pub(super) fn plugin(app: &mut App) {
    app.observe(play_sfx);
}

fn play_sfx(
    trigger: Trigger<PlaySfx>,
    mut commands: Commands,
    sfx_handles: Res<HandleMap<SfxKey>>,
) {
    let sfx_key = match trigger.event() {
        PlaySfx::Key(key) => *key,
        PlaySfx::RandomStep => random_step(),
    };
    commands.spawn(AudioSourceBundle {
        source: sfx_handles[&sfx_key].clone_weak(),
        settings: PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::new(10.0),
            ..default()
        },
    });
}

/// Trigger this event to play a single sound effect.
#[derive(Event)]
pub enum PlaySfx {
    Key(SfxKey),
    RandomStep,
}

fn random_step() -> SfxKey {
    [
        SfxKey::GrassStep1,
        SfxKey::GrassStep2,
        SfxKey::GrassStep3,
        SfxKey::GrassStep4,
    ]
    .choose(&mut rand::thread_rng())
    .copied()
    .unwrap()
}
