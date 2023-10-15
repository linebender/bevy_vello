use crate::{
    assets::parser::{load_lottie_from_bytes, load_svg_from_bytes},
    compression,
};
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    utils::BoxedFuture,
};
#[derive(Default)]
pub struct VelloVectorLoader;

impl AssetLoader for VelloVectorLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let mut bytes = bytes.to_vec();
            let path = load_context.path().to_owned();
            let mut ext = path
                .extension()
                .and_then(std::ffi::OsStr::to_str)
                .ok_or(bevy::asset::Error::msg("Invalid extension"))?
                .to_owned();

            let gzipped = ext == "gz";
            if gzipped {
                debug!("decompressing {}...", path.display());
                // Decompress
                let decrompressed_bytes = compression::decompress_gzip(&bytes);
                let path_without_gz = path.with_extension("");
                bytes = decrompressed_bytes.into_bytes();
                // Remove .gz extension
                ext = path_without_gz
                    .extension()
                    .and_then(std::ffi::OsStr::to_str)
                    .ok_or(bevy::asset::Error::msg("No extension before .gz?"))?
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

                    load_context.set_default_asset(LoadedAsset::new(vello_vector));
                }
                "json" => {
                    let vello_vector = load_lottie_from_bytes(&bytes)?;

                    info!(
                        path = format!("{}", load_context.path().display()),
                        size = format!("{:?}", (vello_vector.width, vello_vector.height)),
                        "finished parsing json asset"
                    );
                    load_context.set_default_asset(LoadedAsset::new(vello_vector));
                }
                _ => {}
            }

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["svg", "json", "svg.gz", "json.gz"]
    }
}
