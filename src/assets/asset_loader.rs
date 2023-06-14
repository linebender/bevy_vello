use crate::{
    assets::vector::Vector,
    lyon_utils::{self, usvg_draw, Convert},
    VelloVector,
};
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    utils::BoxedFuture,
};
use lyon_tessellation::{FillTessellator, StrokeTessellator};
use std::{sync::Arc, time::Instant};
use vello::{SceneBuilder, SceneFragment};
use vello_svg::usvg;

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

            match extension.to_str() {
                Some("svg") => {
                    // Deserialize the SVG source XML string from the file
                    // contents buffer
                    let svg_str = std::str::from_utf8(bytes)?;

                    // Load SVG XML String with PicoSVG Parser
                    let start = Instant::now();
                    debug!("parsing {}", load_context.path().display());
                    let usvg = usvg::Tree::from_str(svg_str, &usvg::Options::default())?;
                    let fin = start.elapsed();

                    // Process the loaded SVG into Vello-compatible data
                    let mut scene_frag = SceneFragment::new();
                    let mut builder = SceneBuilder::for_fragment(&mut scene_frag);
                    vello_svg::render_tree(&mut builder, &usvg);

                    let lyon_svg = usvg_draw::Svg::from_tree(&usvg);
                    let tessellation_mesh_buffer = lyon_utils::generate_buffer(
                        &lyon_svg,
                        &mut FillTessellator::new(),
                        &mut StrokeTessellator::new(),
                    );

                    let tessellation_mesh: Mesh = tessellation_mesh_buffer.convert();

                    let width = usvg.size.width() as f32;
                    let height = usvg.size.height() as f32;

                    let vello_vector = VelloVector {
                        data: Vector::Static(Arc::new(scene_frag)),
                        local_transform: compute_local_transform(width, height),
                        width,
                        height,
                        tessellation_mesh: Some(tessellation_mesh),
                    };

                    info!(
                        path = format!("{}", load_context.path().display()),
                        size = format!("{:?}", (vello_vector.width, vello_vector.height)),
                        parsing = format!("{fin:?}"),
                        "finished parsing svg asset"
                    );
                    load_context.set_default_asset(LoadedAsset::new(vello_vector));
                }
                Some("json") => {
                    let start = Instant::now();

                    if let Ok(composition) = velato::Composition::from_bytes(bytes) {
                        let fin = start.elapsed();

                        let width = composition.width as f32;
                        let height = composition.height as f32;

                        let vello_vector = VelloVector {
                            data: Vector::Animated(composition),
                            local_transform: compute_local_transform(width, height),
                            width,
                            height,
                            tessellation_mesh: None,
                        };

                        info!(
                            path = format!("{}", load_context.path().display()),
                            size = format!("{:?}", (vello_vector.width, vello_vector.height)),
                            parsing = format!("{fin:?}"),
                            "finished parsing json asset"
                        );
                        load_context.set_default_asset(LoadedAsset::new(vello_vector))
                    } else {
                        let comp = velato::Composition::from_bytes(bytes).unwrap_err();
                        error!("{:?}", comp);
                        error!("Invalid lottie file");
                    }
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

fn compute_local_transform(width: f32, height: f32) -> Transform {
    let mut transform = Transform::default();
    transform.translation.x = width / 2.0;
    transform.translation.y = -height;

    transform
}
