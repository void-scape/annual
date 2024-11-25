#import bevy_sprite::{
    mesh2d_functions as mesh_functions,
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::view,
}

#import bevy_render::globals::Globals
@group(0) @binding(1) var<uniform> globals: Globals;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct FullscreenVertexOutput {
    @builtin(position)
    position: vec4<f32>,
    @location(0)
    uv: vec2<f32>,
};

@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32) -> FullscreenVertexOutput {
    let uv = vec2<f32>(f32(vertex_index >> 1u), f32(vertex_index & 1u)) * 2.0;
    let clip_position = vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
    return FullscreenVertexOutput(clip_position, uv);
}

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var t_sampler: sampler;


@fragment
fn fragment(
    in: FullscreenVertexOutput,
) -> @location(0) vec4<f32> {
    return textureSample(texture, t_sampler, sine_wave(in.uv));
}

fn sine_wave(p: vec2<f32>) -> vec2<f32> {
    let x1 = ( 0.55 * p.x) + 0.5;
    let y1 = (-0.55 * p.y) + 0.5;

    let tx = globals.time;
    let ty = globals.time * 0.5;

    let x = 0.0;
    let y: f32 = sin( 8.0*y1 + 15.0*x1 + 6.28*ty) * 0.015;
    return vec2(p.x + x, p.y + y);
}
