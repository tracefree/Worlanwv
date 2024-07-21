//! Spawn the player.

use bevy::{math::VectorSpace, prelude::*};
use bevy_rapier3d::{
    control::KinematicCharacterController,
    dynamics::{RigidBody, Velocity},
    geometry::Collider,
};

use crate::{
    game::movement::{Movement, MovementController},
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
            SpatialBundle::from_transform(Transform::from_xyz(0.0, 2.8, 0.0)),
            RigidBody::KinematicPositionBased,
            Velocity {
                linvel: Vec3::ZERO,
                angvel: Vec3::ZERO,
            },
            KinematicCharacterController::default(),
            Collider::capsule_y(0.5, 0.3),
        ))
        .with_children(|player| {
            player
                .spawn(CameraPivot)
                .insert(SpatialBundle {
                    transform: Transform::from_xyz(0.0, 0.8, 0.0),
                    ..default()
                })
                .with_children(|pivot| {
                    pivot.spawn(Camera3dBundle {
                        camera: Camera {
                            order: 1,
                            hdr: true,
                            ..default()
                        },
                        transform: Transform::from_xyz(0.0, 0.0, 3.0),
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
