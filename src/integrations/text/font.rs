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
        ui_content: Option<Vec2>,
        clip: Option<vello::kurbo::Rect>,
    ) {
        let layout = self.layout(value, style, text_align, max_advance);

        let text_w = layout.width() as f64;
        let text_h = layout.height() as f64;

        let (dx, dy) = if let Some(content_size) = ui_content {
            let offset = compute_ui_anchor_offset(
                text_anchor,
                text_w,
                text_h,
                content_size.x,
                content_size.y,
            );
            // Transform the logical offset through the linear part of the affine
            // (rotation + scale), so anchor positioning is correct under any transform.
            let c = transform.as_coeffs();
            (
                c[0] * offset.0 + c[2] * offset.1,
                c[1] * offset.0 + c[3] * offset.1,
            )
        } else {
            compute_world_anchor_offset(text_anchor, text_w, text_h)
        };

        transform = transform.then_translate(vello::kurbo::Vec2::new(dx, dy));

        // Precompute clip range in layout (logical) space for glyph-run culling.
        // The affine maps layout coords → physical screen space. Inverting lets us
        // express the clip rect in the same space as glyph_run.baseline(), avoiding
        // Vello encoding glyphs that the clip rect would hide anyway.
        // A margin of 2× font_size covers ascenders, descenders, and rounding.
        let cull_y: Option<(f64, f64)> = clip.and_then(|r| {
            let c = transform.as_coeffs();
            // Skip culling when the transform has rotation — screen-space y bounds
            // don't map cleanly to layout-space y bounds under rotation.
            if c[1].abs() > f64::EPSILON || c[2].abs() > f64::EPSILON {
                return None;
            }
            let scale_y = c[3];
            if scale_y.abs() < f64::EPSILON {
                return None;
            }
            let translate_y = c[5];
            let margin = style.font_size as f64 * 2.0;
            Some((
                (r.y0 - translate_y) / scale_y - margin,
                (r.y1 - translate_y) / scale_y + margin,
            ))
        });

        'lines: for line in layout.lines() {
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    continue;
                };

                let baseline = glyph_run.baseline() as f64;

                // Cull glyph runs outside the clip rect. Parley emits lines in
                // top-to-bottom order, so exceeding y_max means all remaining
                // lines are also outside — break out entirely.
                if let Some((y_min, y_max)) = cull_y {
                    if baseline < y_min {
                        continue;
                    }
                    if baseline > y_max {
                        break 'lines;
                    }
                }

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

