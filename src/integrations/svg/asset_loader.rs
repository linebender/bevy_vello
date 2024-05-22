use crate::integrations::svg::load_svg_from_bytes;
use crate::integrations::VectorLoaderError;
use crate::VelloAsset;
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::*;
use bevy::utils::BoxedFuture;

#[derive(Default)]
pub struct VelloSvgLoader;

impl AssetLoader for VelloSvgLoader {
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
            let ext =
                path.extension()
                    .and_then(std::ffi::OsStr::to_str)
                    .ok_or(VectorLoaderError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid file extension",
                    )))?;

            debug!("parsing {}...", load_context.path().display());
            match ext {
                "svg" => {
                    let vello_vector = load_svg_from_bytes(&bytes)?;
                    info!(
                        path = format!("{}", load_context.path().display()),
                        size = format!("{:?}", (vello_vector.width, vello_vector.height)),
                        "finished parsing svg asset"
                    );
                    Ok(vello_vector)
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
