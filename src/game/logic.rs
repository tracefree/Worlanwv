use std::{f32::consts::PI, time::Duration};

use bevy::{animation::RepeatAnimation, prelude::*};
use bevy_rapier3d::{
    plugin::RapierContext,
    prelude::{ColliderDisabled, CollisionGroups, Group, QueryFilter, RigidBodyDisabled},
};

use crate::{game::audio::sfx::GroundMaterial, screen::PlayState, AppSet};

use super::{
    animation::Animations,
    assets::SfxKey,
    audio::sfx::PlaySfx,
    movement::MovementController,
    spawn::{
        level::{SkyMaterial, Sun, SunPivot, Terrain},
        player::{Player, PlayerCamera},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(CurrentCycle(Cycle::One))
        .insert_resource(DayProgress(0.0))
        .insert_resource(CurrentHighlighted(None))
        .insert_resource(BoatPosition::default())
        .insert_resource(Inventory::default());
    app.observe(on_cycle_changed);
    app.observe(cast_ground_ray);
    app.register_type::<Interactable>();
    app.add_systems(
        Update,
        (
            animate_sun,
            //   animate_water,
            tick_animation_timers,
        )
            .in_set(AppSet::TickTimers)
            .run_if(in_state(PlayState::InGame)),
    );
    app.add_systems(
        Update,
        (respawn, handle_interaction)
            .in_set(AppSet::Update)
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
    pub fn next(self) -> Self {
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

#[derive(Component)]
pub struct AnimationTimer(Timer);

#[derive(Event)]
pub struct AnimationFinished;

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

#[derive(Resource, Default)]
pub struct BoatPosition {
    pub initial_transform: Transform,
    pub docked_at_island: bool,
    pub currently_rowing: bool,
}

fn on_cycle_changed(
    trigger: Trigger<CycleChanged>,
    mut current_cycle: ResMut<CurrentCycle>,
    mut colliders: Query<(&mut Transform, &Cycle), Without<AnimationPlayer>>,
    mut prompt: Query<&mut Text, With<PromptText>>,
    mut commands: Commands,
    mut boat: Query<&mut Transform, With<AnimationPlayer>>,
    mut boat_position: ResMut<BoatPosition>,
) {
    current_cycle.0 = trigger.event().0;
    prompt.single_mut().sections[0].value = "".into();

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

    *boat.single_mut() = boat_position.initial_transform;
    boat_position.docked_at_island = false;
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
    boat_position: Res<BoatPosition>,
) {
    // TODO: Cleaner solution for pausing time
    if boat_position.currently_rowing {
        return;
    }

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

/*
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
*/

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
            false
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
) {
    let (_, rotation, translation) = camera.single().to_scale_rotation_translation();
    let result = rapier_context.cast_ray(
        translation + rotation * Vec3::NEG_Z * 0.35,
        rotation * Vec3::NEG_Z,
        2.0,
        false,
        QueryFilter::from(CollisionGroups::new(
            Group::all() & !Group::GROUP_2,
            Group::GROUP_2,
        )),
    );

    if let Some((object, _)) = result {
        highlighted.0 = Some(object);
    } else {
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

fn update_highlight_mesh(
    highlighted: Res<CurrentHighlighted>,
    interactables: Query<(Entity, &Interactable)>,
    mut prompt: Query<&mut Text, With<PromptText>>,
    mut commands: Commands,
) {
    let mut something_highlighted = false;
    let mut text = prompt.single_mut();
    for (entity, interactable) in interactables.iter() {
        if let Some(highlight_mesh) = interactable.highlight_mesh {
            if highlighted.0 == Some(entity) {
                commands.entity(highlight_mesh).insert(Visibility::Visible);
                text.sections[0].value = interactable.text.clone();
                something_highlighted = true;
            } else {
                commands.entity(highlight_mesh).insert(Visibility::Hidden);
            }
        }
    }
    if !something_highlighted && text.sections[0].value.starts_with("E: ") {
        text.sections[0].value = "".into();
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

pub fn on_sapling_taken(
    trigger: Trigger<Interacted>,
    mut inventory: ResMut<Inventory>,
    q_sapling: Query<(Entity, &Name)>,
    mut commands: Commands,
) {
    inventory.sapling = true;
    commands.trigger(PlaySfx::Key(SfxKey::Harvest));
    for (entity, name) in q_sapling.iter() {
        if name.as_str().contains("Sapling") || name.as_str().contains("TreeLower") {
            commands.entity(entity).despawn_recursive();
        }
    }
    commands.entity(trigger.entity()).insert(ColliderDisabled);
}

pub fn on_sapling_planted(
    trigger: Trigger<Interacted>,
    mut inventory: ResMut<Inventory>,
    q_sapling: Query<(Entity, &Name)>,
    mut commands: Commands,
) {
    inventory.sapling = false;
    commands.trigger(PlaySfx::Key(SfxKey::Harvest));
    for (entity, name) in q_sapling.iter() {
        if name.as_str().contains("FinalSap") {
            commands.entity(entity).insert(Visibility::Visible);
        }
        if name.as_str().contains("TreeUpper") {
            commands
                .entity(entity)
                .insert(Visibility::Visible)
                .remove::<ColliderDisabled>();
        }
    }
    commands.entity(trigger.entity()).insert(ColliderDisabled);
}

pub fn on_monument_finished(
    trigger: Trigger<Interacted>,
    q_sapling: Query<(Entity, &Name)>,
    mut commands: Commands,
) {
    println!("Monument finished");
    //commands.trigger(PlaySfx::Key(SfxKey::Harvest));
    for (entity, name) in q_sapling.iter() {
        /*    if name.as_str().contains("FinalSap") {
            commands.entity(entity).insert(Visibility::Visible);
        }
        if name.as_str().contains("TreeUpper") {
            commands
                .entity(entity)
                .insert(Visibility::Visible)
                .remove::<ColliderDisabled>();
        }
        */
    }
    commands.entity(trigger.entity()).insert(ColliderDisabled);
}

pub fn on_boat_used(
    trigger: Trigger<Interacted>,
    mut prompt: Query<&mut Text, With<PromptText>>,
    mut commands: Commands,
    animations: Res<Animations>,
    mut boat_root: Query<(Entity, &mut AnimationPlayer)>,
    mut player: Query<(Entity, &mut Transform, &mut MovementController), With<Player>>,
    mut boat_position: ResMut<BoatPosition>,
) {
    prompt.single_mut().sections[0].value = "".into();
    commands.trigger(PlaySfx::Key(SfxKey::Row));
    commands
        .entity(trigger.entity())
        .insert(ColliderDisabled)
        .insert(AnimationTimer(Timer::from_seconds(4.0, TimerMode::Once)))
        .observe(on_boat_ride_finished);

    let mut transitions = AnimationTransitions::new();
    let (entity, mut animation_player) = boat_root.single_mut();
    let animation_index = match boat_position.docked_at_island {
        true => 1,
        false => 0,
    };
    transitions
        .play(
            &mut animation_player,
            animations.animations[animation_index],
            Duration::ZERO,
        )
        .set_repeat(RepeatAnimation::Count(1))
        .replay();

    let (player, mut transform, mut controller) = player.single_mut();
    controller.disabled = true;
    commands.entity(player).insert(RigidBodyDisabled);
    commands
        .entity(entity)
        .add_child(player)
        .insert(animations.graph.clone())
        .insert(transitions);

    transform.translation = Vec3::new(0.0, 1.0, 0.0);
    boat_position.docked_at_island = !boat_position.docked_at_island;
    boat_position.currently_rowing = true;
}

fn tick_animation_timers(
    mut timers: Query<(Entity, &mut AnimationTimer)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut timer) in timers.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            commands.trigger_targets(AnimationFinished, entity);
        }
    }
}

fn on_boat_ride_finished(
    trigger: Trigger<AnimationFinished>,
    mut commands: Commands,
    mut player: Query<
        (
            Entity,
            &mut Transform,
            &GlobalTransform,
            &mut MovementController,
        ),
        With<Player>,
    >,
    mut boat_position: ResMut<BoatPosition>,
) {
    let (player, mut transform, global_transform, mut controller) = player.single_mut();
    controller.disabled = false;

    commands
        .entity(player)
        .remove::<RigidBodyDisabled>()
        .remove_parent();

    commands
        .entity(trigger.entity())
        .remove::<ColliderDisabled>();
    transform.translation = global_transform.translation() + Vec3::new(0.5, 1.0, 0.5);
    boat_position.currently_rowing = false;
}

fn respawn(
    mut transform: Query<&mut Transform, With<Player>>,
    mut commands: Commands,
    current_cycle: Res<CurrentCycle>,
) {
    let mut transform = transform.single_mut();
    if transform.translation.y <= 0.0 {
        transform.translation = Vec3::new(5.52, 4.4, -33.66);
        commands.trigger(CycleChanged(current_cycle.0.next()));
    }
}
