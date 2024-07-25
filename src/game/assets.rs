use bevy::{
    prelude::*,
    render::texture::{ImageLoaderSettings, ImageSampler},
    utils::HashMap,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<HandleMap<ImageKey>>();
    app.init_resource::<HandleMap<ImageKey>>();

    app.register_type::<HandleMap<SfxKey>>();
    app.init_resource::<HandleMap<SfxKey>>();

    app.register_type::<HandleMap<SoundtrackKey>>();
    app.init_resource::<HandleMap<SoundtrackKey>>();
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum ImageKey {
    Cubemap,
}

impl AssetKey for ImageKey {
    type Asset = Image;
}

impl FromWorld for HandleMap<ImageKey> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        [(
            ImageKey::Cubemap,
            asset_server.load_with_settings(
                "textures/cubemap.ktx2",
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::linear();
                },
            ),
        )]
        .into()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum SfxKey {
    ButtonHover,
    ButtonPress,
    GrassStep1,
    GrassStep2,
    GrassStep3,
    GrassStep4,
    HardStep1,
    HardStep2,
    HardStep3,
    HardStep4,
    CycleChange,
}

impl AssetKey for SfxKey {
    type Asset = AudioSource;
}

impl FromWorld for HandleMap<SfxKey> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        [
            (
                SfxKey::ButtonHover,
                asset_server.load("audio/sfx/button_hover.wav"),
            ),
            (
                SfxKey::ButtonPress,
                asset_server.load("audio/sfx/button_click.wav"),
            ),
            (
                SfxKey::GrassStep1,
                asset_server.load("audio/sfx/footsteps/grass_1.wav"),
            ),
            (
                SfxKey::GrassStep2,
                asset_server.load("audio/sfx/footsteps/grass_2.wav"),
            ),
            (
                SfxKey::GrassStep3,
                asset_server.load("audio/sfx/footsteps/grass_3.wav"),
            ),
            (
                SfxKey::GrassStep4,
                asset_server.load("audio/sfx/footsteps/grass_4.wav"),
            ),
            (
                SfxKey::HardStep1,
                asset_server.load("audio/sfx/footsteps/hard_1.wav"),
            ),
            (
                SfxKey::HardStep2,
                asset_server.load("audio/sfx/footsteps/hard_2.wav"),
            ),
            (
                SfxKey::HardStep3,
                asset_server.load("audio/sfx/footsteps/hard_3.wav"),
            ),
            (
                SfxKey::HardStep4,
                asset_server.load("audio/sfx/footsteps/hard_4.wav"),
            ),
            (
                SfxKey::CycleChange,
                asset_server.load("audio/sfx/cycle_change.wav"),
            ),
        ]
        .into()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum SoundtrackKey {
    Credits,
    Gameplay,
    OceanAmbiance,
}

impl AssetKey for SoundtrackKey {
    type Asset = AudioSource;
}

impl FromWorld for HandleMap<SoundtrackKey> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        [
            (
                SoundtrackKey::Credits,
                asset_server.load("audio/soundtracks/Monkeys Spinning Monkeys.ogg"),
            ),
            (
                SoundtrackKey::Gameplay,
                asset_server.load("audio/soundtracks/Fluffing A Duck.ogg"),
            ),
            (
                SoundtrackKey::OceanAmbiance,
                asset_server.load("audio/ambiance/ocean.wav"),
            ),
        ]
        .into()
    }
}

pub trait AssetKey: Sized {
    type Asset: Asset;
}

#[derive(Resource, Reflect, Deref, DerefMut)]
#[reflect(Resource)]
pub struct HandleMap<K: AssetKey>(HashMap<K, Handle<K::Asset>>);

impl<K: AssetKey, T> From<T> for HandleMap<K>
where
    T: Into<HashMap<K, Handle<K::Asset>>>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl<K: AssetKey> HandleMap<K> {
    pub fn all_loaded(&self, asset_server: &AssetServer) -> bool {
        self.values()
            .all(|x| asset_server.is_loaded_with_dependencies(x))
    }
}
