//! Spawn the main level by triggering other observers.

use std::{cmp::Ordering, f32::consts::PI};

use bevy::{
    color::palettes::tailwind,
    pbr::{ExtendedMaterial, MaterialExtension, NotShadowCaster},
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderRef},
        view::NoFrustumCulling,
    },
};
use bevy_rapier3d::{
    dynamics::RigidBody,
    geometry::{Collider, ComputedColliderShape},
    prelude::{ActiveCollisionTypes, CollisionGroups, GravityScale, Group},
};

use crate::game::logic::{on_boat_used, on_hourglass_taken, Cycle, Interactable};

use super::player::SpawnPlayer;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level).observe(spawn_interactable);
    // TODO: Do this once after loading geometry, don't check every frame
    app.add_plugins(MaterialPlugin::<SkyMaterial> {
        prepass_enabled: false,
        shadows_enabled: false,
        ..default()
    });
    app.add_plugins(MaterialPlugin::<
        ExtendedMaterial<StandardMaterial, WaterMaterial>,
    >::default());
    app.add_systems(Update, spawn_colliders);
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

pub enum InteractableScene {
    Boat,
    Hourglass,
}

#[derive(Event)]
pub struct SpawnInteractable(pub InteractableScene, pub Entity);

#[derive(Component)]
pub struct Terrain;

#[derive(Component)]
pub struct SunPivot;

#[derive(Component)]
pub struct Sun;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct SkyMaterial {
    #[uniform(0)]
    pub time: f32,
    #[cfg(feature = "webgl2")]
    #[uniform(1)]
    pub _webgl2_padding: Vec3,
}

impl Material for SkyMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/sky_shader_vert.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/sky_shader_frag.wgsl".into()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WaterMaterial {
    #[uniform(100)]
    pub time: f32,
}

impl MaterialExtension for WaterMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/water_shader.wgsl".into()
    }
}

fn spawn_level(
    _trigger: Trigger<SpawnLevel>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sky_materials: ResMut<Assets<SkyMaterial>>,
    mut water_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, WaterMaterial>>>,
    asset_server: Res<AssetServer>,
) {
    commands.trigger(SpawnPlayer);

    // Ocean
    commands.spawn(MaterialMeshBundle {
        material: water_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                alpha_mode: AlphaMode::Blend,
                ..default()
            },
            extension: WaterMaterial { time: 0.0 },
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
        .insert(Cycle::Two)
        .with_children(|scene| {
            scene
                .spawn(SceneBundle {
                    scene: asset_server
                        .load(GltfAssetLabel::Scene(0).from_asset("models/hourglass.glb")),
                    transform: Transform::from_xyz(1.0, 8.7, 1.0),
                    ..default()
                })
                .insert(Interactable::new("E: Take".into()))
                .insert(Collider::ball(0.15))
                .insert(CollisionGroups::new(Group::GROUP_2, Group::ALL))
                .observe(on_hourglass_taken);
        });

    // Comet
    commands.spawn(SceneBundle {
        scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/comet.glb")),
        transform: Transform {
            translation: Vec3::new(0.0, 200.0, 400.0),
            rotation: Quat::from_euler(EulerRot::YXZ, PI, 0.0, -PI / 8.0),
            scale: Vec3::new(2.0, 2.0, 2.0),
        },
        ..default()
    });

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

    // Skybox
    commands
        .spawn(MaterialMeshBundle {
            mesh: meshes.add(Cuboid::default()),
            material: sky_materials.add(SkyMaterial::default()),
            ..default()
        })
        .insert(NoFrustumCulling)
        .insert(NotShadowCaster);
}

fn spawn_colliders(
    mut commands: Commands,
    q_children: Query<&Children>,
    mut interactables: Query<(Entity, &mut Interactable)>,
    scene_objects: Query<(Entity, &Name, Option<&Handle<Mesh>>), Added<Name>>,
    meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, name, mesh) in scene_objects.iter() {
        if name.as_str().contains("SpawnBoat") {
            commands.trigger(SpawnInteractable(InteractableScene::Boat, entity));
        } else if name.as_str().contains("SpawnHourglass") {
            commands.trigger(SpawnInteractable(InteractableScene::Hourglass, entity));
        }
        if name.as_str().contains("highlight") {
            for (scene_entity, mut interactable) in interactables.iter_mut() {
                for descendent in q_children.iter_descendants(scene_entity) {
                    if descendent != entity {
                        continue;
                    }
                    interactable.highlight_mesh = Some(entity);
                }
            }
            continue;
        }
        if !name.as_str().contains("_col") {
            continue;
        }
        if let Some(mesh) = mesh {
            let mesh = meshes.get(mesh).unwrap();
            commands
                .entity(entity)
                .insert(RigidBody::Fixed)
                .insert(GravityScale(0.0))
                .insert(ActiveCollisionTypes::all());

            if name.as_str().contains("terrain") {
                let (heights, num_rows, num_cols, scale) = heightfield_from_mesh(mesh);
                commands
                    .entity(entity)
                    .insert(Terrain)
                    .insert(Collider::heightfield(heights, num_rows, num_cols, scale));
            } else {
                commands.entity(entity).insert(
                    Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh).unwrap(),
                );
            }
        }
    }
}

fn heightfield_from_mesh(mesh: &Mesh) -> (Vec<f32>, usize, usize, Vec3) {
    let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
    let num_cuts = (positions.len() as f32).sqrt() as usize;
    let mut heights: Vec<f32> = vec![];
    let mut sorted_positions = positions.as_float3().unwrap().to_vec();
    sorted_positions.sort_by(|pos1, pos2| -> Ordering {
        if pos1[0] < pos2[0] {
            Ordering::Less
        } else if pos1[0] > pos2[0] {
            Ordering::Greater
        } else if pos1[2] < pos2[2] {
            Ordering::Less
        } else if pos1[2] > pos2[2] {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });

    for [_, y, _] in sorted_positions {
        heights.push(y);
    }

    (heights, num_cuts, num_cuts, Vec3::new(100.0, 1.0, 100.0))
}

fn spawn_interactable(
    trigger: Trigger<SpawnInteractable>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    match trigger.event().0 {
        InteractableScene::Boat => {
            commands.entity(trigger.event().1).with_children(|parent| {
                parent
                    .spawn(SceneBundle {
                        scene: asset_server
                            .load(GltfAssetLabel::Scene(0).from_asset("models/boat.glb")),
                        ..default()
                    })
                    .insert(Interactable::new("E: Use".into()))
                    .insert(Collider::cuboid(3.5, 1.5, 2.5))
                    .insert(CollisionGroups::new(
                        Group::GROUP_2,
                        Group::ALL & !Group::GROUP_1,
                    ))
                    .observe(on_boat_used);
            });
        }
        InteractableScene::Hourglass => {
            todo!();
        }
    }
}
