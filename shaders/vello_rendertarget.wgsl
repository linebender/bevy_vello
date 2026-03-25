#import bevy_render::view::{View, frag_coord_to_uv}
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(0) @binding(0)
var<uniform> view: View;
@group(2) @binding(0)
var texture: texture_2d<f32>;
@group(2) @binding(1)
var texture_sampler: sampler;

struct Vertex {
    @location(0) position: vec3<f32>,
};

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    var out: VertexOutput;
    // This is the `clip position`.
    out.position = vec4<f32>(in.position, 1.0);
    return out;
}

fn linear_from_srgba(srgba: vec4<f32>) -> vec4<f32> {
    return vec4(
        select(
            srgba.rgb / 12.92,
            pow((srgba.rgb + .055) / 1.055, vec3(2.4)),
            srgba.rgb > vec3(0.04045)
        ),
        srgba.a,
    );
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = frag_coord_to_uv(in.position.xy, view.viewport);
    let color = textureSample(texture, texture_sampler, uv);
    let color_converted = linear_from_srgba(color);
    return color_converted;
}
