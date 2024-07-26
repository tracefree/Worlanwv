use core::time;
use std::f32::consts::PI;

use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::{
    plugin::RapierContext,
    prelude::{ColliderDisabled, QueryFilter},
};

use crate::{
    game::audio::sfx::GroundMaterial,
    screen::{PlayState, Screen},
    AppSet,
};

use super::{
    assets::SfxKey,
    audio::sfx::PlaySfx,
    spawn::{
        level::{SkyMaterial, Sun, SunPivot, Terrain, WaterMaterial},
        player::{Player, PlayerCamera},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(CurrentCycle(Cycle::One))
        .insert_resource(DayProgress(0.0));
    app.observe(on_cycle_changed);
    app.observe(cast_ground_ray);
    app.add_systems(
        Update,
        (animate_sun, animate_water)
            .in_set(AppSet::TickTimers)
            .run_if(in_state(PlayState::InGame)),
    );
    app.add_systems(
        FixedUpdate,
        (
            reenable_colliders,
            disable_intersecting_colliders,
            check_for_interactables,
        )
            .run_if(in_state(PlayState::InGame)),
    );
    app.add_systems(OnEnter(PlayState::InMenu), animate_sun);
}

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub enum Cycle {
    One,
    Two,
    Three,
}

impl Cycle {
    pub fn next(self: Self) -> Self {
        match self {
            Cycle::One => Cycle::Two,
            Cycle::Two => Cycle::Three,
            Cycle::Three => Cycle::One,
        }
    }
}

#[derive(Resource)]
pub struct CurrentCycle(pub Cycle);

#[derive(Event)]
pub struct CycleChanged(pub Cycle);

#[derive(Event)]
pub struct Footstep;

#[derive(Resource)]
pub struct DayProgress(f32);

fn on_cycle_changed(
    trigger: Trigger<CycleChanged>,
    mut current_cycle: ResMut<CurrentCycle>,
    mut colliders: Query<(&mut Transform, &Cycle)>,
    mut commands: Commands,
) {
    current_cycle.0 = trigger.event().0;
    for (mut transform, cycle) in colliders.iter_mut() {
        if *cycle != current_cycle.0 {
            let depth_modifier = match *cycle {
                Cycle::One => 1.0,
                Cycle::Two => 2.0,
                Cycle::Three => 3.0,
            };
            transform.translation.y = -100.0 * depth_modifier;
        } else {
            transform.translation.y = 0.0;
        }
    }
    commands.trigger(PlaySfx::Key(SfxKey::CycleChange));
}

fn animate_sun(
    mut pivot: Query<&mut Transform, With<SunPivot>>,
    mut sun: Query<&mut DirectionalLight, With<Sun>>,
    mut day_progress: ResMut<DayProgress>,
    mut environment: Query<&mut EnvironmentMapLight, With<Camera>>,
    sky_materials: Query<&Handle<SkyMaterial>>,
    mut mats: ResMut<Assets<SkyMaterial>>,
    current_cycle: Res<CurrentCycle>,
    time: Res<Time>,
    mut commands: Commands,
) {
    day_progress.0 += time.delta_seconds() / 30.0;
    if day_progress.0 >= 1.0 {
        day_progress.0 -= 1.0;
        let next_cycle = current_cycle.0.next();
        commands.trigger(CycleChanged(next_cycle));
    }

    let mut pivot = pivot.single_mut();
    let angle = PI * day_progress.0;
    pivot.rotation = Quat::from_euler(EulerRot::YXZ, 0.0, -PI / 4.0, angle);
    let brightness_factor = (day_progress.0 * PI).sin();
    sun.single_mut().illuminance = light_consts::lux::AMBIENT_DAYLIGHT * brightness_factor;
    environment.single_mut().intensity =
        0.0.lerp(light_consts::lux::DARK_OVERCAST_DAY, brightness_factor);

    for material in sky_materials.iter() {
        if let Some(sky) = mats.get_mut(material) {
            sky.time = brightness_factor;
        }
    }
}

fn animate_water(
    water_materials: Query<&Handle<WaterMaterial>>,
    mut mats: ResMut<Assets<WaterMaterial>>,
    time: Res<Time>,
) {
    for material in water_materials.iter() {
        if let Some(water) = mats.get_mut(material) {
            water.time = time.elapsed_seconds();
        }
    }
}

fn cast_ground_ray(
    _trigger: Trigger<Footstep>,
    rapier_context: Res<RapierContext>,
    player: Query<(Entity, &Transform), With<Player>>,
    terrain: Query<Entity, With<Terrain>>,
    mut commands: Commands,
) {
    let (player, transform) = player.single();
    rapier_context.intersections_with_ray(
        transform.translation + Vec3::NEG_Y * 0.75,
        Vec3::NEG_Y,
        0.5,
        false,
        QueryFilter::default(),
        |entity, _| {
            if entity == player {
                // Keep searching
                return true;
            } else if entity == terrain.single() {
                // Hit grassy terrain
                commands.trigger(PlaySfx::RandomStep(GroundMaterial::Grass));
                return false;
            }
            // Treat every other surface as solid
            commands.trigger(PlaySfx::RandomStep(GroundMaterial::Solid));
            return false;
        },
    )
}

fn check_for_interactables(
    rapier_context: Res<RapierContext>,
    player: Query<Entity, With<Player>>,
    camera: Query<(&Transform, &GlobalTransform), With<PlayerCamera>>,
) {
    let (local, global) = camera.single();
    rapier_context.intersections_with_ray(
        global.translation(),
        -local.local_z().as_vec3(),
        1.0,
        false,
        QueryFilter::default(),
        |entity, _| {
            if entity == player.single() {
                // Keep searching
                return true;
            }
            println!("Looking at {:?}", entity);
            return false;
        },
    )
}

fn disable_intersecting_colliders(
    rapier_context: Res<RapierContext>,
    mut player: Query<Entity, With<Player>>,
    mut commands: Commands,
) {
    let player = player.single_mut();
    for (_, entity, intersecting) in rapier_context.intersection_pairs_with(player) {
        if intersecting {
            commands.entity(entity).insert(ColliderDisabled);
        }
    }
}

// TODO: Custom Schedule instead of local timer?
fn reenable_colliders(
    mut commands: Commands,
    disabled_colliders: Query<Entity, With<ColliderDisabled>>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer -= time.delta_seconds();
    if *timer <= 0.0 {
        *timer = 0.5;
        for collider in disabled_colliders.iter() {
            commands.entity(collider).remove::<ColliderDisabled>();
        }
    }
}
