use std::borrow::Cow;

use super::{
    context::{FONT_CONTEXT, LAYOUT_CONTEXT},
    vello_text::VelloTextSection,
    VelloTextAnchor,
};
use bevy::{prelude::*, reflect::TypePath, render::render_asset::RenderAsset};
use parley::{FontWeight, InlineBox, PositionedLayoutItem, StyleProperty};
use vello::{kurbo::Affine, peniko::Fill, Scene};

#[derive(Asset, TypePath, Debug, Clone)]
pub struct VelloFont {
    pub family_name: String,
    pub bytes: Vec<u8>,
}

impl RenderAsset for VelloFont {
    type SourceAsset = VelloFont;

    type Param = ();

    fn prepare_asset(
        source_asset: Self::SourceAsset,
        _param: &mut bevy::ecs::system::SystemParamItem<Self::Param>,
    ) -> Result<Self, bevy::render::render_asset::PrepareAssetError<Self::SourceAsset>> {
        Ok(source_asset)
    }
}

impl VelloFont {
    pub fn new(font_data: Vec<u8>) -> Self {
        Self {
            bytes: font_data,
            family_name: "".to_string(),
        }
    }

    pub fn sizeof(&self, text: &VelloTextSection) -> Vec2 {
        FONT_CONTEXT.with_borrow_mut(|font_context| {
            debug!(
                "Thread {:?} sizeof collection {:?}",
                std::thread::current().id(),
                font_context.collection.family_names().collect::<Vec<_>>()
            );

            LAYOUT_CONTEXT.with_borrow_mut(|layout_context| {
                // TODO: fix scale magic number
                let mut builder = layout_context.ranged_builder(font_context, &text.value, 1.0);

                // if let Some(weight) = text.style.weight {
                //     // sets the font weight for the entire text
                //     builder.push_default(StyleProperty::FontWeight(FontWeight::new(weight)));
                // }
                //
                // if let Some(line_height) = text.style.line_height {
                //     // sets the line height for the entire text
                //     builder.push_default(StyleProperty::LineHeight(line_height));
                // }

                builder.push_default(StyleProperty::FontStack(parley::FontStack::Single(
                    parley::FontFamily::Named(Cow::Borrowed(&self.family_name)),
                )));

                builder.push_inline_box(InlineBox {
                    id: 0,
                    index: 5,
                    width: 50.0,
                    height: 50.0,
                });

                // TODO: "bb code" for text styling?
                // can use builder.push(StyleProperty::FontWeight(FontWeight::new(700)), 11..42) to
                // make the characters in the range bold;

                let mut layout = builder.build(&text.value);

                // break_all_lines is required to layout anything
                layout.break_all_lines(None);

                // layout.align(None, parley::Alignment::Middle, AlignmentOptions::default());

                Vec2::new(layout.width(), layout.height())
            })
        })
    }

    pub(crate) fn render(
        &self,
        scene: &mut Scene,
        mut transform: Affine,
        text: &VelloTextSection,
        text_anchor: VelloTextAnchor,
    ) {
        FONT_CONTEXT.with_borrow_mut(|font_context| {
            LAYOUT_CONTEXT.with_borrow_mut(|layout_context| {
                debug!(
                    "Thread {:?} render collection {:?}",
                    std::thread::current().id(),
                    font_context.collection.family_names().collect::<Vec<_>>()
                );

                // TODO: fix scale magic number
                let mut builder = layout_context.ranged_builder(font_context, &text.value, 1.0);

                if let Some(weight) = text.style.weight {
                    // sets the font weight for the entire text
                    builder.push_default(StyleProperty::FontWeight(FontWeight::new(weight)));
                }

                if let Some(line_height) = text.style.line_height {
                    // sets the line height for the entire text
                    builder.push_default(StyleProperty::LineHeight(line_height));
                }

                debug!("Family name: {:?}", self.family_name);
                builder.push_default(StyleProperty::FontStack(parley::FontStack::Single(
                    parley::FontFamily::Named(Cow::Borrowed(&self.family_name)),
                )));

                // TODO: "bb code" for text styling?
                // can use builder.push(StyleProperty::FontWeight(FontWeight::new(700)), 11..42) to
                // make the characters in the range bold;

                let mut layout = builder.build(&text.value);

                // break_all_lines is required to render anything
                layout.break_all_lines(None);

                //layout.align(None, parley::Alignment::Middle, AlignmentOptions::default());

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
                            .brush(&text.style.brush)
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

                let width = layout.width() as f64;
                let height = layout.height() as f64;
                debug!("Width: {:?}, Height: {:?}", width, height);

                match text_anchor {
                    VelloTextAnchor::TopLeft => {
                        transform *= vello::kurbo::Affine::translate((0.0, height));
                    }
                    VelloTextAnchor::Left => {
                        transform *= vello::kurbo::Affine::translate((0.0, height / 2.0));
                    }
                    VelloTextAnchor::BottomLeft => {
                        transform *= vello::kurbo::Affine::translate((0.0, 0.0));
                    }
                    VelloTextAnchor::Top => {
                        transform *= vello::kurbo::Affine::translate((-width / 2.0, height));
                    }
                    VelloTextAnchor::Center => {
                        transform *= vello::kurbo::Affine::translate((-width / 2.0, height / 2.0));
                    }
                    VelloTextAnchor::Bottom => {
                        transform *= vello::kurbo::Affine::translate((-width / 2.0, 0.0));
                    }
                    VelloTextAnchor::TopRight => {
                        transform *= vello::kurbo::Affine::translate((-width, height));
                    }
                    VelloTextAnchor::Right => {
                        transform *= vello::kurbo::Affine::translate((-width, height / 2.0));
                    }
                    VelloTextAnchor::BottomRight => {
                        transform *= vello::kurbo::Affine::translate((-width, 0.0));
                    }
                }
            });
        })
    }
}
