//! Spawn the player.

use std::f32::consts::PI;

use bevy::{
    core_pipeline::{
        bloom::BloomSettings, experimental::taa::TemporalAntiAliasBundle, prepass::DepthPrepass,
    },
    pbr::{ScreenSpaceAmbientOcclusionBundle, ScreenSpaceAmbientOcclusionSettings},
    prelude::*,
};
use bevy_rapier3d::{
    control::KinematicCharacterController,
    geometry::Collider,
    prelude::{
        CharacterAutostep, CharacterLength, CollisionGroups, Group,
        KinematicCharacterControllerOutput, RigidBody, Sensor,
    },
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

#[derive(Component)]
pub struct PlayerCamera;

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
                transform: Transform {
                    translation: Vec3::new(5.52, 4.4, -33.66),
                    ..default()
                },
                ..default()
            },
            RigidBody::KinematicPositionBased,
            Sensor,
            KinematicCharacterController {
                custom_mass: Some(5.0),
                up: Vec3::Y,
                offset: CharacterLength::Absolute(0.01),
                slide: true,
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Relative(0.3),
                    min_width: CharacterLength::Relative(0.5),
                    include_dynamic_bodies: false,
                }),
                max_slope_climb_angle: 45.0_f32.to_radians(),
                min_slope_slide_angle: 45.0_f32.to_radians(),
                apply_impulse_to_dynamic_bodies: true,
                snap_to_ground: Some(CharacterLength::Absolute(0.2)),
                ..default()
            },
            KinematicCharacterControllerOutput::default(),
            CollisionGroups::new(Group::GROUP_1, Group::all() & !Group::GROUP_2),
            Collider::capsule_y(0.5, 0.3),
        ))
        .with_children(|player| {
            player
                .spawn(CameraPivot)
                .insert(SpatialBundle {
                    transform: Transform {
                        translation: Vec3::new(0.0, 0.8, 0.0),
                        rotation: Quat::from_euler(EulerRot::YXZ, PI - 0.2, 0.2, 0.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|pivot| {
                    pivot
                        .spawn(PlayerCamera)
                        .insert(Camera3dBundle {
                            camera: Camera {
                                order: 1,
                                hdr: true,
                                ..default()
                            },
                            transform: Transform::default(),
                            projection: PerspectiveProjection {
                                fov: 70.0_f32.to_radians(),
                                near: 0.1,
                                ..default()
                            }
                            .into(),
                            ..default()
                        })
                        .insert(DepthPrepass)
                        .insert(ScreenSpaceAmbientOcclusionBundle {
                            settings: ScreenSpaceAmbientOcclusionSettings {
                                quality_level:
                                    bevy::pbr::ScreenSpaceAmbientOcclusionQualityLevel::Medium,
                            },
                            ..default()
                        })
                        .insert(EnvironmentMapLight {
                            diffuse_map: asset_server.load("textures/cubemap.ktx2"),
                            specular_map: asset_server.load("textures/cubemap.ktx2"),
                            intensity: light_consts::lux::OVERCAST_DAY,
                        })
                        .insert(TemporalAntiAliasBundle::default())
                        .insert(BloomSettings::default());
                });
        });
}
