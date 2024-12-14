use bevy::{
    prelude::*,
    reflect::TypePath,
    render::{
        render_resource::{AsBindGroup, ShaderRef},
        storage::ShaderStorageBuffer,
    },
    sprite::{AlphaMode2d, Material2d},
};
use bytemuck::{Pod, Zeroable};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WaveMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[storage(2, read_only)]
    pub atlas_uvs: Handle<ShaderStorageBuffer>,
    #[uniform(3)]
    pub color: LinearRgba,
}

impl Material2d for WaveMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/text_effect.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/text_effect.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct UvRect {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

impl From<bevy::prelude::Rect> for UvRect {
    fn from(value: bevy::prelude::Rect) -> Self {
        Self {
            min: value.min.to_array(),
            max: value.max.to_array(),
        }
    }
}
