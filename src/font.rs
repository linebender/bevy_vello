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

use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypePath,
    render::render_asset::RenderAsset,
    utils::BoxedFuture,
};
use std::sync::Arc;
use vello::{
    fello::meta::MetadataProvider,
    fello::raw::FontRef,
    glyph::GlyphContext,
    kurbo::Affine,
    peniko::{self, Blob, Brush, Font},
    SceneBuilder, SceneFragment,
};

use crate::assets::VectorLoaderError;

#[derive(Asset, TypePath)]
pub struct VelloFont {
    gcx: GlyphContext,
    pub font: peniko::Font,
}

impl RenderAsset for VelloFont {
    type ExtractedAsset = VelloFont;

    type PreparedAsset = VelloFont;

    type Param = ();

    fn extract_asset(&self) -> Self::ExtractedAsset {
        VelloFont {
            font: self.font.clone(),
            gcx: GlyphContext::default(),
        }
    }

    fn prepare_asset(
        data: Self::ExtractedAsset,
        _param: &mut bevy::ecs::system::SystemParamItem<Self::Param>,
    ) -> Result<
        Self::PreparedAsset,
        bevy::render::render_asset::PrepareAssetError<Self::ExtractedAsset>,
    > {
        Ok(data)
    }
}

impl VelloFont {
    pub fn new(font_data: Vec<u8>) -> Self {
        Self {
            gcx: GlyphContext::new(),
            font: Font::new(Blob::new(Arc::new(font_data)), 0),
        }
    }

    pub fn render_centered(
        &mut self,
        builder: &mut SceneBuilder,
        size: f32,
        transform: Affine,
        text: &str,
    ) {
        let (glyphs, text_width, _text_height) = self.add(size, None, transform, text);
        for (glyph, xform) in glyphs {
            let xform = xform * Affine::translate((-text_width / 2.0, 0.0));
            builder.append(&glyph, Some(xform));
        }
    }

    fn add(
        &mut self,
        size: f32,
        brush: Option<&Brush>,
        transform: Affine,
        text: &str,
    ) -> (Vec<(SceneFragment, Affine)>, f64, f64) {
        let font = FontRef::new(self.font.data.data()).expect("Vello font creation error");

        let mut items = vec![];
        let mut pen_x = 0f64;
        let mut pen_y = 0f64;

        let fello_size = vello::fello::Size::new(size);
        let charmap = font.charmap();
        let metrics = font.metrics(fello_size, Default::default());
        let line_height = metrics.ascent - metrics.descent + metrics.leading;
        let glyph_metrics = font.glyph_metrics(fello_size, Default::default());
        let vars: [(&str, f32); 0] = [];
        let mut provider = self.gcx.new_provider(&font, None, size, false, vars);

        for ch in text.chars() {
            if ch == '\n' {
                pen_y += line_height as f64;
                pen_x = 0.0;
                continue;
            }
            let gid = charmap.map(ch).unwrap_or_default();
            let advance = glyph_metrics.advance_width(gid).unwrap_or_default() as f64;

            if let Some(glyph) = provider.get(gid.to_u16(), brush) {
                let xform = transform
                    * Affine::translate((pen_x, 0.0))
                    * Affine::scale_non_uniform(1.0, -1.0);
                // builder.append(&glyph, Some(xform));
                items.push((glyph, xform));
            }
            pen_x += advance;
        }
        (items, pen_x, pen_y)
    }
}
#[derive(Default)]
pub struct VelloFontLoader;

impl AssetLoader for VelloFontLoader {
    type Asset = VelloFont;

    type Settings = ();

    type Error = VectorLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let vello_font = VelloFont::new(bytes.to_vec());

            Ok(vello_font)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["vttf"]
    }
}
