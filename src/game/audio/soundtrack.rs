use bevy::{audio::PlaybackMode, prelude::*};

use crate::game::assets::{HandleMap, SoundtrackKey};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<IsSoundtrack>();
    app.observe(play_soundtrack);
}

fn play_soundtrack(
    trigger: Trigger<PlaySoundtrack>,
    mut commands: Commands,
    soundtrack_handles: Res<HandleMap<SoundtrackKey>>,
    soundtrack_query: Query<Entity, With<IsSoundtrack>>,
) {
    if let PlaySoundtrack::Key(soundtrack_key) = trigger.event() {
        let mut audio_player = commands.spawn((AudioSourceBundle {
            source: soundtrack_handles[soundtrack_key].clone_weak(),
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                ..default()
            },
        },));
        if soundtrack_key == &SoundtrackKey::CycleOne {
            audio_player.insert(IsSoundtrack);
        }
    } else {
        println!("Desp");
        for track in soundtrack_query.iter() {
            commands.entity(track).despawn();
        }
    }
}

/// Trigger this event to play or disable the soundtrack.
/// Playing a new soundtrack will overwrite the previous one.
/// Soundtracks will loop.
#[derive(Event)]
pub enum PlaySoundtrack {
    Key(SoundtrackKey),
    Disable,
}

/// Marker component for the soundtrack entity so we can find it later.
#[derive(Component, Reflect)]
#[reflect(Component)]
struct IsSoundtrack;
