use super::{context::get_global_font_context, font::VelloFont};
use crate::{integrations::VectorLoaderError, text::context::LOCAL_FONT_CONTEXT};
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    log::warn,
};

#[derive(Default)]
pub struct VelloFontLoader;

impl AssetLoader for VelloFontLoader {
    type Asset = VelloFont;

    type Settings = ();

    type Error = VectorLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        LOCAL_FONT_CONTEXT.with_borrow_mut(|font_context| {
            if font_context.is_none() {
                *font_context = Some(get_global_font_context().clone());
            }
            let font_context = font_context.as_mut().unwrap();
            let registered_fonts = font_context.collection.register_fonts(bytes.clone());
            let maybe_font = registered_fonts.first();
            if maybe_font.is_none() {
                warn!("Failed to register default font");
            }
            let (family_id, _font_info_vec) = maybe_font.unwrap();
            let family_name = font_context.collection.family_name(*family_id).unwrap();
            let vello_font = VelloFont {
                family_name: family_name.to_string(),
                bytes,
            };

            Ok(vello_font)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ttf"]
    }
}
