//! Components and text logic.

mod context;
mod font;
mod font_loader;
mod vello_text;

pub use font::VelloFont;
pub(crate) use font_loader::{VelloFontLoader, VelloFontLoaderPlugin};
pub use vello_text::{VelloTextAnchor, VelloTextSection, VelloTextStyle};
