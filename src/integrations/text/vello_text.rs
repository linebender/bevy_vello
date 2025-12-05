use bevy::{
    camera::{
        primitives::Aabb,
        visibility::{self, VisibilityClass},
    },
    math::DVec2,
    prelude::*,
    ui::{ContentSize, NodeMeasure},
};
use tracing::warn;
use vello::peniko::{self, Brush};

use crate::VelloFont;

#[derive(Component, Default, Clone)]
#[require(Aabb, VelloTextAnchor, Transform, Visibility, VisibilityClass)]
#[component(on_add = visibility::add_visibility_class::<VelloText2d>)]
pub struct VelloText2d {
    pub value: String,
    pub style: VelloTextStyle,
    pub text_align: VelloTextAlign,
    pub max_advance: Option<f32>,
}

#[derive(Component, Default, Clone)]
#[require(VelloTextAnchor, UiTransform, Visibility, VisibilityClass)]
#[component(on_add = visibility::add_visibility_class::<UiVelloText>)]
pub struct UiVelloText {
    pub value: String,
    pub style: VelloTextStyle,
    pub text_align: VelloTextAlign,
    pub max_advance: Option<f32>,
}

#[derive(Clone)]
pub struct VelloTextStyle {
    pub font: Handle<VelloFont>,
    pub brush: Brush,
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
            brush: Brush::Solid(peniko::Color::WHITE),
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

impl VelloTextAnchor {
    pub fn adjustment(&self, width: f64, height: f64) -> DVec2 {
        match self {
            VelloTextAnchor::TopLeft => DVec2::new(0.0, 0.0),
            VelloTextAnchor::Left => DVec2::new(0.0, -height / 2.0),
            VelloTextAnchor::BottomLeft => DVec2::new(0.0, -height),
            VelloTextAnchor::Top => DVec2::new(-width / 2.0, 0.0),
            VelloTextAnchor::Center => DVec2::new(-width / 2.0, -height / 2.0),
            VelloTextAnchor::Bottom => DVec2::new(-width / 2.0, -height),
            VelloTextAnchor::TopRight => DVec2::new(-width, 0.0),
            VelloTextAnchor::Right => DVec2::new(-width, -height / 2.0),
            VelloTextAnchor::BottomRight => DVec2::new(-width, -height),
        }
    }
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

pub fn update_text_2d_aabb_on_change(
    mut text_q: Query<
        (&mut Aabb, &mut VelloText2d, &VelloTextAnchor, &Transform),
        Or<(Changed<VelloText2d>, Changed<Transform>)>,
    >,
    fonts: Res<Assets<VelloFont>>,
) {
    for (mut aabb, text, text_anchor, transform) in text_q.iter_mut() {
        let Some(font) = fonts.get(&text.style.font) else {
            warn!("VelloText2d: font {:?} not found", text.style.font);
            continue;
        };

        let layout = font.layout(&text.value, &text.style, text.text_align, text.max_advance);
        let (width, height) = (layout.width(), layout.height());
        let half_size = Vec3::new(width / 2.0, height / 2.0, 0.0);
        let (dx, dy) = {
            match text_anchor {
                VelloTextAnchor::TopLeft => (half_size.x, -half_size.y),
                VelloTextAnchor::Left => (half_size.x, 0.0),
                VelloTextAnchor::BottomLeft => (half_size.x, half_size.y),
                VelloTextAnchor::Top => (0.0, -half_size.y),
                VelloTextAnchor::Center => (0.0, 0.0),
                VelloTextAnchor::Bottom => (0.0, half_size.y),
                VelloTextAnchor::TopRight => (-half_size.x, -half_size.y),
                VelloTextAnchor::Right => (-half_size.x, 0.0),
                VelloTextAnchor::BottomRight => (-half_size.x, half_size.y),
            }
        };
        let adjustment = Vec3::new(dx, dy, 0.0);
        let min = transform.translation - half_size + adjustment;
        let max = transform.translation + half_size + adjustment;
        *aabb = Aabb::from_min_max(min, max);
    }
}

pub fn update_ui_text_content_size_on_change(
    mut text_q: Query<
        (&mut ContentSize, &ComputedNode, &mut UiVelloText),
        Or<(Changed<UiVelloText>, Changed<ComputedNode>)>,
    >,
    fonts: Res<Assets<VelloFont>>,
) {
    for (mut content_size, node, text) in text_q.iter_mut() {
        let Some(font) = fonts.get(&text.style.font) else {
            warn!("UiVelloText: font {:?} not found", text.style.font);
            continue;
        };

        let layout = font.layout(&text.value, &text.style, text.text_align, text.max_advance);
        let size = Vec2::new(layout.width(), layout.height()) / node.inverse_scale_factor();
        let measure = NodeMeasure::Fixed(bevy::ui::FixedMeasure { size });
        content_size.set(measure);
    }
}
