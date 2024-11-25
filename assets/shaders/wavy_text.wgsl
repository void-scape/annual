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

// @vertex
// fn vertex(vertex: Vertex) -> VertexOutput {
//     var out: VertexOutput;
// 
//     out.uv = vertex.uv;
//     var world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
//     out.world_position = mesh_functions::mesh2d_position_local_to_world(
//         world_from_local,
//         vec4<f32>(vertex.position, 1.0)
//     );
//     out.position = mesh_functions::mesh2d_position_world_to_clip(out.world_position);
// 
//     return out;
// }

struct FullscreenVertexOutput {
    @builtin(position)
    position: vec4<f32>,
    @location(0)
    uv: vec2<f32>,
};

// This vertex shader produces the following, when drawn using indices 0..3:
//
//  1 |  0-----x.....2
//  0 |  |  s  |  . ´
// -1 |  x_____x´
// -2 |  :  .´
// -3 |  1´
//    +---------------
//      -1  0  1  2  3
//
// The axes are clip-space x and y. The region marked s is the visible region.
// The digits in the corners of the right-angled triangle are the vertex
// indices.
//
// The top-left has UV 0,0, the bottom-left has 0,2, and the top-right has 2,0.
// This means that the UV gets interpolated to 1,1 at the bottom-right corner
// of the clip-space rectangle that is at 1,-1 in clip space.
@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32) -> FullscreenVertexOutput {
    // See the explanation above for how this works
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
    // convert Vertex position <-1,+1> to texture coordinate <0,1> and some shrinking so the effect dont overlap screen
    let x1 = ( 0.55*p.x)+0.5;
    let y1 = (-0.55*p.y)+0.5;
    // wave distortion
    let tx = globals.time;
    let ty = globals.time * 0.5;
    // let x: f32 = sin( 25.0*y1 + 30.0*x1 + 6.28*tx) * 0.05;
    let x = 0.0;
    let y: f32 = sin( 10.0*y1 + 7.0*x1 + 6.28*ty) * 0.05;
    return vec2(p.x + x, p.y + y);
}
