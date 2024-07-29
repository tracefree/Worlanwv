//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    dev_tools::{fps_overlay::FpsOverlayPlugin, states::log_transitions},
    prelude::*,
};

use crate::{
    game::logic::{CurrentCycle, CycleChanged},
    screen::Screen,
    AppSet,
};

pub(super) fn plugin(app: &mut App) {
    // Print state transitions in dev builds
    app.add_systems(Update, log_transitions::<Screen>);
    //app.add_plugins(RapierDebugRenderPlugin::default());
    app.add_plugins(FpsOverlayPlugin::default());
    app.add_systems(Update, handle_input.in_set(AppSet::RecordInput));
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
