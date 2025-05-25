//! Components and text logic.

mod context;
mod font;
mod font_loader;
mod vello_text;

pub(crate) mod render;

pub use font::VelloFont;
pub use vello_text::{VelloTextAlign, VelloTextAnchor, VelloTextSection, VelloTextStyle};

mod plugin;
pub(crate) use plugin::VelloTextIntegrationPlugin;

use bevy::prelude::*;
#[derive(Bundle, Default)]
pub struct VelloTextBundle {
    /// Text to render
    pub text: VelloTextSection,
    /// How the text is positioned relative to its [`Transform`].
    pub text_anchor: VelloTextAnchor,
    /// A transform to apply to this text
    pub transform: Transform,
    /// User indication of whether an entity is visible. Propagates down the entity hierarchy.
    pub view_visibility: ViewVisibility,
}
