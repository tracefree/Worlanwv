//! A credits screen that can be accessed from the title screen.

use std::fmt::format;

use bevy::prelude::*;
use ui_palette::NODE_BACKGROUND;

use super::Screen;
use crate::{
    game::{assets::SoundtrackKey, audio::soundtrack::PlaySoundtrack, logic::CurrentCycle},
    ui::prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Credits), enter_credits);
    app.add_systems(OnExit(Screen::Credits), exit_credits);

    app.add_systems(
        Update,
        handle_credits_action.run_if(in_state(Screen::Credits)),
    );
    app.register_type::<CreditsAction>();
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
enum CreditsAction {
    Back,
}

fn enter_credits(mut commands: Commands, current_cycle: Res<CurrentCycle>) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::Credits))
        .insert(BackgroundColor(NODE_BACKGROUND))
        .with_children(|children| {
            children.header(format!("Time taken to complete your project: {} years", current_cycle.1 * 12020));

            children.header("Game Design, Programming, 3D Art");
            children.label("Rie");

            children.header("Original Music");
            children.label("Ira Provectus and Michael Feigl");

            children.header("Bevy Quickstart Template");
            children.label("TheBevyFlock");

            children.header("Free sound assets");
            children.label("AudioPaplin, kangaroovindaloo, Andreas Mustola, moogy73, OwlishMedia, Valenspire, juskiddink");

        });

    commands.trigger(PlaySoundtrack::Key(SoundtrackKey::CycleOne));
}

fn exit_credits(mut commands: Commands) {
    commands.trigger(PlaySoundtrack::Disable);
}

fn handle_credits_action(
    mut next_screen: ResMut<NextState<Screen>>,
    mut button_query: InteractionQuery<&CreditsAction>,
) {
    for (interaction, action) in &mut button_query {
        if matches!(interaction, Interaction::Pressed) {
            match action {
                CreditsAction::Back => next_screen.set(Screen::Playing),
            }
        }
    }
}
