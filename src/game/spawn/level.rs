//! Spawn the main level by triggering other observers.

use std::f32::consts::PI;

use bevy::{color::palettes::tailwind, prelude::*};
use bevy_rapier3d::geometry::Collider;

use super::player::SpawnPlayer;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level).insert_resource(AmbientLight {
        brightness: 100.0,
        ..default()
    });
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

fn spawn_level(
    _trigger: Trigger<SpawnLevel>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    commands.trigger(SpawnPlayer);
    commands
        .spawn(MaterialMeshBundle {
            material: materials.add(Color::from(tailwind::BLUE_50)),
            mesh: meshes.add(Cuboid::new(100.0, 100.0, 100.0)),
            transform: Transform::from_xyz(0.0, -50.0, 0.0),
            ..default()
        })
        .insert(Collider::cuboid(50.0, 50.0, 50.0));
    commands
        .spawn(MaterialMeshBundle {
            material: materials.add(Color::BLACK),
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            transform: Transform::from_xyz(0.0, 0.0, -3.0),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 0.5, 0.5));

    // Lights
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 3.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.0),
            ..default()
        },
        ..default()
    });
}
