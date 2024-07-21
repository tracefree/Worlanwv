//! Spawn the main level by triggering other observers.

use bevy::{color::palettes::tailwind, pbr::PbrPlugin, prelude::*};

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
    commands.spawn(MaterialMeshBundle {
        material: materials.add(Color::from(tailwind::BLUE_50)),
        mesh: meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(1000.0))),
        transform: Transform::from_xyz(0.0, 0.0, -3.0),
        ..default()
    });
    commands.spawn(MaterialMeshBundle {
        material: materials.add(Color::from(tailwind::BLUE_50)),
        mesh: meshes.add(Cuboid::new(0.5, 0.5, 0.5)),
        transform: Transform::from_xyz(0.0, 2.0, -3.0),
        ..default()
    });
}
