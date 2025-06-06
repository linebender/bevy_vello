use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
    tasks::ConditionalSendFuture,
};

use super::asset::VelloSvg;
use crate::integrations::{VectorLoaderError, svg::load_svg_from_bytes};

#[derive(Default)]
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
                path.extension()
                    .and_then(std::ffi::OsStr::to_str)
                    .ok_or(VectorLoaderError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid file extension",
                    )))?;

            tracing::debug!("parsing {}...", load_context.path().display());
            match ext {
                "svg" => {
                    let asset = load_svg_from_bytes(&bytes)?;
                    tracing::info!(
                        path = format!("{}", load_context.path().display()),
                        size = format!("{:?}", (asset.width, asset.height)),
                        "finished parsing svg asset"
                    );
                    Ok(asset)
                }
                ext => Err(VectorLoaderError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid file extension: '{ext}'"),
                ))),
            }
        })
    }

    fn extensions(&self) -> &[&str] {
        &["svg"]
    }
}
