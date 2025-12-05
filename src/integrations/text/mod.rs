//! Components and text logic.

mod context;
mod font;
mod font_loader;
mod systems;
mod vello_text;

pub(crate) mod render;

pub use font::VelloFont;
pub use vello_text::{UiVelloText, VelloText2d, VelloTextAlign, VelloTextAnchor, VelloTextStyle};

mod plugin;
pub(crate) use plugin::VelloTextIntegrationPlugin;
