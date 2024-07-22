//! Spawn the main level by triggering other observers.

use std::f32::consts::PI;

use bevy::{color::palettes::tailwind, math::VectorSpace, prelude::*};
use bevy_rapier3d::{
    dynamics::RigidBody,
    geometry::{Collider, ComputedColliderShape},
    plugin::RapierContext,
    prelude::{
        ActiveCollisionTypes, ActiveEvents, CollisionEvent, ContactForceEvent, GravityScale,
        KinematicCharacterControllerOutput, Velocity,
    },
};

use crate::game::logic::Cycle;

use super::player::{Player, SpawnPlayer};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
    // TODO: Do this once after loading geometry, don't check every frame
    app.add_systems(Update, spawn_colliders);
    app.add_systems(Update, (display_intersection_info));
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

#[derive(Component)]
struct Terrain;

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
        .insert(Cycle::One);

    // Cycle 2
    commands
        .spawn(SceneBundle {
            scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/cycle_2.glb")),
            transform: Transform::from_xyz(0.0, -200.0, 0.0),
            ..default()
        })
        .insert(Cycle::Two);

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
) {
    for (entity, name, mesh) in scene_objects.iter() {
        if !name.as_str().contains("_col") {
            continue;
        }
        let mesh = meshes.get(mesh).unwrap();
        commands
            .entity(entity)
            .insert(RigidBody::Fixed)
            .insert(GravityScale(0.0))
            .insert(ActiveCollisionTypes::all())
            .insert(Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh).unwrap());

        if name.as_str().contains("terrain") {
            commands.entity(entity).insert(Terrain);
        }
    }
}

/* A system that displays the events. */
fn display_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_force_events: EventReader<ContactForceEvent>,
) {
    for collision_event in collision_events.read() {
        println!("Received collision event: {:?}", collision_event);
    }

    for contact_force_event in contact_force_events.read() {
        println!("Received contact force event: {:?}", contact_force_event);
    }
}

fn display_intersection_info(
    rapier_context: Res<RapierContext>,
    mut player: Query<(Entity, &mut Transform, &mut Velocity), With<Player>>,
    terrain: Query<Entity, With<Terrain>>,
) {
    /* Find the intersection pair, if it exists, between two colliders. */
    let (player, mut transform, mut velocity) = player.single_mut();
    for (_, collider, intersecting) in rapier_context.intersection_pairs_with(player) {
        if intersecting {
            if collider == terrain.single() {
                transform.translation += Vec3::new(0.0, 0.1, 0.0);
            } else {
                let back = transform.local_z() * 0.1;
                transform.translation += back;
            }
            velocity.linvel = Vec3::ZERO;
        }
    }
}

/* Read the character controller collisions stored in the character controller’s output. */
fn read_character_controller_collisions(
    mut character_controller_outputs: Query<&mut KinematicCharacterControllerOutput>,
) {
    for mut output in character_controller_outputs.iter_mut() {
        for collision in &output.collisions {
            println!("{}", collision.translation_remaining);
            // Do something with that collision information.
        }
    }
}
