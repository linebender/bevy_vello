use crate::{
    assets::parser::{load_lottie_from_bytes, load_svg_from_bytes},
    VelloAsset,
};
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    utils::{
        thiserror::{self, Error},
        BoxedFuture,
    },
};
#[derive(Default)]
pub struct VelloAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum VectorLoaderError {
    #[error("Could not load vector: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse vector: {0}")]
    Parse(String),
    #[error("Could not parse shader: {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error("Could not parse shader: {0}")]
    FromStrUtf8(#[from] std::str::Utf8Error),
    #[error("Could not parse shader: {0}")]
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
                    "Invalid extension".to_string(),
                ))?
                .to_owned();

            debug!("parsing {}...", load_context.path().display());
            match ext.as_str() {
                "svg" => {
                    let vello_vector = load_svg_from_bytes(&bytes)?;
                    info!(
                        path = format!("{}", load_context.path().display()),
                        size = format!(
                            "{:?}",
                            (vello_vector.width, vello_vector.height)
                        ),
                        "finished parsing svg asset"
                    );
                    Ok(vello_vector)
                }
                "json" => {
                    let vello_vector = load_lottie_from_bytes(&bytes)?;
                    info!(
                        path = format!("{}", load_context.path().display()),
                        size = format!(
                            "{:?}",
                            (vello_vector.width, vello_vector.height)
                        ),
                        "finished parsing lottie json asset"
                    );
                    Ok(vello_vector)
                }
                _ => Err(VectorLoaderError::Parse(
                    "Unknown file extension".to_string(),
                )),
            }
        })
    }

    fn extensions(&self) -> &[&str] {
        &["svg", "json"]
    }
}
