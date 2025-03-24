//! Components and text logic.

mod context;
mod font;
mod font_loader;
mod vello_text;

pub use font::VelloFont;
pub(crate) use font_loader::VelloFontLoader;
pub use vello_text::{VelloTextAnchor, VelloTextSection, VelloTextStyle};

#[cfg(feature = "default_font")]
mod default_font_plugin;

#[cfg(feature = "default_font")]
pub(crate) use default_font_plugin::DefaultFontPlugin;
