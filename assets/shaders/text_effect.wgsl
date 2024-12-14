#import bevy_sprite::{
    mesh2d_functions as mesh_functions,
    mesh2d_view_bindings::view,
}

#import bevy_render::globals::Globals
@group(0) @binding(1) var<uniform> globals: Globals;

struct Vertex {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(2) uv: vec2<f32>,
    // @location(5) atlas_uv: vec4<f32>,
};

struct UvRect {
    min: vec2<f32>,
    max: vec2<f32>,
}
@group(2) @binding(2) var<storage> storage_buffer: array<UvRect>;
@group(2) @binding(3) var<uniform> color: vec4<f32>;

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let uvs = storage_buffer[vertex.instance_index];

    // out.uv = vertex.uv;
    switch vertex.vertex_index % 4u {
        case 0u: { // Top Right
            out.color = vec4(1., 0., 0., 1.);
            out.uv = vec2(uvs.max.x, uvs.max.y);
            break;
        }
        case 1u: { // Top Left
            out.color = vec4(0., 1., 0., 1.);
            out.uv = vec2(uvs.min.x, uvs.max.y);
            break;
        }
        case 2u: { // Bottom Left
            out.color = vec4(0., 0., 1., 1.);
            out.uv = vec2(uvs.min.x, uvs.min.y);
            break;
        }
        case 3u: { // Bottom Right
            out.color = vec4(1., 1., 1., 1.);
            out.uv = vec2(uvs.max.x, uvs.min.y);
            break;
        }
        default: {
            out.color = vec4(0., 0., 0., 1.);
            out.uv = vertex.uv;
            break;
        }
    }

    out.color = vec4(f32(vertex.instance_index) / 10., 0., 0., 1.);

    // switch vertex.instance_index {
    //     case 0u: {
    //         out.color = vec4(1., 0., 0., 1.);
    //         break;
    //     }
    //     case 1u: {
    //         out.color = vec4(0., 1., 0., 1.);
    //         break;
    //     }
    //     case 2u: {
    //         out.color = vec4(0., 0., 1., 1.);
    //         break;
    //     }
    //     case 3u: {
    //         out.color = vec4(1., 1., 1., 1.);
    //         break;
    //     }
    //     case default: {
    //         out.color = vec4(0., 0., 0., 1.);
    //         break;
    //     }
    // }

    // let x = 1.0 - f32((vertex.vertex_index & 1u) == 0u);
    // let y = 1.0 - f32((vertex.vertex_index & 2u) == 0u);
    // out.uv = vec2(
    //     uvs.min.x * (1.0 - x) + uvs.max.x * x,
    //     uvs.min.y * (1.0 - y) + uvs.max.y * y
    // );

    var world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    let world_position = mesh_functions::mesh2d_position_local_to_world(
        world_from_local,
        vec4<f32>(vertex.position, 1.0)
    );
    out.position = mesh_functions::mesh2d_position_world_to_clip(world_position);
    out.position.y += sin(globals.time + out.position.x * 4.) * 0.05;

    // out.position.y += sin((globals.time + f32(vertex.instance_index)) * 32.) * 0.005;
    // out.position.x += sin((globals.time + f32(vertex.instance_index)) * 64.) * 0.005;

    return out;
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>
}

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var t_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, t_sampler, in.uv) * color;
    // return in.color;
    // return color;
}
