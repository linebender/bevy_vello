use std::borrow::Cow;

use bevy::{prelude::*, reflect::TypePath, render::render_asset::RenderAsset};
use parley::{
    FontSettings, FontStyle, FontVariation, Layout, PositionedLayoutItem, RangedBuilder,
    StyleProperty,
};
use vello::{
    Scene,
    kurbo::Affine,
    peniko::{Brush, Fill},
};

use super::{VelloFontAxes, VelloTextAnchor, context::LOCAL_FONT_CONTEXT};
use crate::{
    integrations::text::context::{LOCAL_LAYOUT_CONTEXT, get_global_font_context},
    prelude::{VelloTextAlign, VelloTextStyle},
};

/// Computes the (dx, dy) translation offset for text anchoring.
///
/// For world-space text (`ui_content_size` is `None`), the anchor positions the text
/// bounding box relative to the transform origin (sprite-style).
///
/// For UI text (`ui_content_size` is `Some((node_w, node_h))`), the anchor controls
/// text alignment within the node's content box. The transform origin is at the node's
/// center (Bevy UI convention), so offsets are relative to that center point.
pub(crate) fn compute_anchor_offset(
    text_anchor: VelloTextAnchor,
    text_w: f64,
    text_h: f64,
    ui_content_size: Option<(f32, f32)>,
) -> (f64, f64) {
    if let Some((node_w, node_h)) = ui_content_size {
        // UI text: anchor controls alignment within the node's content box.
        // Bevy UI uses center-origin transforms, so the rendering origin is at
        // the node's center. We compute offsets in two steps:
        //   Step 1: shift from node center to content box top-left
        //   Step 2: apply CSS-like alignment within the content box
        let node_w = node_w as f64;
        let node_h = node_h as f64;
        let (align_x, align_y) = match text_anchor {
            VelloTextAnchor::TopLeft => (0.0, 0.0),
            VelloTextAnchor::Top => ((node_w - text_w) / 2.0, 0.0),
            VelloTextAnchor::TopRight => (node_w - text_w, 0.0),
            VelloTextAnchor::Left => (0.0, (node_h - text_h) / 2.0),
            VelloTextAnchor::Center => ((node_w - text_w) / 2.0, (node_h - text_h) / 2.0),
            VelloTextAnchor::Right => (node_w - text_w, (node_h - text_h) / 2.0),
            VelloTextAnchor::BottomLeft => (0.0, node_h - text_h),
            VelloTextAnchor::Bottom => ((node_w - text_w) / 2.0, node_h - text_h),
            VelloTextAnchor::BottomRight => (node_w - text_w, node_h - text_h),
        };
        (-node_w / 2.0 + align_x, -node_h / 2.0 + align_y)
    } else {
        // World-space text: anchor positions the text bounding box relative to the
        // transform origin. TopLeft=(0,0) means text grows down-right from origin.
        match text_anchor {
            VelloTextAnchor::TopLeft => (0.0, 0.0),
            VelloTextAnchor::Left => (0.0, -text_h / 2.0),
            VelloTextAnchor::BottomLeft => (0.0, -text_h),
            VelloTextAnchor::Top => (-text_w / 2.0, 0.0),
            VelloTextAnchor::Center => (-text_w / 2.0, -text_h / 2.0),
            VelloTextAnchor::Bottom => (-text_w / 2.0, -text_h),
            VelloTextAnchor::TopRight => (-text_w, 0.0),
            VelloTextAnchor::Right => (-text_w, -text_h / 2.0),
            VelloTextAnchor::BottomRight => (-text_w, -text_h),
        }
    }
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct VelloFont {
    /// Defaults to Bevy's bevy_text default font family name.
    ///
    /// https://github.com/bevyengine/bevy/tree/v0.15.3/crates/bevy_text/src/FiraMono-subset.ttf
    pub(crate) family_name: String,
    pub bytes: Vec<u8>,
}

impl RenderAsset for VelloFont {
    type SourceAsset = VelloFont;

    type Param = ();

    fn prepare_asset(
        source_asset: Self::SourceAsset,
        _asset_id: AssetId<Self::SourceAsset>,
        _param: &mut bevy::ecs::system::SystemParamItem<Self::Param>,
        _previous_asset: Option<&Self>,
    ) -> Result<Self, bevy::render::render_asset::PrepareAssetError<Self::SourceAsset>> {
        Ok(source_asset)
    }
}

impl VelloFont {
    pub fn new(font_data: Vec<u8>) -> Self {
        Self {
            bytes: font_data,
            family_name: "Fira Mono".to_string(),
        }
    }

    pub fn layout(
        &self,
        value: &str,
        style: &VelloTextStyle,
        text_align: VelloTextAlign,
        max_advance: Option<f32>,
    ) -> Layout<Brush> {
        LOCAL_FONT_CONTEXT.with_borrow_mut(|font_context| {
            if font_context.is_none() {
                *font_context = Some(get_global_font_context().clone());
            }

            let font_context = font_context.as_mut().unwrap();

            LOCAL_LAYOUT_CONTEXT.with_borrow_mut(|layout_context| {
                let mut builder = layout_context.ranged_builder(font_context, value, 1.0, true);

                apply_font_styles(&mut builder, style);
                apply_variable_axes(&mut builder, &style.font_axes);

                builder.push_default(StyleProperty::FontStack(parley::FontStack::Single(
                    parley::FontFamily::Named(Cow::Owned(self.family_name.clone())),
                )));

                let mut layout = builder.build(value);
                layout.break_all_lines(max_advance);
                layout.align(
                    max_advance,
                    text_align.into(),
                    parley::AlignmentOptions::default(),
                );

                layout
            })
        })
    }

    #[expect(clippy::too_many_arguments, reason = "Common lint in bevy")]
    pub(crate) fn render(
        &self,
        scene: &mut Scene,
        mut transform: Affine,
        value: &str,
        style: &VelloTextStyle,
        text_align: VelloTextAlign,
        max_advance: Option<f32>,
        text_anchor: VelloTextAnchor,
        ui_content_size: Option<(f32, f32)>,
    ) {
        let layout = self.layout(value, style, text_align, max_advance);

        let text_w = layout.width() as f64;
        let text_h = layout.height() as f64;

        let (dx, dy) = compute_anchor_offset(text_anchor, text_w, text_h, ui_content_size);
        transform *= vello::kurbo::Affine::translate((dx, dy));

        for line in layout.lines() {
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    continue;
                };

                let mut x = glyph_run.offset();
                let y = glyph_run.baseline();
                let run = glyph_run.run();
                let font = run.font();
                let font_size = run.font_size();
                let synthesis = run.synthesis();
                let glyph_xform = synthesis
                    .skew()
                    .map(|angle| Affine::skew(angle.to_radians().tan() as f64, 0.0));

                scene
                    .draw_glyphs(font)
                    .brush(&style.brush)
                    .hint(true)
                    .transform(transform)
                    .glyph_transform(glyph_xform)
                    .font_size(font_size)
                    .normalized_coords(run.normalized_coords())
                    .draw(
                        Fill::NonZero,
                        glyph_run.glyphs().map(|glyph| {
                            let gx = x + glyph.x;
                            let gy = y - glyph.y;
                            x += glyph.advance;
                            vello::Glyph {
                                id: glyph.id as _,
                                x: gx,
                                y: gy,
                            }
                        }),
                    );
            }
        }
    }
}

