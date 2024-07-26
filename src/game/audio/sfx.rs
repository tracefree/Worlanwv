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
        PlaySfx::RandomStep(material) => random_step(material),
    };
    commands.spawn(AudioSourceBundle {
        source: sfx_handles[&sfx_key].clone_weak(),
        settings: PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::new(2.0),
            ..default()
        },
    });
}

pub enum GroundMaterial {
    Grass,
    Solid,
}

/// Trigger this event to play a single sound effect.
#[derive(Event)]
pub enum PlaySfx {
    Key(SfxKey),
    RandomStep(GroundMaterial),
}

fn random_step(ground_material: &GroundMaterial) -> SfxKey {
    match ground_material {
        GroundMaterial::Grass => [
            SfxKey::GrassStep1,
            SfxKey::GrassStep2,
            SfxKey::GrassStep3,
            SfxKey::GrassStep4,
        ],
        GroundMaterial::Solid => [
            SfxKey::HardStep1,
            SfxKey::HardStep2,
            SfxKey::HardStep3,
            SfxKey::HardStep4,
        ],
    }
    .choose(&mut rand::thread_rng())
    .copied()
    .unwrap()
}
