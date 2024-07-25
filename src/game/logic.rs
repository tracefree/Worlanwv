use core::time;
use std::f32::consts::PI;

use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::{plugin::RapierContext, prelude::QueryFilter};

use crate::{
    game::audio::sfx::GroundMaterial,
    screen::{PlayState, Screen},
    AppSet,
};

use super::{
    assets::SfxKey,
    audio::sfx::PlaySfx,
    spawn::{
        level::{SkyMaterial, Sun, SunPivot, Terrain},
        player::Player,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(CurrentCycle(Cycle::One))
        .insert_resource(DayProgress(0.0));
    app.observe(on_cycle_changed);
    app.observe(cast_ground_ray);
    app.add_systems(
        Update,
        animate_sun
            .run_if(in_state(PlayState::InGame))
            .in_set(AppSet::Update),
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

fn prevent_collider_overlap(
    rapier_context: Res<RapierContext>,
    mut player: Query<Entity, With<Player>>,
    mut commands: Commands,
) {
    /* Find the intersection pair, if it exists, between two colliders. */
    let player = player.single_mut();
    for (_, _, intersecting) in rapier_context.intersection_pairs_with(player) {
        if intersecting {
            continue;
            /*
            if collider == terrain.single() {
                transform.translation += Vec3::new(0.0, 0.1, 0.0);
            } else {
                let back = transform.local_z() * 0.1;
                transform.translation += back;
            }
            velocity.linvel = Vec3::ZERO;
            */
        }
    }
    /*
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
                //   commands.entity(player).insert(StuckInGeometry(push_vector));
            }
        }
    }
    if stuck.is_some() && !overlap {
        println!("No oberlap");
        velocity.linvel = Vec3::ZERO;
        //   commands.entity(player).remove::<StuckInGeometry>();
    }
    */
}
