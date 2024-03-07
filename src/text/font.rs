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
use bevy::{prelude::*, reflect::TypePath};
use std::sync::Arc;
use vello::{
    glyph::{
        skrifa::{FontRef, MetadataProvider},
        Glyph, GlyphContext,
    },
    kurbo::Affine,
    peniko::{self, Blob, Font},
    Scene,
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
        let axes = font.axes();
        let variations: &[(&str, f32)] = &[];
        let var_loc = axes.location(variations);
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

    pub(crate) fn render(
        &self,
        scene: &mut Scene,
        transform: Affine,
        text: &VelloText,
    ) {
        let font = FontRef::new(self.font.data.data())
            .expect("Vello font creation error");

        let font_size = vello::skrifa::instance::Size::new(text.size);
        let charmap = font.charmap();
        let axes = font.axes();
        let variations: &[(&str, f32)] = &[];
        let var_loc = axes.location(variations);
        let metrics = font.metrics(font_size, &var_loc);
        let line_height = metrics.ascent - metrics.descent + metrics.leading;
        let glyph_metrics = font.glyph_metrics(font_size, &var_loc);

        let mut pen_x = 0f32;
        let mut pen_y = 0f32;
        scene
            .draw_glyphs(&self.font)
            .font_size(text.size)
            .transform(transform)
            .normalized_coords(var_loc.coords())
            .brush(&text.brush.clone().unwrap_or_default())
            .draw(
                &vello::kurbo::Stroke::new(1.0),
                text.content.chars().filter_map(|ch| {
                    if ch == '\n' {
                        pen_y += line_height;
                        pen_x = 0.0;
                        return None;
                    }
                    let gid = charmap.map(ch).unwrap_or_default();
                    let advance =
                        glyph_metrics.advance_width(gid).unwrap_or_default();
                    let x = pen_x;
                    pen_x += advance;
                    Some(Glyph {
                        id: gid.to_u16() as u32,
                        x,
                        y: pen_y,
                    })
                }),
            )
    }
}
