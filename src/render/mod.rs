//! Components and logic for rendering.

use std::sync::{Arc, Mutex};

use bevy::{
    asset::uuid_handle,
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
pub(crate) mod systems;

pub(crate) mod extract;

pub(crate) use plugin::VelloRenderPlugin;

pub mod diagnostics;

/// A handle to the screen space render target shader.
pub const RT_SHADER_HANDLE: Handle<Shader> = uuid_handle!("e7235b72-1181-4e18-a9f2-93b32026a820");

/// A component that should be added to the camera that will render Vello assets.
#[derive(Component, Debug, Clone, Copy, ExtractComponent)]
#[require(Camera2d)]
pub struct VelloView;

/// Internal marker indicating the render target was auto-created by
/// [`manage_render_targets`](systems::manage_render_targets). When absent (i.e. the
/// user provided their own [`RenderTarget::Image`](bevy::camera::RenderTarget::Image)),
/// the system redirects the Camera2d pipeline to a dummy image so it doesn't
/// interfere with other cameras rendering to the primary window.
#[derive(Component)]
pub(crate) struct VelloAutoSpawned;

/// Marker component for a [`Sprite`] that displays a Vello render target.
///
/// When added alongside a `Sprite`, this component automatically keeps the
/// sprite's `custom_size` in sync with the primary window's logical dimensions,
/// ensuring the render target fills the camera view at full physical resolution.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct VelloCanvas;

/// A canvas material, with a shader that samples a texture with view-independent UV coordinates.
/// This can be used as an opt-in display method for Vello render targets.
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
        item: crate::integrations::scene::render::ExtractedUiVelloScene,
    },
    #[cfg(feature = "svg")]
    Svg {
        affine: Affine,
        item: crate::integrations::svg::render::ExtractedUiVelloSvg,
    },
    #[cfg(feature = "lottie")]
    Lottie {
        affine: Affine,
        item: crate::integrations::lottie::render::ExtractedUiVelloLottie,
    },
    #[cfg(feature = "text")]
    Text {
        affine: Affine,
        item: crate::integrations::text::render::ExtractedUiVelloText,
    },
}

/// Per-camera render queue containing sorted render items.
#[derive(Default)]
pub(crate) struct PerCameraRenderQueue {
    pub world: Vec<VelloWorldRenderItem>,
    pub ui: Vec<VelloUiRenderItem>,
}

/// Per-camera render queues, keyed by camera entity in the render world.
#[derive(Resource, Default)]
pub(crate) struct VelloRenderQueues {
    pub cameras: std::collections::HashMap<Entity, PerCameraRenderQueue>,
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
