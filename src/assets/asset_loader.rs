use crate::assets::parser::{load_lottie_from_bytes, load_svg_from_bytes};
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
            let extension = load_context
                .path()
                .extension()
                .ok_or(bevy::asset::Error::msg(
                    "Invalid vello vector asset file extension",
                ))?;
            debug!("parsing {}", load_context.path().display());

            match extension.to_str() {
                Some("svg") => {
                    // Deserialize the SVG source XML string from the file
                    // contents buffer
                    let vello_vector = load_svg_from_bytes(bytes)?;

                    info!(
                        path = format!("{}", load_context.path().display()),
                        size = format!("{:?}", (vello_vector.width, vello_vector.height)),
                        "finished parsing svg asset"
                    );

                    load_context.set_default_asset(LoadedAsset::new(vello_vector));
                }
                Some("json") => {
                    let vello_vector = load_lottie_from_bytes(bytes)?;

                    info!(
                        path = format!("{}", load_context.path().display()),
                        size = format!("{:?}", (vello_vector.width, vello_vector.height)),
                        "finished parsing json asset"
                    );
                    load_context.set_default_asset(LoadedAsset::new(vello_vector))
                }
                _ => {}
            }

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["svg", "json"]
    }
}
