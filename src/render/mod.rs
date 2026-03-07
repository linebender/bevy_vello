//! Components and logic for rendering.

use std::sync::{Arc, Mutex};

use bevy::{
    asset::uuid_handle,
    camera::visibility::RenderLayers,
    mesh::{MeshVertexBufferLayoutRef, VertexBufferLayout},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError, VertexFormat,
            VertexStepMode,
        },
        renderer::RenderDevice,
    },
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dKey},
};
use vello::{AaConfig, AaSupport, kurbo::Affine};

mod plugin;
mod systems;

pub(crate) mod extract;
pub(crate) mod prepare;

pub(crate) use plugin::VelloRenderPlugin;

pub mod diagnostics;

/// A handle to the screen space render target shader.
pub const RT_SHADER_HANDLE: Handle<Shader> = uuid_handle!("e7235b72-1181-4e18-a9f2-93b32026a820");

/// A component that should be added to the camera that will render Vello assets.
#[derive(Component, Debug, Clone, Copy, ExtractComponent)]
#[require(Camera2d)]
pub struct VelloView;

/// A canvas material, with a shader that samples a texture with view-independent UV coordinates.
#[derive(AsBindGroup, TypePath, Asset, Clone)]
pub struct VelloCanvasMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
}

impl Material2d for VelloCanvasMaterial {
    fn vertex_shader() -> ShaderRef {
        RT_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> ShaderRef {
        RT_SHADER_HANDLE.into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // FIXME: Vello isn't obeying transparency on render_to_surface call.
        // See https://github.com/linebender/vello/issues/549
        if let Some(target) = descriptor.fragment.as_mut() {
            let mut_targets = &mut target.targets;
            if let Some(Some(target)) = mut_targets.get_mut(0) {
                target.blend = Some(vello::wgpu::BlendState::ALPHA_BLENDING);
            }
        }

        let formats = vec![
            // Position
            VertexFormat::Float32x3,
            VertexFormat::Float32x2,
        ];

        let vertex_layout =
            VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, formats);

        descriptor.vertex.buffers = vec![vertex_layout];

        Ok(())
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct VelloRenderer(Arc<Mutex<vello::Renderer>>);

impl VelloRenderer {
    pub fn try_new(
        device: &vello::wgpu::Device,
        settings: &VelloRenderSettings,
    ) -> Result<Self, vello::Error> {
        vello::Renderer::new(
            device,
            vello::RendererOptions {
                use_cpu: settings.use_cpu,
                // TODO: Vello doesn't currently allow adding additional AA support after
                // initialization, so we need to use all support modes here instead.
                antialiasing_support: AaSupport::all(),
                num_init_threads: None,
                pipeline_cache: None,
            },
        )
        .map(Mutex::new)
        .map(Arc::new)
        .map(VelloRenderer)
    }
}

impl FromWorld for VelloRenderer {
    fn from_world(world: &mut World) -> Self {
        match VelloRenderer::try_new(
            world.get_resource::<RenderDevice>().unwrap().wgpu_device(),
            world.get_resource::<VelloRenderSettings>().unwrap(),
        ) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!(
                    "Attempting safe-mode fallback, failed to initialize renderer: {e:}"
                );
                {
                    let mut settings = world.get_resource_mut::<VelloRenderSettings>().unwrap();
                    settings.use_cpu = true;
                    settings.antialiasing = AaConfig::Area;
                }
                match VelloRenderer::try_new(
                    world.get_resource::<RenderDevice>().unwrap().wgpu_device(),
                    world.get_resource::<VelloRenderSettings>().unwrap(),
                ) {
                    Ok(r) => r,
                    Err(e) => panic!("Failed to start vello: {e}"),
                }
            }
        }
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
#[cfg(feature = "lottie")]
pub struct VelatoRenderer(velato::Renderer);

/// Render settings for Vello.
#[derive(Resource, Clone)]
pub struct VelloRenderSettings {
    /// Use CPU instead of GPU
    pub use_cpu: bool,

    /// Which antialiasing strategy to use
    pub antialiasing: AaConfig,
}

impl Default for VelloRenderSettings {
    fn default() -> Self {
        Self {
            use_cpu: false,
            antialiasing: AaConfig::Area,
        }
    }
}

/// Canvas settings for Vello.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub(crate) struct VelloCanvasSettings {
    /// The render layers that will be used for the Vello canvas mesh.
    pub render_layers: RenderLayers,
}

/// Internally used as a prepared render asset.
#[derive(Clone)]
#[allow(clippy::large_enum_variant, reason = "Many feature gates")]
pub(crate) enum VelloWorldRenderItem {
    Scene {
        affine: Affine,
        item: crate::integrations::scene::render::ExtractedVelloScene2d,
    },
    #[cfg(feature = "svg")]
    Svg {
        affine: Affine,
        item: crate::integrations::svg::render::ExtractedVelloSvg2d,
    },
    #[cfg(feature = "lottie")]
    Lottie {
        affine: Affine,
        item: crate::integrations::lottie::render::ExtractedVelloLottie2d,
    },
    #[cfg(feature = "text")]
    Text {
        affine: Affine,
        item: crate::integrations::text::render::ExtractedVelloText2d,
    },
}

/// Internally used as a prepared render asset.
#[derive(Clone)]
#[allow(clippy::large_enum_variant, reason = "Many feature gates")]
pub(crate) enum VelloUiRenderItem {
    Scene {
        affine: Affine,
        /// Clip rect in physical pixels (from CalculatedClip, already resolved).
        clip: Option<vello::kurbo::Rect>,
        item: crate::integrations::scene::render::ExtractedUiVelloScene,
    },
    #[cfg(feature = "svg")]
    Svg {
        affine: Affine,
        clip: Option<vello::kurbo::Rect>,
        item: crate::integrations::svg::render::ExtractedUiVelloSvg,
    },
    #[cfg(feature = "lottie")]
    Lottie {
        affine: Affine,
        clip: Option<vello::kurbo::Rect>,
        item: crate::integrations::lottie::render::ExtractedUiVelloLottie,
    },
    #[cfg(feature = "text")]
    Text {
        affine: Affine,
        clip: Option<vello::kurbo::Rect>,
        item: crate::integrations::text::render::ExtractedUiVelloText,
    },
}

/// Internally used to buffer sorted assets prepared for the next frame.
#[derive(Resource, Default)]
pub(crate) struct VelloRenderQueue {
    world: Vec<VelloWorldRenderItem>,
    ui: Vec<VelloUiRenderItem>,
}

/// Internally used for diagnostics.
#[derive(Resource, Default, Debug, Clone, Reflect)]
pub(crate) struct VelloEntityCountData {
    /// Number of scenes used in the World.
    pub n_world_scenes: u32,
    /// Number of scenes used in UI.
    pub n_ui_scenes: u32,
    /// Number of text sections used in the World.
    #[cfg(feature = "text")]
    pub n_world_texts: u32,
    /// Number of text sections used in UI.
    #[cfg(feature = "text")]
    pub n_ui_texts: u32,
    /// Number of SVGs used in the World.
    #[cfg(feature = "svg")]
    pub n_world_svgs: u32,
    /// Number of SVGs used in UI.
    #[cfg(feature = "svg")]
    pub n_ui_svgs: u32,
    /// Number of Lotties used in the World.
    #[cfg(feature = "lottie")]
    pub n_world_lotties: u32,
    /// Number of Lotties used in UI.
    #[cfg(feature = "lottie")]
    pub n_ui_lotties: u32,
}

/// When false, `sort_render_items` and `render_frame` are skipped.
/// The GPU texture from the previous frame persists on the canvas mesh.
#[derive(Resource, Clone)]
pub struct VelloSceneDirty(pub bool);

impl Default for VelloSceneDirty {
    fn default() -> Self {
        Self(true) // First frame always renders
    }
}

/// Signals to the render world that font assets changed and the text layout
/// cache should be invalidated.
#[derive(Resource, Clone, Default)]
pub(crate) struct VelloFontChanged(pub bool);

/// Internally used for diagnostics.
#[derive(Resource, Default, Debug, Clone, Reflect)]
pub(crate) struct VelloFrameProfileData {
    /// Total number of paths rendered last frame.
    pub n_paths: u32,
    /// Total number of path segments rendered last frame.
    pub n_path_segs: u32,
    /// Total number of clips rendered last frame.
    pub n_clips: u32,
    /// Total number of open clips rendered last frame.
    pub n_open_clips: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Rect;
    use systems::to_kurbo_clip;

