use crate::{
    assets::parser::{load_lottie_from_bytes, load_svg_from_bytes},
    compression, VelloVector,
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
pub struct VelloVectorLoader;

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

impl AssetLoader for VelloVectorLoader {
    type Asset = VelloVector;

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
            let mut ext = path
                .extension()
                .and_then(std::ffi::OsStr::to_str)
                .ok_or(VectorLoaderError::Parse("Invalid extension".to_string()))?
                .to_owned();

            let gzipped = ext == "gz";
            if gzipped {
                debug!("decompressing {}...", path.display());
                // Decompress
                let decrompressed_bytes = compression::decompress_gzip(&bytes)?;
                let path_without_gz = path.with_extension("");
                bytes = decrompressed_bytes.into_bytes();
                // Remove .gz extension
                ext = path_without_gz
                    .extension()
                    .and_then(std::ffi::OsStr::to_str)
                    .ok_or(VectorLoaderError::Parse(
                        "No extension before .gz?".to_string(),
                    ))?
                    .to_string();
            }

            debug!("parsing {}...", load_context.path().display());
            match ext.as_str() {
                "svg" => {
                    // Deserialize the SVG source XML string from the file
                    // contents buffer
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
                        "finished parsing json asset"
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
        &["svg", "json", "svg.gz", "json.gz"]
    }
}
