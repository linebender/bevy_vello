#import bevy_render::view

@group(0) @binding(0)
var<uniform> view: View;
@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;

// returns the (0-1, 0-1) position within the given viewport for the current buffer coords .
// buffer coords can be obtained from `@builtin(position).xy`.
// the view uniform struct contains the current camera viewport in `view.viewport`.
// topleft = 0,0
fn coords_to_viewport_uv(position: vec2<f32>, viewport: vec4<f32>) -> vec2<f32> {
    return (position - viewport.xy) / viewport.zw;
}

struct Vertex {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let position = vertex.position;
    var out: VertexOutput;
    out.clip_position = vec4<f32>(position, 1.0);

    return out;
}

fn sRGB_OETF(a: f32) -> f32 {
    if .04045f < a {
        return pow((a + .055f) / 1.055f, 2.4f);
    } else {
        return  a / 12.92f;
    }
}

fn linear_from_srgba(srgba: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
        sRGB_OETF(srgba.r),
        sRGB_OETF(srgba.g),
        sRGB_OETF(srgba.b),
        srgba.a);
}

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    let uvs = coords_to_viewport_uv(position.xy, view.viewport);
    let color = textureSample(texture, texture_sampler, uvs);
    let color_converted = linear_from_srgba(color);
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}