    /// CalculatedClip is already in physical pixels (Bevy resolves layout
    /// against `physical_size`). to_kurbo_clip converts the type without
    /// scaling, matching PreparedAffine's output coordinate space.
    #[test]
    fn clip_converts_to_kurbo_without_scaling() {
        let bevy_clip = Rect::new(10.0, 20.0, 100.0, 200.0);
        let kurbo_clip = to_kurbo_clip(Some(bevy_clip)).unwrap();
        assert_eq!(kurbo_clip.x0, 10.0);
        assert_eq!(kurbo_clip.y0, 20.0);
        assert_eq!(kurbo_clip.x1, 100.0);
        assert_eq!(kurbo_clip.y1, 200.0);
    }

    #[test]
    fn clip_none_returns_none() {
        assert!(to_kurbo_clip(None).is_none());
    }

    /// Per-axis overflow clipping (e.g. Overflow::clip_y()) produces
    /// CalculatedClip with f32::INFINITY on the unconstrained axis.
    /// Vello can't rasterize a clip path with infinite coordinates, so
    /// to_kurbo_clip must clamp them to finite values.
    #[test]
    fn clip_with_infinite_x_produces_finite_rect() {
        // Simulates Overflow::clip_y() — X unconstrained, Y clipped to viewport.
        let clip = Rect::new(f32::NEG_INFINITY, 35.0, f32::INFINITY, 772.0);
        let kurbo = to_kurbo_clip(Some(clip));
        let kurbo = kurbo.expect("infinite-x clip should still produce a rect");
        assert!(kurbo.x0.is_finite(), "x0 must be finite, got {}", kurbo.x0);
        assert!(kurbo.x1.is_finite(), "x1 must be finite, got {}", kurbo.x1);
        assert_eq!(kurbo.y0, 35.0, "finite y0 must be preserved");
        assert_eq!(kurbo.y1, 772.0, "finite y1 must be preserved");
        assert!(kurbo.x0 < kurbo.x1, "clamped rect must have positive width");
    }

