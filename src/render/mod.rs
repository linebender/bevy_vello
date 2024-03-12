use bevy::prelude::*;
use bevy::render::mesh::MeshVertexBufferLayout;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
    VertexBufferLayout, VertexFormat, VertexStepMode,
};
use bevy::render::renderer::RenderDevice;
use bevy::sprite::{Material2d, Material2dKey};
use vello::{Renderer, RendererOptions};

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

pub struct BevyVelloRenderer(Renderer);

impl FromWorld for BevyVelloRenderer {
    fn from_world(world: &mut World) -> Self {
        let device = world.get_resource::<RenderDevice>().unwrap();
        BevyVelloRenderer(
            Renderer::new(
                device.wgpu_device(),
                RendererOptions {
                    surface_format: None,
                    use_cpu: false,
                    antialiasing_support: vello::AaSupport {
                        area: true,
                        msaa8: false,
                        msaa16: false,
                    },
                    num_init_threads: None,
                },
            )
            .expect("no gpu device"),
        )
    }
}

#[derive(Resource)]
pub struct LottieRenderer(velato::Renderer);

impl Default for LottieRenderer {
    fn default() -> Self {
        Self(velato::Renderer::new())
    }
}
