//! Spawn the player.

use bevy::prelude::*;

use crate::{
    game::{
        animation::PlayerAnimation,
        assets::{HandleMap, ImageKey},
        movement::{Movement, MovementController},
    },
    screen::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_player);
    app.register_type::<Player>();
}

#[derive(Event, Debug)]
pub struct SpawnPlayer;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Component, Debug)]
pub struct CameraPivot;

fn spawn_player(_trigger: Trigger<SpawnPlayer>, mut commands: Commands) {
    commands
        .spawn((
            Name::new("Player"),
            Player,
            MovementController::default(),
            Movement { speed: 420.0 },
            StateScoped(Screen::Playing),
            SpatialBundle::default(),
        ))
        .with_children(|player| {
            player
                .spawn(CameraPivot)
                .insert(SpatialBundle {
                    transform: Transform::from_xyz(0.0, 1.6, 0.0),
                    ..default()
                })
                .with_children(|pivot| {
                    pivot.spawn(Camera3dBundle {
                        camera: Camera {
                            order: 1,
                            hdr: true,
                            ..default()
                        },
                        projection: PerspectiveProjection {
                            fov: 70.0_f32.to_radians(),
                            ..default()
                        }
                        .into(),
                        ..default()
                    });
                });
        });
}