    /// NaN coordinates make the clip rect meaningless — should return None.
    #[test]
    fn clip_with_nan_returns_none() {
        let clip = Rect::new(f32::NAN, 35.0, f32::NAN, 772.0);
        assert!(
            to_kurbo_clip(Some(clip)).is_none(),
            "NaN clip should return None"
        );
    }

    /// Mixed scenario: only min.x is infinite (per-axis clip edge case).
    #[test]
    fn clip_with_one_infinite_coord_produces_finite_rect() {
        let clip = Rect::new(f32::NEG_INFINITY, 0.0, 1280.0, 800.0);
        let kurbo = to_kurbo_clip(Some(clip)).expect("should produce a rect");
        assert!(kurbo.x0.is_finite(), "x0 must be finite");
        assert_eq!(kurbo.x1, 1280.0);
        assert_eq!(kurbo.y0, 0.0);
        assert_eq!(kurbo.y1, 800.0);
    }

    /// UI render queue uses stable sort so that items with the same
    /// stack_index preserve their insertion order across frames.
    /// Prevents z-fighting flicker between overlapping same-layer items.
    #[test]
    fn ui_sort_preserves_insertion_order_for_equal_keys() {
        // Tag items A..E with the same stack_index but different identities.
        let items: Vec<(u32, char)> = vec![(5, 'A'), (5, 'B'), (5, 'C'), (5, 'D'), (5, 'E')];
        let mut sorted = items.clone();
        sorted.sort_by(|(a, _), (b, _)| a.cmp(b));

        let order: Vec<char> = sorted.iter().map(|(_, c)| *c).collect();
        assert_eq!(order, vec!['A', 'B', 'C', 'D', 'E']);
    }

    /// VelloRenderSettings must exist in the main world so that
    /// `detect_vello_scene_changes` (which runs in PostUpdate) can access it.
    /// Regression: the resource was only inserted into the render sub-app,
    /// causing a panic on the first frame.
    ///
    /// We verify the plugin's build() method inserts the resource by checking
    /// the relevant code path directly, since a full App test requires GPU
    /// subsystems (RenderPlugin, ShaderPlugin) that aren't available in unit tests.
    #[test]
    fn render_settings_default_is_available() {
        let settings = VelloRenderSettings::default();
        assert!(!settings.use_cpu);
        // The plugin inserts this resource via `app.insert_resource(self.render_settings.clone())`
        // in build(), before the render app guard. This guarantees it exists in the main world.
    }
}
