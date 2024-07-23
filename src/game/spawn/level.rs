//! Spawn the main level by triggering other observers.

use std::f32::consts::PI;

use bevy::{color::palettes::tailwind, prelude::*, render::view::NoFrustumCulling};
use bevy_rapier3d::{
    dynamics::RigidBody,
    geometry::{Collider, ComputedColliderShape},
    plugin::RapierContext,
    prelude::{ActiveCollisionTypes, GravityScale, Velocity},
};

use crate::game::logic::Cycle;

use super::player::{Player, SpawnPlayer};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
    // TODO: Do this once after loading geometry, don't check every frame
    app.add_systems(Update, spawn_colliders);
    app.add_systems(FixedUpdate, prevent_collider_overlap);
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

#[derive(Component)]
struct Terrain;

#[derive(Component)]
struct StuckInGeometry(Vec3);

#[derive(Component)]
pub struct SunPivot;

#[derive(Component)]
pub struct Sun;

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
        material: materials.add(StandardMaterial {
            base_color: Color::from(tailwind::BLUE_600),
            cull_mode: None,
            ..default()
        }),
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
    // Sun
    commands
        .spawn(SpatialBundle::default())
        .insert(SunPivot)
        .with_children(|pivot| {
            pivot
                .spawn(Sun)
                .insert(DirectionalLightBundle {
                    directional_light: DirectionalLight {
                        illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
                        shadows_enabled: true,
                        ..default()
                    },
                    transform: Transform::from_rotation(Quat::from_rotation_y(-PI / 2.0)),
                    ..default()
                })
                .insert(MaterialMeshBundle {
                    mesh: meshes.add(Sphere::new(50.0)),
                    material: materials.add(StandardMaterial {
                        base_color: Color::from(tailwind::YELLOW_950),
                        emissive: LinearRgba::new(100.0, 80.0, 10.0, 1.0),
                        ..default()
                    }),
                    transform: Transform {
                        translation: Vec3::new(1000.0, 0.0, 0.0),
                        rotation: Quat::from_rotation_y(PI / 2.0),
                        ..default()
                    },
                    ..default()
                })
                .insert(NoFrustumCulling);
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

fn prevent_collider_overlap(
    rapier_context: Res<RapierContext>,
    mut player: Query<
        (
            Entity,
            &mut Transform,
            &mut Velocity,
            Option<&StuckInGeometry>,
        ),
        With<Player>,
    >,
    terrain: Query<Entity, With<Terrain>>,
    mut commands: Commands,
) {
    /* Find the intersection pair, if it exists, between two colliders. */
    let (player, mut transform, mut velocity, stuck) = player.single_mut();
    for (_, collider, intersecting) in rapier_context.intersection_pairs_with(player) {
        if intersecting {
            continue;
            if collider == terrain.single() {
                transform.translation += Vec3::new(0.0, 0.1, 0.0);
            } else {
                let back = transform.local_z() * 0.1;
                transform.translation += back;
            }
            velocity.linvel = Vec3::ZERO;
        }
    }

    let mut overlap = false;
    for contact_pair in rapier_context.contact_pairs_with(player) {
        if let Some((manifold, contact)) = contact_pair.find_deepest_contact() {
            overlap = true;
            if contact_pair.collider2() == terrain.single() {
                //    rapier_context.move_shape(Vec3::Y * 0.3, contact_pair.collider1(), transform.translation(), transform.ro, shape_mass, options, filter, events)
                transform.translation.y += contact.dist();
                velocity.linvel = Vec3::ZERO;
                continue;
            }

            if let Some(normal) = stuck {
                transform.translation += normal.0 * contact.dist();
                velocity.linvel = Vec3::ZERO;
            } else {
                println!("{:?}", manifold.normal());
                let push_vector = Vec3::new(
                    -manifold.normal().x,
                    manifold.normal().y.abs(),
                    -manifold.normal().z,
                );
                commands.entity(player).insert(StuckInGeometry(push_vector));
            }
        }
    }
    if stuck.is_some() && !overlap {
        println!("No oberlap");
        velocity.linvel = Vec3::ZERO;
        commands.entity(player).remove::<StuckInGeometry>();
    }
}
