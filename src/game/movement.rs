//! Handle player input and translate it into movement.
//! Note that the approach used here is simple for demonstration purposes.
//! If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/latest/examples/movement/physics_in_fixed_timestep.rs).

use std::f32::consts::PI;

use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_rapier3d::{
    control::KinematicCharacterController, dynamics::Velocity,
    prelude::KinematicCharacterControllerOutput,
};

use crate::{
    screen::{PlayState, Screen},
    AppSet,
};

use super::{assets::SfxKey, audio::sfx::PlaySfx, spawn::player::CameraPivot};

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.register_type::<MovementController>();
    app.insert_resource(FootstepTimer(0.0));
    app.add_systems(
        Update,
        update_footstep_timer
            .in_set(AppSet::TickTimers)
            .run_if(in_state(PlayState::InGame)),
    );
    app.add_systems(
        Update,
        record_movement_controller
            .in_set(AppSet::RecordInput)
            .run_if(in_state(PlayState::InGame)),
    );

    // Apply movement based on controls.
    app.add_systems(
        Update,
        (apply_movement, rotate_camera)
            .chain()
            .run_if(in_state(PlayState::InGame))
            .in_set(AppSet::Update),
    );
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MovementController {
    pub direction: Vec2,
    pub jump: bool,
}

#[derive(Resource)]
pub struct FootstepTimer(pub f32);

fn record_movement_controller(
    input: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<(&mut MovementController, &KinematicCharacterControllerOutput)>,
    mut footstep_timer: ResMut<FootstepTimer>,
) {
    // Collect directional input.
    let mut intent = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) {
        intent.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) {
        intent.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) {
        intent.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) {
        intent.x += 1.0;
    }

    // Normalize so that diagonal movement has the same speed as
    // horizontal and vertical movement.
    let intent = intent.normalize_or_zero();

    // Apply movement intent to controllers.
    for (mut controller, kinematic_output) in controller_query.iter_mut() {
        controller.direction = intent;
        controller.jump = input.just_pressed(KeyCode::Space) && kinematic_output.grounded;
        if intent.length() < 0.5 || !kinematic_output.grounded {
            footstep_timer.0 = 0.0;
        }
    }
}

fn apply_movement(
    time: Res<Time>,
    mut movement_query: Query<(
        &MovementController,
        &Velocity,
        &mut KinematicCharacterController,
    )>,
    camera_pivot: Query<&Transform, With<CameraPivot>>,
) {
    for (controller, velocity, mut body) in &mut movement_query {
        let pivot = camera_pivot.single();
        // TODO: Find better way to do this
        let speed = 3.0;
        let planar_velocity =
            Quat::from_rotation_y(pivot.rotation.to_euler(EulerRot::YZX).0).mul_vec3(Vec3::new(
                controller.direction.x,
                0.0,
                -controller.direction.y,
            )) * speed;

        //velocity.linvel = Vec3::new(planar_velocity.x, velocity.linvel.y, planar_velocity.z);

        println!("{}", velocity.linvel.y);

        body.translation = Some(
            Vec3::new(
                planar_velocity.x,
                if controller.jump {
                    9.0
                } else {
                    velocity.linvel.y - 9.8 * time.delta_seconds()
                },
                planar_velocity.z,
            ) * time.delta_seconds(),
        );
    }
}

fn rotate_camera(
    mut pivot: Query<&mut Transform, With<CameraPivot>>,
    mut mouse_motion: EventReader<MouseMotion>,
) {
    for mut transform in pivot.iter_mut() {
        for motion in mouse_motion.read() {
            let yaw = -motion.delta.x * 0.003;
            let pitch = -motion.delta.y * 0.002;

            transform.rotate_y(yaw);

            let current_pitch = transform.rotation.to_euler(EulerRot::YXZ).1;
            if (current_pitch > -60.0_f32.to_radians() && pitch < 0.0)
                || (current_pitch < 60.0_f32.to_radians() && pitch > 0.0)
            {
                transform.rotate_local_x(pitch);
            }
        }
    }
}

fn update_footstep_timer(
    mut footstep_timer: ResMut<FootstepTimer>,
    time: Res<Time>,
    mut commands: Commands,
) {
    footstep_timer.0 += time.delta_seconds();
    if footstep_timer.0 >= 0.5 {
        footstep_timer.0 = 0.0;
        commands.trigger(PlaySfx::RandomStep);
    }
}
