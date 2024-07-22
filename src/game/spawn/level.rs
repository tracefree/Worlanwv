//! Spawn the main level by triggering other observers.

use std::{any::Any, f32::consts::PI};

use bevy::{color::palettes::tailwind, prelude::*};
use bevy_rapier3d::{
    dynamics::RigidBody,
    geometry::{Collider, ComputedColliderShape},
    prelude::ColliderDisabled,
};

use super::player::SpawnPlayer;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
    app.observe(on_cycle_changed);
    app.insert_resource(CurrentCycle(Cycle::One));
    // TODO: Do this once after loading geometry, don't check every frame
    app.add_systems(Update, spawn_colliders);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cycle {
    One,
    Two,
    Three,
}

#[derive(Resource)]
pub struct CurrentCycle(Cycle);

#[derive(Component)]
pub struct BelongsToCycle(Cycle);

#[derive(Event)]
pub struct CycleChanged(pub Cycle);

#[derive(Event, Debug)]
pub struct SpawnLevel;

fn spawn_level(
    _trigger: Trigger<SpawnLevel>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    commands.trigger(SpawnPlayer);

    // Ocean
    commands.spawn(PbrBundle {
        material: materials.add(Color::from(tailwind::BLUE_950)),
        mesh: meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(1000.0))),
        ..default()
    });

    // Terrain
    commands.spawn(SceneBundle {
        scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/terrain.glb")),
        ..default()
    });

    // Cycle 1
    commands
        .spawn(SceneBundle {
            scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/cycle_1.glb")),
            ..default()
        })
        .insert(BelongsToCycle(Cycle::One));

    // Cycle 2
    commands
        .spawn(SceneBundle {
            scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/cycle_2.glb")),
            ..default()
        })
        .insert(BelongsToCycle(Cycle::Two))
        .insert(Visibility::Hidden);

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

fn spawn_colliders(
    mut commands: Commands,
    scene_objects: Query<(Entity, &Name, &Handle<Mesh>), Added<Name>>,
    meshes: ResMut<Assets<Mesh>>,
    cycle: Res<CurrentCycle>,
) {
    for (entity, name, mesh) in scene_objects.iter() {
        if !name.as_str().contains("_col") {
            continue;
        }
        let mesh = meshes.get(mesh).unwrap();
        commands
            .entity(entity)
            .insert(BelongsToCycle(cycle.0))
            .insert(RigidBody::Fixed)
            .insert(Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh).unwrap());
    }
}

fn on_cycle_changed(
    trigger: Trigger<CycleChanged>,
    mut current_cycle: ResMut<CurrentCycle>,
    colliders: Query<(Entity, &BelongsToCycle)>,
    mut commands: Commands,
) {
    println!("CycleChanged");
    current_cycle.0 = trigger.event().0;
    for (entity, cycle) in colliders.iter() {
        if cycle.0 != current_cycle.0 {
            commands.entity(entity).insert(ColliderDisabled);
            commands.entity(entity).insert(Visibility::Hidden);
        } else {
            println!("{}: {:?}", commands.entity(entity).id(), cycle.0);
            commands.entity(entity).remove::<ColliderDisabled>();
            commands.entity(entity).remove::<Visibility>();
        }
    }
}
