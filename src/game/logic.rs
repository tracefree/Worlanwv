use bevy::prelude::*;

use crate::AppSet;

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(CurrentCycle(Cycle::One));
    app.observe(on_cycle_changed);
    app.add_systems(Update, handle_input.in_set(AppSet::RecordInput));
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

fn on_cycle_changed(
    trigger: Trigger<CycleChanged>,
    mut current_cycle: ResMut<CurrentCycle>,
    mut colliders: Query<(&mut Transform, &Cycle)>,
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
}

fn handle_input(
    input: Res<ButtonInput<KeyCode>>,
    current_cycle: Res<CurrentCycle>,
    mut commands: Commands,
) {
    if input.just_pressed(KeyCode::ArrowRight) {
        let next_cycle = current_cycle.0.next();
        commands.trigger(CycleChanged(next_cycle));
    }
}
