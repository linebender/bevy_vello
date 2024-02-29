// Copyright 2022 The vello authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// Also licensed under MIT license, at your choice.

use super::vello_text::VelloText;
use bevy::{prelude::*, reflect::TypePath, render::render_asset::RenderAsset};
use std::sync::Arc;
use vello::{
    glyph::{skrifa::FontRef, GlyphContext},
    kurbo::Affine,
    peniko::{self, Blob, Brush, Font},
};

#[derive(Asset, TypePath)]
pub struct VelloFont {
    gcx: GlyphContext,
    pub font: peniko::Font,
}

impl VelloFont {
    pub fn new(font_data: Vec<u8>) -> Self {
        Self {
            gcx: GlyphContext::new(),
            font: Font::new(Blob::new(Arc::new(font_data)), 0),
        }
    }

    pub fn sizeof(&self, text: &VelloText) -> Vec2 {
        let font = FontRef::new(self.font.data.data())
            .expect("Vello font creation error");

        let font_size = vello::skrifa::instance::Size::new(text.size);
        let charmap = font.charmap();
        let metrics = font.metrics(font_size, Default::default());
        let line_height = metrics.ascent - metrics.descent + metrics.leading;
        let glyph_metrics = font.glyph_metrics(font_size, Default::default());

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

    pub fn render(
        &mut self,
        builder: &mut SceneBuilder,
        transform: Affine,
        text: &VelloText,
    ) {
        let frags = self.render_fragments(
            text.size,
            text.brush.as_ref(),
            transform,
            &text.content,
        );
        for (glyph, transform) in frags {
            builder.append(&glyph, Some(transform));
        }
    }

    pub(crate) fn render_fragments(
        &mut self,
        size: f32,
        brush: Option<&Brush>,
        transform: Affine,
        text: &str,
    ) -> Vec<(SceneFragment, Affine)> {
        let font = FontRef::new(self.font.data.data())
            .expect("Vello font creation error");

        let mut items = vec![];

        let font_size = vello::skrifa::instance::Size::new(size);
        let charmap = font.charmap();
        let metrics = font.metrics(font_size, Default::default());
        let line_height = metrics.ascent - metrics.descent + metrics.leading;
        let glyph_metrics = font.glyph_metrics(font_size, Default::default());
        let vars: [(&str, f32); 0] = [];
        let mut provider =
            self.gcx.new_provider(&font, None, size, false, vars);

        let mut pen_x: f64 = 0.0;
        let mut pen_y: f64 = 0.0;

        for ch in text.chars() {
            if ch == '\n' {
                pen_y += line_height as f64;
                pen_x = 0.0;
                continue;
            }
            let gid = charmap.map(ch).unwrap_or_default();
            let advance =
                glyph_metrics.advance_width(gid).unwrap_or_default() as f64;

            if let Some(glyph) = provider.get(gid.to_u16(), brush) {
                let xform = transform
                    * Affine::translate((pen_x, pen_y))
                    * Affine::scale_non_uniform(1.0, -1.0);
                // builder.append(&glyph, Some(xform));
                items.push((glyph, xform));
            }
            pen_x += advance;
        }
        items
            .into_iter()
            // Push all lines up to account for new lines
            .map(|(f, a)| (f, a * Affine::translate((0.0, pen_y))))
            .collect()
    }
}
