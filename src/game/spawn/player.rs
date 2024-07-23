//! Spawn the player.

use bevy::{
    core_pipeline::experimental::taa::TemporalAntiAliasBundle,
    pbr::{ScreenSpaceAmbientOcclusionBundle, ScreenSpaceAmbientOcclusionSettings},
    prelude::*,
};
use bevy_rapier3d::{
    control::KinematicCharacterController,
    dynamics::{Ccd, RigidBody, Velocity},
    geometry::Collider,
    prelude::{ActiveCollisionTypes, ActiveEvents, Sensor},
};

use crate::{game::movement::MovementController, screen::Screen};

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

fn spawn_player(
    _trigger: Trigger<SpawnPlayer>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn((
            Name::new("Player"),
            Player,
            MovementController::default(),
            StateScoped(Screen::Playing),
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 10.0, 0.0),
                ..default()
            },
            RigidBody::KinematicPositionBased,
            //    Sensor,
            //    Ccd::enabled(),
            Velocity {
                linvel: Vec3::ZERO,
                angvel: Vec3::ZERO,
            },
            KinematicCharacterController {
                max_slope_climb_angle: 45.0_f32.to_radians(),
                slide: true,
                min_slope_slide_angle: 30.0_f32.to_radians(),
                ..default()
            },
            Collider::cuboid(0.3, 0.8, 0.3),
        ))
        .with_children(|player| {
            player
                .spawn(CameraPivot)
                .insert(SpatialBundle {
                    transform: Transform::from_xyz(0.0, 0.8, 0.0),
                    ..default()
                })
                .with_children(|pivot| {
                    pivot
                        .spawn(Camera3dBundle {
                            camera: Camera {
                                order: 1,
                                hdr: true,
                                ..default()
                            },
                            transform: Transform::from_xyz(0.0, 0.0, 0.0),
                            projection: PerspectiveProjection {
                                fov: 70.0_f32.to_radians(),
                                ..default()
                            }
                            .into(),
                            ..default()
                        })
                        .insert(ScreenSpaceAmbientOcclusionBundle {
                            settings: ScreenSpaceAmbientOcclusionSettings {
                                quality_level:
                                    bevy::pbr::ScreenSpaceAmbientOcclusionQualityLevel::High,
                            },
                            ..default()
                        })
                        /*         .insert(EnvironmentMapLight {
                            diffuse_map: asset_server.load("textures/cubemap.ktx2"),
                            specular_map: asset_server.load("textures/cubemap.ktx2"),
                            intensity: light_consts::lux::OVERCAST_DAY,
                        }) */
                        .insert(TemporalAntiAliasBundle::default());
                });
        });
}
