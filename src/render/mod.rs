//! Components and logic for rendering.

use std::sync::{Arc, Mutex};

use bevy::{
    asset::weak_handle,
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        extract_resource::ExtractResource,
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
            VertexBufferLayout, VertexFormat, VertexStepMode,
        },
        renderer::RenderDevice,
        view::RenderLayers,
    },
    sprite::{Material2d, Material2dKey},
};
use vello::{AaConfig, AaSupport, kurbo::Affine};

mod plugin;
mod systems;

pub(crate) mod extract;
pub(crate) mod picking;
pub(crate) mod prepare;

pub(crate) use plugin::VelloRenderPlugin;

/// A handle to the screen space render target shader.
pub const SSRT_SHADER_HANDLE: Handle<Shader> = weak_handle!("e7235b72-1181-4e18-a9f2-93b32026a820");

/// A component that should be added to the camera that will render Vello assets.
#[derive(Component, Debug, Clone, Copy, ExtractComponent)]
pub struct VelloView;

/// A resource that holds the scale factor for Vello world coordinates.
#[derive(Resource, Clone)]
pub struct VelloWorldScale(pub f32);

impl Default for VelloWorldScale {
    fn default() -> Self {
        Self(1.0)
    }
}

impl ExtractResource for VelloWorldScale {
    type Source = VelloWorldScale;

    fn extract_resource(source: &Self::Source) -> Self {
        source.clone()
    }
}

/// A resource that holds the scale factor for Vello screen coordinates.
#[derive(Resource, Clone)]
pub struct VelloScreenScale(pub f32);

impl Default for VelloScreenScale {
    fn default() -> Self {
        Self(1.0)
    }
}

impl ExtractResource for VelloScreenScale {
    type Source = VelloScreenScale;

    fn extract_resource(source: &Self::Source) -> Self {
        source.clone()
    }
}

#[derive(Component, Debug, Clone)]
pub struct SkipScaling;

/// A canvas material, with a shader that samples a texture with view-independent UV coordinates.
#[derive(AsBindGroup, TypePath, Asset, Clone)]
pub struct VelloCanvasMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
}

impl Material2d for VelloCanvasMaterial {
    fn vertex_shader() -> ShaderRef {
        SSRT_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> ShaderRef {
        SSRT_SHADER_HANDLE.into()
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

/// Add this to any renderable vello asset to skip encoding that renderable.
#[derive(Component, Debug, Clone, Copy)]
pub struct SkipEncoding;

/// Internally used as a prepared render asset.
#[derive(Clone)]
pub(crate) enum VelloRenderItem {
    #[cfg(feature = "svg")]
    Svg {
        affine: Affine,
        item: crate::integrations::svg::render::ExtractedVelloSvg,
    },
    #[cfg(feature = "lottie")]
    Lottie {
        affine: Affine,
        item: crate::integrations::lottie::render::ExtractedLottieAsset,
    },
    Scene {
        affine: Affine,
        item: extract::ExtractedVelloScene,
    },
    #[cfg(feature = "text")]
    Text {
        affine: Affine,
        item: crate::integrations::text::render::ExtractedVelloText,
    },
}

/// Internally used to buffer sorted assets prepared for the next frame.
#[derive(Resource, Default, Deref, DerefMut)]
pub(crate) struct VelloRenderQueue(Vec<VelloRenderItem>);

/// Internally used for diagnostics.
#[derive(Resource, ExtractResource, Default, Debug, Clone, Reflect)]
pub(crate) struct VelloEntityCountData {
    /// Number of scenes.
    pub n_scenes: u32,
    /// Number of text sections.
    #[cfg(feature = "text")]
    pub n_texts: u32,
    /// Number of SVGs.
    #[cfg(feature = "svg")]
    pub n_svgs: u32,
    /// Number of Lotties.
    #[cfg(feature = "lottie")]
    pub n_lotties: u32,
}

/// Internally used for diagnostics.
#[derive(Resource, ExtractResource, Default, Debug, Clone, Reflect)]
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
