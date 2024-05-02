use crate::assets::parser::{load_lottie_from_bytes, load_svg_from_bytes};
use crate::VelloAsset;
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::*;
use bevy::utils::thiserror::{self, Error};
use bevy::utils::BoxedFuture;
#[derive(Default)]
pub struct VelloAssetLoader;

#[derive(Debug, Error)]
pub enum VectorLoaderError {
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse file: {0}")]
    Parse(String),
    #[error("Could not parse utf-8: {0}")]
    FromStrUtf8(#[from] std::str::Utf8Error),
    #[error("Could not parse svg: {0}")]
    Usvg(#[from] vello_svg::usvg::Error),
}

impl AssetLoader for VelloAssetLoader {
    type Asset = VelloAsset;

    type Settings = ();

    type Error = VectorLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let path = load_context.path().to_owned();
            let ext = path
                .extension()
                .and_then(std::ffi::OsStr::to_str)
                .ok_or(VectorLoaderError::Parse(
                    "Invalid file extension".to_string(),
                ))?
                .to_owned();

            debug!("parsing {}...", load_context.path().display());
            match ext.as_str() {
                "svg" => {
                    let vello_vector = load_svg_from_bytes(&bytes)?;
                    info!(
                        path = format!("{}", load_context.path().display()),
                        size = format!("{:?}", (vello_vector.width, vello_vector.height)),
                        "finished parsing svg asset"
                    );
                    Ok(vello_vector)
                }
                "json" => {
                    let vello_vector = load_lottie_from_bytes(&bytes)?;
                    info!(
                        path = format!("{}", load_context.path().display()),
                        size = format!("{:?}", (vello_vector.width, vello_vector.height)),
                        "finished parsing lottie json asset"
                    );
                    Ok(vello_vector)
                }
                ext => Err(VectorLoaderError::Parse(format!(
                    "Unknown file extension: {ext}"
                ))),
            }
        })
    }

    fn extensions(&self) -> &[&str] {
        &["svg", "json"]
    }
}
