mod extract;
mod plugin;
mod prepare;
mod render;

pub use extract::SSRenderTarget;
pub use plugin::VelloRenderPlugin;

use bevy::{prelude::*, render::renderer::RenderDevice};
use vello::{Renderer, RendererOptions};

pub struct VelloRenderer(Renderer);

impl FromWorld for VelloRenderer {
    fn from_world(world: &mut World) -> Self {
        let device = world.get_resource::<RenderDevice>().unwrap();
        VelloRenderer(
            Renderer::new(
                device.wgpu_device(),
                RendererOptions {
                    surface_format: None,
                    timestamp_period: 0.0,
                    use_cpu: false,
                    antialiasing_support: vello::AaSupport {
                        area: true,
                        msaa8: false,
                        msaa16: false,
                    },
                },
            )
            .expect("no gpu device"),
        )
    }
}

#[derive(Resource)]
pub struct LottieRenderer(pub vellottie::Renderer);
