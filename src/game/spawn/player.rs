//! Spawn the player.

use bevy::{
    core_pipeline::{
        bloom::{BloomPlugin, BloomSettings},
        experimental::taa::TemporalAntiAliasBundle,
        prepass::DepthPrepass,
    },
    pbr::{ScreenSpaceAmbientOcclusionBundle, ScreenSpaceAmbientOcclusionSettings},
    prelude::*,
    render::view::NoFrustumCulling,
};
use bevy_rapier3d::{
    control::KinematicCharacterController,
    dynamics::{Ccd, RigidBody, Velocity},
    geometry::Collider,
    prelude::{
        ActiveCollisionTypes, ActiveEvents, CharacterAutostep, CharacterLength,
        KinematicCharacterControllerOutput, Sensor,
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
                transform: Transform::from_xyz(0.0, 10.0, 0.0),
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
                min_slope_slide_angle: 30.0_f32.to_radians(),
                apply_impulse_to_dynamic_bodies: true,
                snap_to_ground: Some(CharacterLength::Absolute(0.2)),
                ..default()
            },
            KinematicCharacterControllerOutput::default(),
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
                    pivot
                        .spawn(PlayerCamera)
                        .insert(Camera3dBundle {
                            camera: Camera {
                                order: 1,
                                hdr: true,
                                ..default()
                            },
                            transform: Transform::from_xyz(0.0, 0.0, 0.0),
                            projection: PerspectiveProjection {
                                fov: 70.0_f32.to_radians(),
                                near: 0.25,
                                ..default()
                            }
                            .into(),
                            ..default()
                        })
                        .insert(DepthPrepass)
                        .insert(ScreenSpaceAmbientOcclusionBundle {
                            settings: ScreenSpaceAmbientOcclusionSettings {
                                quality_level:
                                    bevy::pbr::ScreenSpaceAmbientOcclusionQualityLevel::High,
                            },
                            ..default()
                        })
                        .insert(EnvironmentMapLight {
                            diffuse_map: asset_server.load("textures/cubemap.ktx2"),
                            specular_map: asset_server.load("textures/cubemap.ktx2"),
                            intensity: light_consts::lux::OVERCAST_DAY,
                        })
                        .insert(BloomSettings::default())
                        .insert(TemporalAntiAliasBundle::default());
                });
        });
}
