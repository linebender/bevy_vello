use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
    tasks::ConditionalSendFuture,
};

use super::asset::VelloSvg;
use crate::integrations::{VectorLoaderError, svg::load_svg_from_bytes};

#[derive(Default, TypePath)]
pub struct VelloSvgLoader;

impl AssetLoader for VelloSvgLoader {
    type Asset = VelloSvg;

    type Settings = ();

    type Error = VectorLoaderError;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let path = load_context.path().to_owned();
            let ext =
                path.get_full_extension()
                    .ok_or(VectorLoaderError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid file extension",
                    )))?;
            tracing::debug!("parsing {path}...");

            if ext.ends_with("svg") {
                let asset = load_svg_from_bytes(&bytes)?;
                tracing::info!(
                    path = %path,
                    size = format!("{:?}", (asset.width, asset.height)),
                    "finished parsing svg asset"
                );
                Ok(asset)
            } else {
                Err(VectorLoaderError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid file extension: '{ext}'"),
                )))
            }
        })
    }

    fn extensions(&self) -> &[&str] {
        &["svg"]
    }
}
