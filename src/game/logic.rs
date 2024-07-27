use core::time;
use std::{f32::consts::PI, time::Duration};

use bevy::{
    pbr::ExtendedMaterial,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::{
    plugin::RapierContext,
    prelude::{ColliderDisabled, CollisionGroups, Group, QueryFilter, RigidBodyDisabled},
};

use crate::{
    game::audio::sfx::GroundMaterial,
    screen::{PlayState, Screen},
    AppSet,
};

use super::{
    animation::Animations,
    assets::SfxKey,
    audio::sfx::PlaySfx,
    spawn::{
        level::{SkyMaterial, Sun, SunPivot, Terrain, WaterMaterial},
        player::{Player, PlayerCamera},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(CurrentCycle(Cycle::One))
        .insert_resource(DayProgress(0.0))
        .insert_resource(CurrentHighlighted(None))
        .insert_resource(Inventory::default());
    app.observe(on_cycle_changed);
    app.observe(cast_ground_ray);
    app.register_type::<Interactable>();
    app.add_systems(
        Update,
        (animate_sun, animate_water, handle_interaction)
            .in_set(AppSet::TickTimers)
            .run_if(in_state(PlayState::InGame)),
    );
    app.add_systems(
        FixedUpdate,
        (
            reenable_colliders,
            disable_intersecting_colliders,
            check_for_interactables,
            update_highlight_mesh.after(check_for_interactables),
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

#[derive(Event)]
pub struct HighlightChanged {
    pub from: Option<Entity>,
    pub to: Option<Entity>,
}

#[derive(Resource)]
pub struct CurrentHighlighted(pub Option<Entity>);

#[derive(Component, Reflect)]
pub struct Interactable {
    pub highlight_mesh: Option<Entity>,
    pub text: String,
}

#[derive(Resource, Default)]
pub struct Inventory {
    hourglass: bool,
    sapling: bool,
}

impl Interactable {
    pub fn new(text: String) -> Self {
        Self {
            highlight_mesh: None,
            text,
        }
    }
}

#[derive(Component)]
pub struct PromptText;

#[derive(Event)]
pub struct Interacted;

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
    input: Res<ButtonInput<KeyCode>>,
    inventory: Res<Inventory>,
    mut commands: Commands,
) {
    let time_modifier = match inventory.hourglass && input.pressed(KeyCode::KeyQ) {
        false => 1.0,
        true => 30.0,
    };
    day_progress.0 += time.delta_seconds() * time_modifier / 60.0;
    if day_progress.0 >= 1.0 {
        day_progress.0 -= 1.0;
        let next_cycle = current_cycle.0.next();
        commands.trigger(CycleChanged(next_cycle));
    }

    let mut pivot = pivot.single_mut();
    let angle = 2.0 * PI * day_progress.0;
    pivot.rotation = Quat::from_euler(EulerRot::YXZ, 0.0, -PI / 4.0, angle);
    let brightness_factor = (day_progress.0 * 2.0 * PI).sin().clamp(0.0, 1.0);
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
    mut mats: ResMut<Assets<ExtendedMaterial<StandardMaterial, WaterMaterial>>>,
    time: Res<Time>,
) {
    for material in water_materials.iter() {
        /*   if let Some(water) = mats.get_mut(material) {
        water.time = time.elapsed_seconds();
        }*/
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
    disabled_colliders: Query<Entity, (With<ColliderDisabled>, Without<Interactable>)>,
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

// --- Interactions ---

fn check_for_interactables(
    rapier_context: Res<RapierContext>,
    camera: Query<&GlobalTransform, With<PlayerCamera>>,
    mut highlighted: ResMut<CurrentHighlighted>,
    mut commands: Commands,
) {
    let (_, rotation, translation) = camera.single().to_scale_rotation_translation();
    let result = rapier_context.cast_ray(
        translation + rotation * Vec3::NEG_Z * 0.35,
        rotation * Vec3::NEG_Z,
        1.0,
        false,
        QueryFilter::from(CollisionGroups::new(
            Group::all() & !Group::GROUP_2,
            Group::GROUP_2,
        )),
    );

    if let Some((object, _)) = result {
        if highlighted.0.is_none() {
            commands.trigger(HighlightChanged {
                from: highlighted.0,
                to: Some(object),
            });
            highlighted.0 = Some(object);
        }
    } else {
        if highlighted.0.is_some() {
            commands.trigger(HighlightChanged {
                from: highlighted.0,
                to: None,
            });
        }
        highlighted.0 = None;
    }
}

fn handle_interaction(
    input: Res<ButtonInput<KeyCode>>,
    current_highlighted: Res<CurrentHighlighted>,
    interactables: Query<Entity, With<Interactable>>,
    mut commands: Commands,
) {
    for object in interactables.iter() {
        if current_highlighted.0 == Some(object) && input.just_pressed(KeyCode::KeyE) {
            commands.trigger_targets(Interacted, object);
        }
    }
}

/*
fn on_highlighted(trigger: Trigger<HighlightChanged>) {
    if let Some(previous) = trigger.event().from {
        println!("Lost highlight on {:?}", previous);
    }
    if let Some(new) = trigger.event().to {
        println!("Highlighted {:?}", new);
    }
}
*/

fn update_highlight_mesh(
    highlighted: Res<CurrentHighlighted>,
    interactables: Query<(Entity, &Interactable)>,
    mut prompt: Query<&mut Text, With<PromptText>>,
    mut commands: Commands,
) {
    for (entity, interactable) in interactables.iter() {
        if let Some(highlight_mesh) = interactable.highlight_mesh {
            let mut text = prompt.single_mut();
            if highlighted.0 == Some(entity) {
                commands.entity(highlight_mesh).insert(Visibility::Visible);
                text.sections[0].value = interactable.text.clone();
            } else {
                commands.entity(highlight_mesh).insert(Visibility::Hidden);
                text.sections[0].value = "".into();
            }
        }
    }
}

// Content specific logic

pub fn on_hourglass_taken(
    trigger: Trigger<Interacted>,
    mut inventory: ResMut<Inventory>,
    mut prompt: Query<&mut Text, With<PromptText>>,
    mut commands: Commands,
) {
    inventory.hourglass = true;
    prompt.single_mut().sections[0].value = "Hold Q: Fast-forward time".into();
    commands.trigger(PlaySfx::Key(SfxKey::PickupHourglass));
    commands.entity(trigger.entity()).despawn();
}

pub fn on_boat_used(
    trigger: Trigger<Interacted>,
    mut prompt: Query<&mut Text, With<PromptText>>,
    mut commands: Commands,
    animations: Res<Animations>,
    mut boat_root: Query<(Entity, &mut AnimationPlayer)>,
    mut player: Query<(Entity, &mut Transform), With<Player>>,
) {
    prompt.single_mut().sections[0].value = "".into();
    println!("Boat used");
    commands.entity(trigger.entity()).insert(ColliderDisabled);

    let mut transitions = AnimationTransitions::new();
    let (entity, mut animation_player) = boat_root.single_mut();
    transitions.play(
        &mut animation_player,
        animations.animations[0],
        Duration::new(0, 50000),
    );

    let (player, mut transform) = player.single_mut();
    commands.entity(player).insert(RigidBodyDisabled);
    commands
        .entity(entity)
        .add_child(player)
        .insert(animations.graph.clone())
        .insert(transitions);
    transform.translation = Vec3::new(0.0, 1.0, 0.0);
}
