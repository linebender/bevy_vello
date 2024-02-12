use super::font::VelloFont;
use crate::assets::VectorLoaderError;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    utils::BoxedFuture,
};

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