/// Computes the (dx, dy) translation offset for world-space text anchoring.
///
/// Positions the text bounding box relative to the transform origin.
/// `TopLeft=(0,0)` means text grows down-right from origin.
pub(crate) fn compute_world_anchor_offset(
    text_anchor: VelloTextAnchor,
    text_w: f64,
    text_h: f64,
) -> (f64, f64) {
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

/// Computes the (dx, dy) translation offset for UI text anchoring.
///
/// Aligns text within the node's content box. The UiGlobalTransform places the
/// origin at the node's center, so we compute offsets relative to that center
/// to position text according to the anchor.
pub(crate) fn compute_ui_anchor_offset(
    text_anchor: VelloTextAnchor,
    text_w: f64,
    text_h: f64,
    node_w: f32,
    node_h: f32,
) -> (f64, f64) {
    let node_w = node_w as f64;
    let node_h = node_h as f64;
    let top_left_x = -node_w / 2.0;
    let top_left_y = -node_h / 2.0;

    let (anchor_x, anchor_y) = match text_anchor {
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

    (top_left_x + anchor_x, top_left_y + anchor_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- World-space text ---

    #[test]
    fn world_center_offsets_by_half_text_dims() {
        let (dx, dy) = compute_world_anchor_offset(VelloTextAnchor::Center, 200.0, 40.0);
        assert_eq!((dx, dy), (-100.0, -20.0));
    }

    #[test]
    fn world_top_left_is_zero() {
        let (dx, dy) = compute_world_anchor_offset(VelloTextAnchor::TopLeft, 200.0, 40.0);
        assert_eq!((dx, dy), (0.0, 0.0));
    }

    #[test]
    fn world_bottom_right_offsets_by_full_text_dims() {
        let (dx, dy) = compute_world_anchor_offset(VelloTextAnchor::BottomRight, 200.0, 40.0);
        assert_eq!((dx, dy), (-200.0, -40.0));
    }

    // --- UI text ---
    // Anchor positions text within the node's content box.
    // Transform origin is at the node's CENTER (UiGlobalTransform origin).
    //
    // Given: node 400x200, text 200x40
    // Top-left corner is at (-200, -100) from center
    //
    // TopLeft:     (-200, -100)
    // Center:      (-100, -20)
    // BottomRight: (0, 60)

    const NODE_W: f32 = 400.0;
    const NODE_H: f32 = 200.0;
    const TEXT_W: f64 = 200.0;
    const TEXT_H: f64 = 40.0;

    #[test]
    fn ui_top_left_positions_at_node_top_left() {
        let (dx, dy) =
            compute_ui_anchor_offset(VelloTextAnchor::TopLeft, TEXT_W, TEXT_H, NODE_W, NODE_H);
        assert_eq!((dx, dy), (-200.0, -100.0));
    }

    #[test]
    fn ui_center_centers_text_in_node() {
        let (dx, dy) =
            compute_ui_anchor_offset(VelloTextAnchor::Center, TEXT_W, TEXT_H, NODE_W, NODE_H);
        assert_eq!((dx, dy), (-100.0, -20.0));
    }

    #[test]
    fn ui_bottom_right_positions_at_node_bottom_right() {
        let (dx, dy) =
            compute_ui_anchor_offset(VelloTextAnchor::BottomRight, TEXT_W, TEXT_H, NODE_W, NODE_H);
        assert_eq!((dx, dy), (0.0, 60.0));
    }

    #[test]
    fn ui_left_vertically_centers_at_left_edge() {
        let (dx, dy) =
            compute_ui_anchor_offset(VelloTextAnchor::Left, TEXT_W, TEXT_H, NODE_W, NODE_H);
        assert_eq!((dx, dy), (-200.0, -20.0));
    }

    #[test]
    fn ui_top_right_positions_at_node_top_right() {
        let (dx, dy) =
            compute_ui_anchor_offset(VelloTextAnchor::TopRight, TEXT_W, TEXT_H, NODE_W, NODE_H);
        assert_eq!((dx, dy), (0.0, -100.0));
    }

    #[test]
    fn ui_bottom_centers_at_bottom_edge() {
        let (dx, dy) =
            compute_ui_anchor_offset(VelloTextAnchor::Bottom, TEXT_W, TEXT_H, NODE_W, NODE_H);
        assert_eq!((dx, dy), (-100.0, 60.0));
    }

    #[test]
    fn ui_top_centers_at_top_edge() {
        let (dx, dy) =
            compute_ui_anchor_offset(VelloTextAnchor::Top, TEXT_W, TEXT_H, NODE_W, NODE_H);
        assert_eq!((dx, dy), (-100.0, -100.0));
    }

    #[test]
    fn ui_right_vertically_centers_at_right_edge() {
        let (dx, dy) =
            compute_ui_anchor_offset(VelloTextAnchor::Right, TEXT_W, TEXT_H, NODE_W, NODE_H);
        assert_eq!((dx, dy), (0.0, -20.0));
    }

    #[test]
    fn ui_bottom_left_positions_at_node_bottom_left() {
        let (dx, dy) =
            compute_ui_anchor_offset(VelloTextAnchor::BottomLeft, TEXT_W, TEXT_H, NODE_W, NODE_H);
        assert_eq!((dx, dy), (-200.0, 60.0));
    }

    /// The full UI anchor pipeline — compute logical offset, transform through
    /// the affine's linear part, apply to affine — must produce correct
    /// translations at non-1x scale factors.
    #[test]
    fn ui_anchor_pipeline_correct_at_2x_dpi() {
        let scale = 2.0_f64;
        // Affine for a UI node at (250, 150) logical on a 2x display.
        let affine = Affine::new([scale, 0.0, 0.0, scale, 500.0, 300.0]);

        // Node is 200x100 logical, text is 100x20.
        // Center anchor: offset = (-50, -10) logical.
        let offset = compute_ui_anchor_offset(VelloTextAnchor::Center, 100.0, 20.0, 200.0, 100.0);

        // Apply linear part of affine to offset — same path as render().
        let c = affine.as_coeffs();
        let dx = c[0] * offset.0 + c[2] * offset.1;
        let dy = c[1] * offset.0 + c[3] * offset.1;
        let result = affine.then_translate(vello::kurbo::Vec2::new(dx, dy));
        let c = result.as_coeffs();

        // Text origin should land at (250-50, 150-10) = (200, 140) logical,
        // which is (400, 280) physical.
        assert_eq!((c[4], c[5]), (400.0, 280.0));
    }

    /// Anchor offset must be rotated correctly when the node's affine includes
    /// a rotation. A 90-degree CCW rotation swaps axes, so the logical x-offset
    /// becomes a physical y-offset and vice versa.
    #[test]
    fn ui_anchor_pipeline_correct_under_rotation() {
        let scale = 2.0_f64;
        let cos90 = 0.0_f64;
        let sin90 = 1.0_f64;
        // 90-degree CCW rotation with 2x scale, node center at (500, 300) physical.
        // Affine columns: [cos*s, sin*s, -sin*s, cos*s, tx, ty]
        let affine = Affine::new([
            cos90 * scale,
            sin90 * scale,
            -sin90 * scale,
            cos90 * scale,
            500.0,
            300.0,
        ]);

        // Node is 200x100 logical, text is 100x20.
        // Center anchor: offset = (-50, -10) logical.
        let offset = compute_ui_anchor_offset(VelloTextAnchor::Center, 100.0, 20.0, 200.0, 100.0);

        // Apply linear part of affine to offset — same path as render().
        let c = affine.as_coeffs();
        let dx = c[0] * offset.0 + c[2] * offset.1;
        let dy = c[1] * offset.0 + c[3] * offset.1;
        let result = affine.then_translate(vello::kurbo::Vec2::new(dx, dy));
        let c = result.as_coeffs();

        // Under 90-degree CCW rotation, logical (-50, -10) maps to physical:
        //   screen_x += cos*s*(-50) + (-sin*s)*(-10) = 0*(-50) + (-2)*(-10) = +20
        //   screen_y += sin*s*(-50) + cos*s*(-10)    = 2*(-50) + 0*(-10)    = -100
        // So text origin lands at (500+20, 300-100) = (520, 200) physical.
        assert_eq!((c[4], c[5]), (520.0, 200.0));
    }

    #[test]
    fn ui_center_is_origin_when_text_fills_node() {
        let (ui_dx, ui_dy) =
            compute_ui_anchor_offset(VelloTextAnchor::Center, 400.0, 200.0, 400.0, 200.0);
        assert_eq!((ui_dx, ui_dy), (-200.0, -100.0));

        let (ui_tl_dx, ui_tl_dy) =
            compute_ui_anchor_offset(VelloTextAnchor::TopLeft, 400.0, 200.0, 400.0, 200.0);
        assert_eq!((ui_tl_dx, ui_tl_dy), (-200.0, -100.0));

        let (w_dx, w_dy) = compute_world_anchor_offset(VelloTextAnchor::TopLeft, 400.0, 200.0);
        assert_eq!((w_dx, w_dy), (0.0, 0.0));
    }
}
