//! Handle player input and translate it into movement.
//! Note that the approach used here is simple for demonstration purposes.
//! If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/latest/examples/movement/physics_in_fixed_timestep.rs).

use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_rapier3d::control::KinematicCharacterController;

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
pub struct MovementController(pub Vec2);

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
        controller.0 = intent;
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
    mut movement_query: Query<(&MovementController, &mut KinematicCharacterController)>,
    camera_pivot: Query<&Transform, With<CameraPivot>>,
) {
    for (controller, mut body) in &mut movement_query {
        let pivot = camera_pivot.single();
        // TODO: Find better way to do this
        let mut velocity = Quat::from_rotation_y(pivot.rotation.to_euler(EulerRot::YZX).0)
            .mul_vec3(Vec3::new(controller.0.x, 0.0, -controller.0.y));

        if let Some(current_velocity) = body.translation {
            velocity.y += current_velocity.y;
        }

        let speed = 3.0;
        velocity *= speed;
        body.translation = Some(velocity * time.delta_seconds());
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
