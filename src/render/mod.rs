//! Components and logic for rendering.

use bevy::prelude::*;
use bevy::render::mesh::MeshVertexBufferLayout;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
    VertexBufferLayout, VertexFormat, VertexStepMode,
};
use bevy::sprite::{Material2d, Material2dKey};

mod extract;
mod plugin;
mod prepare;
mod systems;
mod z_function;

pub use plugin::VelloRenderPlugin;
pub use z_function::ZFunction;

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
        _layout: &MeshVertexBufferLayout,
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

#[derive(Resource, Deref, DerefMut)]
#[cfg(feature = "lottie")]
pub struct VelatoRenderer(velato::Renderer);

#[cfg(feature = "lottie")]
impl Default for VelatoRenderer {
    fn default() -> Self {
        // TODO: Velato should have a ::default()
        Self(velato::Renderer::new())
    }
}
