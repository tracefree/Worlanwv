use core::time;
use std::f32::consts::PI;

use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use crate::{screen::PlayState, AppSet};

use super::{
    assets::SfxKey,
    audio::sfx::PlaySfx,
    spawn::level::{SkyMaterial, Sun, SunPivot},
};

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(CurrentCycle(Cycle::One))
        .insert_resource(DayProgress(0.0));
    app.observe(on_cycle_changed);
    app.add_systems(Update, handle_input.in_set(AppSet::RecordInput));
    app.add_systems(
        Update,
        animate_sun
            .run_if(in_state(PlayState::InGame))
            .in_set(AppSet::Update),
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub enum Cycle {
    One,
    Two,
    Three,
}

impl Cycle {
    fn next(self: Self) -> Self {
        match self {
            Cycle::One => Cycle::Two,
            Cycle::Two => Cycle::Three,
            Cycle::Three => Cycle::One,
        }
    }
}

#[derive(Resource)]
pub struct CurrentCycle(Cycle);

#[derive(Event)]
pub struct CycleChanged(pub Cycle);

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

fn handle_input(
    input: Res<ButtonInput<KeyCode>>,
    current_cycle: Res<CurrentCycle>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut commands: Commands,
) {
    if input.just_pressed(KeyCode::ArrowRight) {
        let next_cycle = current_cycle.0.next();
        commands.trigger(CycleChanged(next_cycle));
    }
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
