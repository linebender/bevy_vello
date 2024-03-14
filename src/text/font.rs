use super::vello_text::VelloText;
use bevy::reflect::TypePath;
use bevy::render::render_asset::RenderAsset;
use bevy::{prelude::*, time};
use std::sync::Arc;
use std::time::Instant;
use vello::glyph::skrifa::{FontRef, MetadataProvider};
use vello::glyph::Glyph;
use vello::kurbo::Affine;
use vello::peniko::{self, Blob, Brush, Color, Font};
use vello::Scene;

const VARIATIONS: &[(&str, f32)] = &[];

#[derive(Asset, TypePath, Clone)]
pub struct VelloFont {
    pub font: Arc<peniko::Font>,
}

impl RenderAsset for VelloFont {
    type PreparedAsset = VelloFont;

    type Param = ();

    fn asset_usage(&self) -> bevy::render::render_asset::RenderAssetUsages {
        Default::default()
    }

    fn prepare_asset(
        self,
        _param: &mut bevy::ecs::system::SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, bevy::render::render_asset::PrepareAssetError<Self>> {
        Ok(self)
    }
}

impl VelloFont {
    pub fn new(font_data: Vec<u8>) -> Self {
        Self {
            font: Arc::new(Font::new(Blob::new(Arc::new(font_data)), 0)),
        }
    }

    pub fn sizeof(&self, text: &VelloText) -> Vec2 {
        let font = FontRef::new(self.font.data.data()).expect("Vello font creation error");
        let font_size = vello::skrifa::instance::Size::new(text.size);
        let charmap = font.charmap();
        let axes = font.axes();
        let var_loc = axes.location(VARIATIONS);
        let metrics = font.metrics(font_size, &var_loc);
        let line_height = metrics.ascent - metrics.descent + metrics.leading;
        let glyph_metrics = font.glyph_metrics(font_size, &var_loc);

        let mut pen_x = 0.0;
        let mut pen_y: f32 = 0.0;
        let mut width: f32 = 0.0;
        let mut height: f32 = line_height;
        for ch in text.content.chars() {
            if ch == '\n' {
                pen_y += line_height;
                pen_x = 0.0;
                continue;
            }
            let gid = charmap.map(ch).unwrap_or_default();
            let advance = glyph_metrics.advance_width(gid).unwrap_or_default();

            pen_x += advance;
            width = width.max(pen_x);
        }
        height += pen_y;
        Vec2::new(width, height)
    }

    pub(crate) fn render(&self, scene: &mut Scene, transform: Affine, text: &VelloText) {
        let font = FontRef::new(self.font.data.data()).expect("Vello font creation error");

        let font_size = vello::skrifa::instance::Size::new(text.size);
        let charmap = font.charmap();
        let axes = font.axes();
        let var_loc = axes.location(VARIATIONS);
        let metrics = font.metrics(font_size, &var_loc);
        let line_height = metrics.ascent - metrics.descent + metrics.leading;
        let glyph_metrics = font.glyph_metrics(font_size, &var_loc);

        let mut pen_x = 0f32;
        let mut pen_y = 0f32;
        let mut glyphs: Vec<Glyph> = text
            .content
            .chars()
            .filter_map(|ch| {
                if ch == '\n' {
                    pen_y += line_height;
                    pen_x = 0.0;
                    return None;
                }
                let gid = charmap.map(ch).unwrap_or_default();
                let advance = glyph_metrics.advance_width(gid).unwrap_or_default();
                let x = pen_x;
                pen_x += advance;
                Some(Glyph {
                    id: gid.to_u16() as u32,
                    x,
                    y: pen_y,
                })
            })
            .collect();
        // Push text up
        glyphs.iter_mut().for_each(|g| {
            g.y -= pen_y;
        });
        scene
            .draw_glyphs(&self.font)
            .font_size(text.size)
            .transform(transform)
            .normalized_coords(var_loc.coords())
            .brush(&text.brush.clone().unwrap_or(Brush::Solid(Color::WHITE)))
            .draw(vello::peniko::Fill::EvenOdd, glyphs.into_iter());
    }
}
