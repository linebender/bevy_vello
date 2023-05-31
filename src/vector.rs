use crate::lyon_utils::{self, usvg_draw, Convert};
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    math::{Vec3A, Vec4Swizzles},
    prelude::*,
    reflect::TypeUuid,
    render::render_asset::RenderAsset,
    utils::BoxedFuture,
};
use lyon_tessellation::{FillTessellator, StrokeTessellator};
use std::{sync::Arc, time::Instant};
use vello::{SceneBuilder, SceneFragment};
use vello_svg::usvg;

#[derive(Clone)]
pub enum Vector {
    Static(Arc<SceneFragment>),
    Animated(velato::Composition),
}

#[derive(Clone)]
pub struct VectorAssetData {
    vector: Vector,
    local_transform: Transform,
}

#[derive(TypeUuid)]
#[uuid = "39cadc56-aa9c-4543-3640-a018b74b5053"]
pub struct VelloVector {
    pub data: Vector,
    pub local_transform: Transform,
    pub width: f32,
    pub height: f32,
    pub tessellation_mesh: Option<Mesh>,
}

impl VelloVector {
    /// Returns the 4 corner points of this vector's bounding box in world space
    pub fn bb_in_world(&self, transform: &GlobalTransform) -> [Vec2; 4] {
        let min = Vec3A::ZERO;
        let x_axis = Vec3A::new(self.width, 0.0, 0.0);

        let max = Vec3A::new(self.width, -self.height, 0.0);
        let y_axis = Vec3A::new(0.0, -self.height, 0.0);

        let world_transform = transform.compute_matrix();
        let local_transform = self.local_transform.compute_matrix().inverse();
        let min = (world_transform * local_transform * min.extend(1.0)).xy();
        let x_axis = (world_transform * local_transform * x_axis.extend(1.0)).xy();
        let max = (world_transform * local_transform * max.extend(1.0)).xy();
        let y_axis = (world_transform * local_transform * y_axis.extend(1.0)).xy();

        [min, x_axis, max, y_axis]
    }
}

impl RenderAsset for VelloVector {
    type ExtractedAsset = VectorAssetData;

    type PreparedAsset = RenderInstanceData;

    type Param = ();

    fn extract_asset(&self) -> Self::ExtractedAsset {
        VectorAssetData {
            vector: self.data.clone(),
            local_transform: self.local_transform,
        }
    }

    fn prepare_asset(
        data: Self::ExtractedAsset,
        _param: &mut bevy::ecs::system::SystemParamItem<Self::Param>,
    ) -> Result<
        Self::PreparedAsset,
        bevy::render::render_asset::PrepareAssetError<Self::ExtractedAsset>,
    > {
        Ok(data.into())
    }
}

#[derive(TypeUuid, Clone)]
#[uuid = "39cadc56-aa9c-4543-3640-a018b74b5054"]
pub struct RenderInstanceData {
    pub local_matrix: Mat4,
    pub data: Vector,
}

impl From<VectorAssetData> for RenderInstanceData {
    fn from(value: VectorAssetData) -> Self {
        let local_matrix = value.local_transform.compute_matrix().inverse();
        let vector_data = value.vector;

        RenderInstanceData {
            data: vector_data,
            local_matrix,
        }
    }
}

impl Default for RenderInstanceData {
    fn default() -> Self {
        Self {
            data: Vector::Static(Arc::new(SceneFragment::default())),
            local_matrix: Mat4::default(),
        }
    }
}

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
                        local_transform: compute_transform(width, height),
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
                            local_transform: compute_transform(width, height),
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

fn compute_transform(width: f32, height: f32) -> Transform {
    let mut transform = Transform::default();
    transform.translation.x = width / 2.0;
    transform.translation.y = -height;

    transform
}
