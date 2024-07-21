//! Handle player input and translate it into movement.
//! Note that the approach used here is simple for demonstration purposes.
//! If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/latest/examples/movement/physics_in_fixed_timestep.rs).

use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_rapier3d::{control::KinematicCharacterController, dynamics::Velocity};

use crate::AppSet;

use super::spawn::player::CameraPivot;

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.register_type::<MovementController>();
    app.add_systems(
        Update,
        record_movement_controller.in_set(AppSet::RecordInput),
    );

    // Apply movement based on controls.
    app.register_type::<Movement>();
    app.add_systems(
        Update,
        (apply_movement, rotate_camera)
            .chain()
            .in_set(AppSet::Update),
    );
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MovementController {
    pub direction: Vec2,
    pub jump: bool,
}

fn record_movement_controller(
    input: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<&mut MovementController>,
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
    for mut controller in &mut controller_query {
        controller.direction = intent;
        if input.just_pressed(KeyCode::Space) {
            controller.jump = true;
        } else {
            controller.jump = false;
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Movement {
    /// Since Bevy's default 2D camera setup is scaled such that
    /// one unit is one pixel, you can think of this as
    /// "How many pixels per second should the player move?"
    /// Note that physics engines may use different unit/pixel ratios.
    pub speed: f32,
}

fn apply_movement(
    time: Res<Time>,
    mut movement_query: Query<(
        &MovementController,
        &mut Velocity,
        &mut KinematicCharacterController,
    )>,
    camera_pivot: Query<&Transform, With<CameraPivot>>,
) {
    for (controller, mut velocity, mut body) in &mut movement_query {
        let pivot = camera_pivot.single();
        // TODO: Find better way to do this
        let speed = 3.0;
        let planar_velocity =
            Quat::from_rotation_y(pivot.rotation.to_euler(EulerRot::YZX).0).mul_vec3(Vec3::new(
                controller.direction.x,
                0.0,
                -controller.direction.y,
            )) * speed;

        velocity.linvel = Vec3::new(
            planar_velocity.x,
            velocity.linvel.y - 9.8 * time.delta_seconds(),
            planar_velocity.z,
        );

        if controller.jump {
            velocity.linvel.y = 3.0;
        }
        println!("{}", velocity.linvel);

        body.translation = Some(velocity.linvel * time.delta_seconds());
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
            transform.rotate_local_x(pitch);
        }
    }
}
