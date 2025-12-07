//! Components and text logic.

mod context;
mod font;
mod font_loader;
mod systems;

pub(crate) mod render;

pub use font::VelloFont;

mod plugin;
pub(crate) use plugin::VelloTextIntegrationPlugin;

use bevy::{
    camera::{primitives::Aabb, visibility::VisibilityClass},
    prelude::*,
    ui::ContentSize,
};

#[derive(Component, Default, Clone)]
#[require(Aabb, VelloTextAnchor, Transform, Visibility, VisibilityClass)]
#[cfg_attr(feature = "picking", require(Pickable))]
#[component(on_add = bevy::camera::visibility::add_visibility_class::<VelloText2d>)]
pub struct VelloText2d {
    pub value: String,
    pub style: VelloTextStyle,
    pub text_align: VelloTextAlign,
    pub max_advance: Option<f32>,
}

#[derive(Component, Default, Clone)]
#[require(ContentSize, VelloTextAnchor, UiTransform, Visibility, VisibilityClass)]
#[component(on_add = bevy::camera::visibility::add_visibility_class::<UiVelloText>)]
pub struct UiVelloText {
    pub value: String,
    pub style: VelloTextStyle,
    pub text_align: VelloTextAlign,
    pub max_advance: Option<f32>,
}

#[derive(Clone)]
pub struct VelloTextStyle {
    pub font: Handle<VelloFont>,
    pub brush: vello::peniko::Brush,
    pub font_size: f32,
    /// Line height multiplier.
    pub line_height: f32,
    /// Extra spacing between words.
    pub word_spacing: f32,
    /// Extra spacing between letters.
    pub letter_spacing: f32,
    pub font_axes: VelloFontAxes,
}

impl Default for VelloTextStyle {
    fn default() -> Self {
        Self {
            font: Default::default(),
            brush: vello::peniko::Brush::Solid(vello::peniko::Color::WHITE),
            font_size: 24.0,
            line_height: 1.0,
            word_spacing: 0.0,
            letter_spacing: 0.0,
            font_axes: Default::default(),
        }
    }
}

/// Describes the variable axes of a font.
///
/// https://fonts.google.com/knowledge/introducing_type/introducing_variable_fonts
///
/// Each axis is optional and only present if the font supports it.
#[derive(Default, Clone)]
pub struct VelloFontAxes {
    /// wght variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/weight_axis
    pub weight: Option<f32>,
    /// wdth variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/width_axis
    pub width: Option<f32>,
    /// opsz variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/optical_size_axis
    pub optical_size: Option<f32>,
    /// ital variable axis only if the font supports it
    ///
    /// Mutually exclusive with `slant`.
    ///
    /// https://fonts.google.com/knowledge/glossary/italic_axis
    pub italic: bool,
    /// slnt variable axis only if the font supports it
    ///
    /// Mutually exclusive with `italic`.
    ///
    /// If italic is true, slant will be ignored.
    ///
    /// https://fonts.google.com/knowledge/glossary/slant_axis
    pub slant: Option<f32>,

    /// GRAD variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/grade_axis
    pub grade: Option<f32>,

    /// XOPQ variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/xopq_axis
    pub thick_stroke: Option<f32>,
    /// yopq variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/yopq_axis
    pub thin_stroke: Option<f32>,

    /// XTRA variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/xtra_axis
    pub counter_width: Option<f32>,

    /// YTUC variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/ytuc_axis
    pub uppercase_height: Option<f32>,
    /// YTLC variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/ytlc_axis
    pub lowercase_height: Option<f32>,

    /// YTAS variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/ytas_axis
    pub ascender_height: Option<f32>,
    /// YTDE variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/ytde_axis
    pub descender_depth: Option<f32>,

    /// YTFI variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/ytfi_axis
    pub figure_height: Option<f32>,
}

/// Describes how the text is positioned relative to its [`Transform`]. It defaults to
/// [`VelloTextAnchor::BottomLeft`].
#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
pub enum VelloTextAnchor {
    /// Bounds start from the render position and advance up and to the right.
    BottomLeft,
    /// Bounds start from the render position and advance up.
    Bottom,
    /// Bounds start from the render position and advance up and to the left.
    BottomRight,

    /// Bounds start from the render position and advance right.
    Left,
    /// Bounds start from the render position and advance equally on both axes.
    #[default]
    Center,
    /// Bounds start from the render position and advance left.
    Right,

    /// Bounds start from the render position and advance down and to the right.
    TopLeft,
    /// Bounds start from the render position and advance down.
    Top,
    /// Bounds start from the render position and advance down and to the left.
    TopRight,
}

/// Alignment of a parley layout.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum VelloTextAlign {
    /// This is [`parley::Alignment::Left`] for LTR text and [`parley::Alignment::Right`] for RTL
    /// text.
    #[default]
    Start,
    /// This is [`parley::Alignment::Right`] for LTR text and [`parley::Alignment::Left`] for RTL
    /// text.
    End,
    /// Align content to the left edge.
    ///
    /// For alignment that should be aware of text direction, use [`parley::Alignment::Start`] or
    /// [`parley::Alignment::End`] instead.
    Left,
    /// Align each line centered within the container.
    Middle,
    /// Align content to the right edge.
    ///
    /// For alignment that should be aware of text direction, use [`parley::Alignment::Start`] or
    /// [`parley::Alignment::End`] instead.
    Right,
    /// Justify each line by spacing out content, except for the last line.
    Justified,
}

impl From<VelloTextAlign> for parley::Alignment {
    fn from(value: VelloTextAlign) -> Self {
        match value {
            VelloTextAlign::Start => parley::Alignment::Start,
            VelloTextAlign::End => parley::Alignment::End,
            VelloTextAlign::Left => parley::Alignment::Left,
            VelloTextAlign::Middle => parley::Alignment::Center,
            VelloTextAlign::Right => parley::Alignment::Right,
            VelloTextAlign::Justified => parley::Alignment::Justify,
        }
    }
}
