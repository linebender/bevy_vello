mod asset_loader;
mod systems;

pub(crate) mod render;

pub mod asset;
pub use asset::VelloLottie;

mod parse;
use crate::integrations::VelloAnchor;
pub use parse::{load_lottie_from_bytes, load_lottie_from_str};

mod lottie_ext;
pub use lottie_ext::LottieExt;

mod plugin;
pub(crate) use plugin::LottieIntegrationPlugin;

mod theme;
pub use theme::Theme;

mod player;
pub use player::{
    LottiePlayer, PlaybackDirection, PlaybackLoopBehavior, PlaybackOptions, PlaybackPlayMode,
    PlayerState, PlayerTransition, Playhead,
};

use bevy::{
    camera::{primitives::Aabb, visibility::VisibilityClass},
    ecs::component::Mutable,
    prelude::*,
};

pub trait LottieAssetVariant: Component<Mutability = Mutable> + Clone {
    fn asset_id(&self) -> AssetId<VelloLottie>;
}

/// A renderable Lottie in the world.
#[derive(Component, Default, Debug, Clone, Deref, DerefMut, PartialEq, Eq, Reflect)]
#[require(
    Aabb,
    Playhead,
    PlaybackOptions,
    LottiePlayer::<VelloLottie2d>,
    Transform,
    VelloAnchor,
    Visibility,
    VisibilityClass
)]
#[cfg_attr(feature = "picking", require(Pickable))]
#[reflect(Component)]
#[component(on_add = super::lottie::player::hooks::on_add_lottie::<VelloLottie2d>)]
pub struct VelloLottie2d(pub Handle<VelloLottie>);

impl LottieAssetVariant for VelloLottie2d {
    fn asset_id(&self) -> AssetId<VelloLottie> {
        self.id()
    }
}

/// A renderable Lottie that may be used in Bevy UI.
///
/// ### Object fit
/// The image will preserve the aspect ratio, and fits the image inside the container, without cutting - will leave empty space if needed.
#[derive(Component, Default, Debug, Clone, Deref, DerefMut, PartialEq, Eq, Reflect)]
#[require(
    Node,
    Playhead,
    PlaybackOptions,
    LottiePlayer::<UiVelloLottie>,
    UiTransform,
    VelloAnchor,
    Visibility,
    VisibilityClass
)]
#[reflect(Component)]
#[component(on_add = super::lottie::player::hooks::on_add_lottie::<UiVelloLottie>)]
pub struct UiVelloLottie(pub Handle<VelloLottie>);

impl LottieAssetVariant for UiVelloLottie {
    fn asset_id(&self) -> AssetId<VelloLottie> {
        self.id()
    }
}
