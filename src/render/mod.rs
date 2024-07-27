//! Components and logic for rendering.

use bevy::{
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
            VertexBufferLayout, VertexFormat, VertexStepMode,
        },
    },
    sprite::{Material2d, Material2dKey},
};

mod extract;
mod plugin;
mod prepare;
mod systems;

pub use plugin::VelloRenderPlugin;

/// A handle to the screen space render target shader.
pub const SSRT_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(2314894693238056781);

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

#[derive(Deref, DerefMut)]
pub struct VelloRenderer(vello::Renderer);

impl VelloRenderer {
    pub fn from_device(device: &vello::wgpu::Device) -> Self {
        let renderer = vello::Renderer::new(
            device,
            vello::RendererOptions {
                surface_format: None,
                use_cpu: false,
                antialiasing_support: vello::AaSupport::area_only(),
                num_init_threads: None,
            },
        )
        // TODO: Attempt CPU fallback. Support changing antialias settings.
        .expect("No GPU Device");
        VelloRenderer(renderer)
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
#[cfg(feature = "lottie")]
pub struct VelatoRenderer(velato::Renderer);