/// Applies the font styles to the text
///
/// font_size - font size
/// line_height - line height
/// word_spacing - extra spacing between words
/// letter_spacing - extra spacing between letters
fn apply_font_styles(builder: &mut RangedBuilder<'_, Brush>, style: &VelloTextStyle) {
    builder.push_default(StyleProperty::FontSize(style.font_size));
    builder.push_default(StyleProperty::LineHeight(
        parley::LineHeight::MetricsRelative(style.line_height),
    ));
    builder.push_default(StyleProperty::WordSpacing(style.word_spacing));
    builder.push_default(StyleProperty::LetterSpacing(style.letter_spacing));
}

/// Applies the variable axes to the text
///
/// wght - font weight
/// wdth - font width
/// opsz - optical size
/// ital - italic
/// slnt - slant
/// GRAD - grade
/// XOPQ - thick stroke
/// YOPQ - thin stroke
/// YTUC - uppercase height
/// YTLC - lowercase height
/// YTAS - ascender height
/// YTDE - descender depth
/// YTFI - figure height
fn apply_variable_axes(builder: &mut RangedBuilder<'_, Brush>, axes: &VelloFontAxes) {
    let mut variable_axes: Vec<FontVariation> = vec![];

    if let Some(weight) = axes.weight {
        variable_axes.push(parley::swash::Setting {
            tag: parley::swash::tag_from_str_lossy("wght"),
            value: weight,
        });
    }

    if let Some(width) = axes.width {
        variable_axes.push(parley::swash::Setting {
            tag: parley::swash::tag_from_str_lossy("wdth"),
            value: width,
        });
    }

    if let Some(optical_size) = axes.optical_size {
        variable_axes.push(parley::swash::Setting {
            tag: parley::swash::tag_from_str_lossy("opsz"),
            value: optical_size,
        });
    }

    if let Some(grade) = axes.grade {
        variable_axes.push(parley::swash::Setting {
            tag: parley::swash::tag_from_str_lossy("GRAD"),
            value: grade,
        });
    }

    if let Some(thick_stroke) = axes.thick_stroke {
        variable_axes.push(parley::swash::Setting {
            tag: parley::swash::tag_from_str_lossy("XOPQ"),
            value: thick_stroke,
        });
    }

    if let Some(thin_stroke) = axes.thin_stroke {
        variable_axes.push(parley::swash::Setting {
            tag: parley::swash::tag_from_str_lossy("YOPQ"),
            value: thin_stroke,
        });
    }

    if let Some(counter_width) = axes.counter_width {
        variable_axes.push(parley::swash::Setting {
            tag: parley::swash::tag_from_str_lossy("XTRA"),
            value: counter_width,
        });
    }

    if let Some(uppercase_height) = axes.uppercase_height {
        variable_axes.push(parley::swash::Setting {
            tag: parley::swash::tag_from_str_lossy("YTUC"),
            value: uppercase_height,
        });
    }

    if let Some(lowercase_height) = axes.lowercase_height {
        variable_axes.push(parley::swash::Setting {
            tag: parley::swash::tag_from_str_lossy("YTLC"),
            value: lowercase_height,
        });
    }

    if let Some(ascender_height) = axes.ascender_height {
        variable_axes.push(parley::swash::Setting {
            tag: parley::swash::tag_from_str_lossy("YTAS"),
            value: ascender_height,
        });
    }

    if let Some(descender_depth) = axes.descender_depth {
        variable_axes.push(parley::swash::Setting {
            tag: parley::swash::tag_from_str_lossy("YTDE"),
            value: descender_depth,
        });
    }

    if let Some(figure_height) = axes.figure_height {
        variable_axes.push(parley::swash::Setting {
            tag: parley::swash::tag_from_str_lossy("YTFI"),
            value: figure_height,
        });
    }

    if axes.italic {
        builder.push_default(StyleProperty::FontStyle(FontStyle::Italic));
    } else if axes.slant.is_some() {
        builder.push_default(StyleProperty::FontStyle(FontStyle::Oblique(axes.slant)));
    }

    builder.push_default(StyleProperty::FontVariations(FontSettings::List(
        variable_axes.into(),
    )));
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- World-space text (ui_content_size = None) ---
    // These should pass: existing behavior is correct for world-space.

    #[test]
    fn world_center_offsets_by_half_text_dims() {
        let (dx, dy) = compute_anchor_offset(VelloTextAnchor::Center, 200.0, 40.0, None);
        assert_eq!(dx, -100.0);
        assert_eq!(dy, -20.0);
    }

    #[test]
    fn world_top_left_is_zero() {
        let (dx, dy) = compute_anchor_offset(VelloTextAnchor::TopLeft, 200.0, 40.0, None);
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 0.0);
    }

    #[test]
    fn world_bottom_right_offsets_by_full_text_dims() {
        let (dx, dy) = compute_anchor_offset(VelloTextAnchor::BottomRight, 200.0, 40.0, None);
        assert_eq!(dx, -200.0);
        assert_eq!(dy, -40.0);
    }

    // --- UI text (ui_content_size = Some) ---
    // These test correct behavior: anchor positions text within the node's content box.
    // The transform origin is at the node's CENTER (Bevy UI convention).
    //
    // Given: node 400x200, text 200x40
    //
    // Step 1: shift from node center to top-left: (-200, -100)
    // Step 2: apply CSS-like alignment within content box
    //
    // TopLeft:     step2=(0, 0)         → (-200, -100)
    // Center:      step2=(100, 80)      → (-100, -20)  [same as text-only Center]
    // BottomRight: step2=(200, 160)     → (0, 60)
    // Left:        step2=(0, 80)        → (-200, -20)
    // TopRight:    step2=(200, 0)       → (0, -100)
    // Bottom:      step2=(100, 160)     → (-100, 60)
    // Top:         step2=(100, 0)       → (-100, -100)
    // Right:       step2=(200, 80)      → (0, -20)
    // BottomLeft:  step2=(0, 160)       → (-200, 60)

    const NODE_W: f32 = 400.0;
    const NODE_H: f32 = 200.0;
    const TEXT_W: f64 = 200.0;
    const TEXT_H: f64 = 40.0;
    const UI_SIZE: Option<(f32, f32)> = Some((NODE_W, NODE_H));

    #[test]
    fn ui_top_left_positions_at_node_top_left() {
        let (dx, dy) = compute_anchor_offset(VelloTextAnchor::TopLeft, TEXT_W, TEXT_H, UI_SIZE);
        assert_eq!((dx, dy), (-200.0, -100.0));
    }

    #[test]
    fn ui_center_centers_text_in_node() {
        // Center should produce same result as world-space when text != node,
        // but using node dims: (-text_w/2, -text_h/2) is the simple case.
        // Actually for UI: step1=(-200,-100) + step2=(100,80) = (-100, -20)
        let (dx, dy) = compute_anchor_offset(VelloTextAnchor::Center, TEXT_W, TEXT_H, UI_SIZE);
        assert_eq!((dx, dy), (-100.0, -20.0));
    }

    #[test]
    fn ui_bottom_right_positions_at_node_bottom_right() {
        // step1=(-200,-100) + step2=(200, 160) = (0, 60)
        let (dx, dy) = compute_anchor_offset(VelloTextAnchor::BottomRight, TEXT_W, TEXT_H, UI_SIZE);
        assert_eq!((dx, dy), (0.0, 60.0));
    }

    #[test]
    fn ui_left_vertically_centers_at_left_edge() {
        // step1=(-200,-100) + step2=(0, 80) = (-200, -20)
        let (dx, dy) = compute_anchor_offset(VelloTextAnchor::Left, TEXT_W, TEXT_H, UI_SIZE);
        assert_eq!((dx, dy), (-200.0, -20.0));
    }

    #[test]
    fn ui_top_right_positions_at_node_top_right() {
        // step1=(-200,-100) + step2=(200, 0) = (0, -100)
        let (dx, dy) = compute_anchor_offset(VelloTextAnchor::TopRight, TEXT_W, TEXT_H, UI_SIZE);
        assert_eq!((dx, dy), (0.0, -100.0));
    }

    #[test]
    fn ui_bottom_centers_at_bottom_edge() {
        // step1=(-200,-100) + step2=(100, 160) = (-100, 60)
        let (dx, dy) = compute_anchor_offset(VelloTextAnchor::Bottom, TEXT_W, TEXT_H, UI_SIZE);
        assert_eq!((dx, dy), (-100.0, 60.0));
    }

    #[test]
    fn ui_top_centers_at_top_edge() {
        // step1=(-200,-100) + step2=(100, 0) = (-100, -100)
        let (dx, dy) = compute_anchor_offset(VelloTextAnchor::Top, TEXT_W, TEXT_H, UI_SIZE);
        assert_eq!((dx, dy), (-100.0, -100.0));
    }

    #[test]
    fn ui_right_vertically_centers_at_right_edge() {
        // step1=(-200,-100) + step2=(200, 80) = (0, -20)
        let (dx, dy) = compute_anchor_offset(VelloTextAnchor::Right, TEXT_W, TEXT_H, UI_SIZE);
        assert_eq!((dx, dy), (0.0, -20.0));
    }

    #[test]
    fn ui_bottom_left_positions_at_node_bottom_left() {
        // step1=(-200,-100) + step2=(0, 160) = (-200, 60)
        let (dx, dy) = compute_anchor_offset(VelloTextAnchor::BottomLeft, TEXT_W, TEXT_H, UI_SIZE);
        assert_eq!((dx, dy), (-200.0, 60.0));
    }

    // Edge case: text fills node exactly — UI and world Center should match
    #[test]
    fn ui_center_matches_world_when_text_fills_node() {
        let (ui_dx, ui_dy) =
            compute_anchor_offset(VelloTextAnchor::Center, 400.0, 200.0, Some((400.0, 200.0)));
        let (w_dx, w_dy) = compute_anchor_offset(VelloTextAnchor::Center, 400.0, 200.0, None);
        assert_eq!((ui_dx, ui_dy), (w_dx, w_dy));
    }
}
