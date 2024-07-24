//! A loading screen during which game assets are loaded.
//! This reduces stuttering, especially for audio on WASM.

use bevy::{
    prelude::*,
    render::texture::{ImageLoaderSettings, ImageSampler},
};

use super::Screen;
use crate::{
    game::assets::{HandleMap, ImageKey, SfxKey, SoundtrackKey},
    ui::prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.enable_state_scoped_entities::<Screen>();
    app.add_systems(OnEnter(Screen::Loading), enter_loading);
    app.add_systems(
        Update,
        continue_to_title.run_if(in_state(Screen::Loading).and_then(all_assets_loaded)),
    );
}

const SPLASH_BACKGROUND_COLOR: Color = Color::srgb(0.157, 0.157, 0.157);
const SPLASH_DURATION_SECS: f32 = 1.8;
const SPLASH_FADE_DURATION_SECS: f32 = 0.6;

fn enter_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::Loading))
        .insert((
            Name::new("Splash screen"),
            BackgroundColor(SPLASH_BACKGROUND_COLOR),
            StateScoped(Screen::Loading),
        ))
        .with_children(|children| {
            children
                .spawn((
                    Name::new("Splash image"),
                    ImageBundle {
                        style: Style {
                            margin: UiRect::all(Val::Auto),
                            width: Val::Percent(70.0),
                            ..default()
                        },
                        image: UiImage::new(asset_server.load_with_settings(
                            // This should be an embedded asset for instant loading, but that is
                            // currently [broken on Windows Wasm builds](https://github.com/bevyengine/bevy/issues/14246).
                            "textures/splash.png",
                            |settings: &mut ImageLoaderSettings| {
                                // Make an exception for the splash image in case
                                // `ImagePlugin::default_nearest()` is used for pixel art.
                                settings.sampler = ImageSampler::linear();
                            },
                        )),
                        ..default()
                    },
                ))
                .insert(StateScoped(Screen::Loading));
        });
}

fn all_assets_loaded(
    asset_server: Res<AssetServer>,
    image_handles: Res<HandleMap<ImageKey>>,
    sfx_handles: Res<HandleMap<SfxKey>>,
    soundtrack_handles: Res<HandleMap<SoundtrackKey>>,
) -> bool {
    image_handles.all_loaded(&asset_server)
        && sfx_handles.all_loaded(&asset_server)
        && soundtrack_handles.all_loaded(&asset_server)
}

fn continue_to_title(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Playing);
}
