use super::effect::EffectGlyph;
use super::text::{UvRect, WaveMaterial};
use bevy::app::{App, Last, Plugin, PostUpdate, Update};
use bevy::asset::{Asset, AssetApp, AssetId, AssetServer, Assets, Handle, RenderAssetUsages};
use bevy::core_pipeline::{
    core_2d::{AlphaMask2d, AlphaMask2dBinKey, Opaque2d, Opaque2dBinKey, Transparent2d},
    tonemapping::{DebandDither, Tonemapping},
};
use bevy::ecs::system::lifetimeless::SQuery;
use bevy::ecs::{
    prelude::*,
    system::{lifetimeless::SRes, SystemParamItem},
};
use bevy::math::FloatOrd;
use bevy::prelude::{Deref, DerefMut, Mesh2d};
use bevy::reflect::{prelude::ReflectDefault, Reflect};
use bevy::render::batching::gpu_preprocessing::batch_and_prepare_binned_render_phase;
use bevy::render::batching::GetBatchData;
use bevy::render::mesh::{VertexAttributeDescriptor, VertexBufferLayout};
use bevy::render::render_resource::{Buffer, VertexAttribute, VertexFormat, VertexStepMode};
use bevy::render::storage::ShaderStorageBuffer;
use bevy::render::sync_world::MainEntityHashMap;
use bevy::render::view::RenderVisibleEntities;
use bevy::render::{
    mesh::{MeshVertexBufferLayoutRef, RenderMesh},
    render_asset::{
        prepare_assets, PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssets,
    },
    render_phase::{
        AddRenderCommand, BinnedRenderPhaseType, DrawFunctions, PhaseItem, PhaseItemExtraIndex,
        RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
        ViewBinnedRenderPhases, ViewSortedRenderPhases,
    },
    render_resource::{
        AsBindGroup, AsBindGroupError, BindGroup, BindGroupLayout, OwnedBindingResource,
        PipelineCache, RenderPipelineDescriptor, Shader, ShaderRef, SpecializedMeshPipeline,
        SpecializedMeshPipelineError, SpecializedMeshPipelines,
    },
    renderer::RenderDevice,
    view::{ExtractedView, Msaa, ViewVisibility},
    Extract, ExtractSchedule, Render, RenderApp, RenderSet,
};
use bevy::sprite::{
    AlphaMode2d, DrawMesh2d, Material2dBindGroupId, Mesh2dPipeline, Mesh2dPipelineKey,
    RenderMesh2dInstances, SetMesh2dBindGroup, SetMesh2dViewBindGroup, TextureAtlasLayout,
};
use bevy::text::{TextLayoutInfo, Update2dText};
use bevy::utils::tracing::error;
use bevy_bits::text::{TextMod, TypeWriterSection};
use core::{hash::Hash, marker::PhantomData};

pub trait TextMaterial2d: AsBindGroup + Asset + Clone + Sized {
    /// Bind this material to a [`TextMod`].
    fn text_mod() -> TextMod;

    /// Returns this material's vertex shader. If [`ShaderRef::Default`] is returned, the default mesh vertex shader
    /// will be used.
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    /// Returns this material's fragment shader. If [`ShaderRef::Default`] is returned, the default mesh fragment shader
    /// will be used.
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }

    /// Add a bias to the view depth of the mesh which can be used to force a specific render order.
    #[inline]
    fn depth_bias(&self) -> f32 {
        0.0
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    /// Customizes the default [`RenderPipelineDescriptor`].
    #[allow(unused_variables)]
    #[inline]
    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        Ok(())
    }
}

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect, PartialEq, Eq)]
#[reflect(Component, Default)]
pub struct TextMeshMaterial2d<M: TextMaterial2d>(pub Handle<M>);

impl<M: TextMaterial2d> Default for TextMeshMaterial2d<M> {
    fn default() -> Self {
        Self(Handle::default())
    }
}

impl<M: TextMaterial2d> From<TextMeshMaterial2d<M>> for AssetId<M> {
    fn from(material: TextMeshMaterial2d<M>) -> Self {
        material.id()
    }
}

impl<M: TextMaterial2d> From<&TextMeshMaterial2d<M>> for AssetId<M> {
    fn from(material: &TextMeshMaterial2d<M>) -> Self {
        material.id()
    }
}

pub struct TextMaterial2dPlugin<M: TextMaterial2d>(PhantomData<M>);

impl<M: TextMaterial2d> Default for TextMaterial2dPlugin<M> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<M: TextMaterial2d> Plugin for TextMaterial2dPlugin<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        app.init_asset::<M>()
            .register_type::<TextMeshMaterial2d<M>>()
            .add_plugins(RenderAssetPlugin::<PreparedTextMaterial2d<M>>::default());

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Opaque2d, DrawTextMaterial2d<M>>()
                .add_render_command::<AlphaMask2d, DrawTextMaterial2d<M>>()
                .add_render_command::<Transparent2d, DrawTextMaterial2d<M>>()
                .init_resource::<RenderTextMaterial2dInstances<M>>()
                .init_resource::<SpecializedMeshPipelines<TextMaterial2dPipeline<M>>>()
                .add_systems(ExtractSchedule, extract_mesh_materials_2d::<M>)
                .add_systems(
                    Render,
                    queue_material2d_meshes::<M>
                        .in_set(RenderSet::QueueMeshes)
                        .after(prepare_assets::<PreparedTextMaterial2d<M>>),
                );
        }
    }

    fn finish(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<TextMaterial2dPipeline<M>>();
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct UpdateTextEffects;

// fn init_material<M: TextMaterial2d>(
//     mut commands: Commands,
//     sections: Query<(Entity, &TypeWriterSection, &TextLayoutInfo), Added<TypeWriterSection>>,
// ) {
//     for (entity, section, text_layout_info) in sections.iter() {
//         for effect in section.text.modifiers.iter() {
//             if effect.text_mod == M::text_mod() {
//                 // let mut atlas = None;
//                 // let mut texture = None;
//                 // let atlas_uvs = text_layout_info
//                 //     .glyphs
//                 //     .iter()
//                 //     .map(|g| {
//                 //         if atlas.is_none() {
//                 //             texture = Some(g.atlas_info.texture.clone());
//                 //             atlas = Some(texture_atlases.get(&g.atlas_info.texture_atlas).unwrap());
//                 //         }
//                 //
//                 //         let atlas = atlas.as_ref().unwrap();
//                 //         let rect: super::text::Rect = atlas.textures
//                 //             [g.atlas_info.location.glyph_index]
//                 //             .as_rect()
//                 //             .into();
//                 //         super::text::Rect {
//                 //             min: [
//                 //                 rect.min[0] / atlas.size.x as f32,
//                 //                 rect.min[1] / atlas.size.y as f32,
//                 //             ],
//                 //             max: [
//                 //                 rect.max[0] / atlas.size.x as f32,
//                 //                 rect.max[1] / atlas.size.y as f32,
//                 //             ],
//                 //         }
//                 //     })
//                 //     .collect::<Vec<_>>();
//                 //
//                 // println!("{atlas_uvs:?}");
//
//                 if !text_layout_info.glyphs.is_empty() {
//                     commands.entity(entity).insert(TextEffectInfo {
//                         atlas: text_layout_info
//                             .glyphs
//                             .iter()
//                             .map(|g| g.atlas_info.texture.clone())
//                             .next()
//                             .unwrap(),
//                         glyphs: Vec::with_capacity(text_layout_info.glyphs.len()),
//                         mods: section.text.modifiers.iter().map(|m| m.text_mod).collect(),
//                     });
//                 } else {
//                     unimplemented!()
//                 }
//             }
//         }
//     }
// }

#[derive(Resource, Deref, DerefMut)]
pub struct RenderTextMaterial2dInstances<M: TextMaterial2d>(MainEntityHashMap<AssetId<M>>);

impl<M: TextMaterial2d> Default for RenderTextMaterial2dInstances<M> {
    fn default() -> Self {
        Self(Default::default())
    }
}

fn extract_mesh_materials_2d<M: TextMaterial2d>(
    mut material_instances: ResMut<RenderTextMaterial2dInstances<M>>,
    query: Extract<Query<(Entity, &ViewVisibility, &TextMeshMaterial2d<M>), With<Mesh2d>>>,
) {
    material_instances.clear();

    for (entity, view_visibility, material) in &query {
        if view_visibility.get() {
            material_instances.insert(entity.into(), material.id());
        }
    }
}

#[derive(Resource)]
pub struct TextMaterial2dPipeline<M: TextMaterial2d> {
    pub mesh2d_pipeline: Mesh2dPipeline,
    pub material2d_layout: BindGroupLayout,
    pub vertex_shader: Option<Handle<Shader>>,
    pub fragment_shader: Option<Handle<Shader>>,
    marker: PhantomData<M>,
}

pub struct Material2dKey<M: TextMaterial2d> {
    pub mesh_key: Mesh2dPipelineKey,
    pub bind_group_data: M::Data,
}

impl<M: TextMaterial2d> Eq for Material2dKey<M> where M::Data: PartialEq {}

impl<M: TextMaterial2d> PartialEq for Material2dKey<M>
where
    M::Data: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.mesh_key == other.mesh_key && self.bind_group_data == other.bind_group_data
    }
}

impl<M: TextMaterial2d> Clone for Material2dKey<M>
where
    M::Data: Clone,
{
    fn clone(&self) -> Self {
        Self {
            mesh_key: self.mesh_key,
            bind_group_data: self.bind_group_data.clone(),
        }
    }
}

impl<M: TextMaterial2d> Hash for Material2dKey<M>
where
    M::Data: Hash,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.mesh_key.hash(state);
        self.bind_group_data.hash(state);
    }
}

impl<M: TextMaterial2d> Clone for TextMaterial2dPipeline<M> {
    fn clone(&self) -> Self {
        Self {
            mesh2d_pipeline: self.mesh2d_pipeline.clone(),
            material2d_layout: self.material2d_layout.clone(),
            vertex_shader: self.vertex_shader.clone(),
            fragment_shader: self.fragment_shader.clone(),
            marker: PhantomData,
        }
    }
}

impl<M: TextMaterial2d> SpecializedMeshPipeline for TextMaterial2dPipeline<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    type Key = Material2dKey<M>;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh2d_pipeline.specialize(key.mesh_key, layout)?;
        if let Some(vertex_shader) = &self.vertex_shader {
            descriptor.vertex.shader = vertex_shader.clone();
        }

        if let Some(fragment_shader) = &self.fragment_shader {
            descriptor.fragment.as_mut().unwrap().shader = fragment_shader.clone();
        }

        descriptor.layout = vec![
            self.mesh2d_pipeline.view_layout.clone(),
            self.mesh2d_pipeline.mesh_layout.clone(),
            self.material2d_layout.clone(),
        ];

        // descriptor.vertex.buffers.push(VertexBufferLayout {
        //     array_stride: std::mem::size_of::<UvRect>() as u64,
        //     step_mode: VertexStepMode::Instance,
        //     attributes: vec![VertexAttribute {
        //         format: VertexFormat::Float32x4,
        //         shader_location: 5,
        //         offset: 0,
        //     }],
        // });

        M::specialize(&mut descriptor, layout, key)?;
        Ok(descriptor)
    }
}

impl<M: TextMaterial2d> FromWorld for TextMaterial2dPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let render_device = world.resource::<RenderDevice>();
        let material2d_layout = M::bind_group_layout(render_device);

        TextMaterial2dPipeline {
            mesh2d_pipeline: world.resource::<Mesh2dPipeline>().clone(),
            material2d_layout,
            vertex_shader: match M::vertex_shader() {
                ShaderRef::Default => None,
                ShaderRef::Handle(handle) => Some(handle),
                ShaderRef::Path(path) => Some(asset_server.load(path)),
            },
            fragment_shader: match M::fragment_shader() {
                ShaderRef::Default => None,
                ShaderRef::Handle(handle) => Some(handle),
                ShaderRef::Path(path) => Some(asset_server.load(path)),
            },
            marker: PhantomData,
        }
    }
}

pub(super) type DrawTextMaterial2d<M> = (
    SetItemPipeline,
    SetMesh2dViewBindGroup<0>,
    SetMesh2dBindGroup<1>,
    SetMaterial2dBindGroup<M, 2>,
    DrawMesh2d,
);

pub struct SetMaterial2dBindGroup<M: TextMaterial2d, const I: usize>(PhantomData<M>);
impl<P: PhaseItem, M: TextMaterial2d, const I: usize> RenderCommand<P>
    for SetMaterial2dBindGroup<M, I>
{
    type Param = (
        SRes<RenderAssets<PreparedTextMaterial2d<M>>>,
        SRes<RenderTextMaterial2dInstances<M>>,
    );
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        _item_query: Option<()>,
        (materials, material_instances): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let materials = materials.into_inner();
        let material_instances = material_instances.into_inner();
        let Some(material_instance) = material_instances.get(&item.main_entity()) else {
            return RenderCommandResult::Skip;
        };
        let Some(material2d) = materials.get(*material_instance) else {
            return RenderCommandResult::Skip;
        };
        // pass.set_vertex_buffer(1, material2d.vertex_buffer.slice(..));
        pass.set_bind_group(I, &material2d.bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub const fn alpha_mode_pipeline_key(alpha_mode: AlphaMode2d) -> Mesh2dPipelineKey {
    match alpha_mode {
        AlphaMode2d::Blend => Mesh2dPipelineKey::BLEND_ALPHA,
        AlphaMode2d::Mask(_) => Mesh2dPipelineKey::MAY_DISCARD,
        _ => Mesh2dPipelineKey::NONE,
    }
}

pub const fn tonemapping_pipeline_key(tonemapping: Tonemapping) -> Mesh2dPipelineKey {
    match tonemapping {
        Tonemapping::None => Mesh2dPipelineKey::TONEMAP_METHOD_NONE,
        Tonemapping::Reinhard => Mesh2dPipelineKey::TONEMAP_METHOD_REINHARD,
        Tonemapping::ReinhardLuminance => Mesh2dPipelineKey::TONEMAP_METHOD_REINHARD_LUMINANCE,
        Tonemapping::AcesFitted => Mesh2dPipelineKey::TONEMAP_METHOD_ACES_FITTED,
        Tonemapping::AgX => Mesh2dPipelineKey::TONEMAP_METHOD_AGX,
        Tonemapping::SomewhatBoringDisplayTransform => {
            Mesh2dPipelineKey::TONEMAP_METHOD_SOMEWHAT_BORING_DISPLAY_TRANSFORM
        }
        Tonemapping::TonyMcMapface => Mesh2dPipelineKey::TONEMAP_METHOD_TONY_MC_MAPFACE,
        Tonemapping::BlenderFilmic => Mesh2dPipelineKey::TONEMAP_METHOD_BLENDER_FILMIC,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn queue_material2d_meshes<M: TextMaterial2d>(
    opaque_draw_functions: Res<DrawFunctions<Opaque2d>>,
    alpha_mask_draw_functions: Res<DrawFunctions<AlphaMask2d>>,
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    material2d_pipeline: Res<TextMaterial2dPipeline<M>>,
    mut pipelines: ResMut<SpecializedMeshPipelines<TextMaterial2dPipeline<M>>>,
    pipeline_cache: Res<PipelineCache>,
    render_meshes: Res<RenderAssets<RenderMesh>>,
    render_materials: Res<RenderAssets<PreparedTextMaterial2d<M>>>,
    mut render_mesh_instances: ResMut<RenderMesh2dInstances>,
    render_material_instances: Res<RenderTextMaterial2dInstances<M>>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
    mut opaque_render_phases: ResMut<ViewBinnedRenderPhases<Opaque2d>>,
    mut alpha_mask_render_phases: ResMut<ViewBinnedRenderPhases<AlphaMask2d>>,
    views: Query<(
        Entity,
        &ExtractedView,
        &RenderVisibleEntities,
        &Msaa,
        Option<&Tonemapping>,
        Option<&DebandDither>,
    )>,
) where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    if render_material_instances.is_empty() {
        return;
    }

    for (view_entity, view, visible_entities, msaa, tonemapping, dither) in &views {
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view_entity) else {
            continue;
        };
        let Some(opaque_phase) = opaque_render_phases.get_mut(&view_entity) else {
            continue;
        };
        let Some(alpha_mask_phase) = alpha_mask_render_phases.get_mut(&view_entity) else {
            continue;
        };

        let draw_transparent_2d = transparent_draw_functions
            .read()
            .id::<DrawTextMaterial2d<M>>();
        let draw_opaque_2d = opaque_draw_functions.read().id::<DrawTextMaterial2d<M>>();
        let draw_alpha_mask_2d = alpha_mask_draw_functions
            .read()
            .id::<DrawTextMaterial2d<M>>();

        let mut view_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples())
            | Mesh2dPipelineKey::from_hdr(view.hdr);

        if !view.hdr {
            if let Some(tonemapping) = tonemapping {
                view_key |= Mesh2dPipelineKey::TONEMAP_IN_SHADER;
                view_key |= tonemapping_pipeline_key(*tonemapping);
            }
            if let Some(DebandDither::Enabled) = dither {
                view_key |= Mesh2dPipelineKey::DEBAND_DITHER;
            }
        }
        for (render_entity, visible_entity) in visible_entities.iter::<With<Mesh2d>>() {
            let Some(material_asset_id) = render_material_instances.get(visible_entity) else {
                continue;
            };
            let Some(mesh_instance) = render_mesh_instances.get_mut(visible_entity) else {
                continue;
            };
            let Some(material_2d) = render_materials.get(*material_asset_id) else {
                continue;
            };
            let Some(mesh) = render_meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };
            let mesh_key = view_key
                | Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology())
                | material_2d.properties.mesh_pipeline_key_bits;

            let pipeline_id = pipelines.specialize(
                &pipeline_cache,
                &material2d_pipeline,
                Material2dKey {
                    mesh_key,
                    bind_group_data: material_2d.key.clone(),
                },
                &mesh.layout,
            );

            let pipeline_id = match pipeline_id {
                Ok(id) => id,
                Err(err) => {
                    error!("{}", err);
                    continue;
                }
            };

            mesh_instance.material_bind_group_id = material_2d.get_bind_group_id();
            let mesh_z = mesh_instance.transforms.world_from_local.translation.z;

            match material_2d.properties.alpha_mode {
                AlphaMode2d::Opaque => {
                    let bin_key = Opaque2dBinKey {
                        pipeline: pipeline_id,
                        draw_function: draw_opaque_2d,
                        asset_id: mesh_instance.mesh_asset_id.into(),
                        material_bind_group_id: material_2d.get_bind_group_id().0,
                    };
                    opaque_phase.add(
                        bin_key,
                        (*render_entity, *visible_entity),
                        BinnedRenderPhaseType::mesh(mesh_instance.automatic_batching),
                    );
                }
                AlphaMode2d::Mask(_) => {
                    let bin_key = AlphaMask2dBinKey {
                        pipeline: pipeline_id,
                        draw_function: draw_alpha_mask_2d,
                        asset_id: mesh_instance.mesh_asset_id.into(),
                        material_bind_group_id: material_2d.get_bind_group_id().0,
                    };
                    alpha_mask_phase.add(
                        bin_key,
                        (*render_entity, *visible_entity),
                        BinnedRenderPhaseType::mesh(mesh_instance.automatic_batching),
                    );
                }
                AlphaMode2d::Blend => {
                    transparent_phase.add(Transparent2d {
                        entity: (*render_entity, *visible_entity),
                        draw_function: draw_transparent_2d,
                        pipeline: pipeline_id,
                        // NOTE: Back-to-front ordering for transparent with ascending sort means far should have the
                        // lowest sort key and getting closer should increase. As we have
                        // -z in front of the camera, the largest distance is -far with values increasing toward the
                        // camera. As such we can just use mesh_z as the distance
                        sort_key: FloatOrd(mesh_z + material_2d.properties.depth_bias),
                        // Batching is done in batch_and_prepare_render_phase
                        batch_range: 0..1,
                        extra_index: PhaseItemExtraIndex::NONE,
                    });
                }
            }
        }
    }
}

/// Common [`Material2d`] properties, calculated for a specific material instance.
pub struct Material2dProperties {
    /// The [`AlphaMode2d`] of this material.
    pub alpha_mode: AlphaMode2d,
    /// Add a bias to the view depth of the mesh which can be used to force a specific render order
    /// for meshes with equal depth, to avoid z-fighting.
    /// The bias is in depth-texture units so large values may
    pub depth_bias: f32,
    /// The bits in the [`Mesh2dPipelineKey`] for this material.
    ///
    /// These are precalculated so that we can just "or" them together in
    /// [`queue_material2d_meshes`].
    pub mesh_pipeline_key_bits: Mesh2dPipelineKey,
}

/// Data prepared for a [`Material2d`] instance.
pub struct PreparedTextMaterial2d<T: TextMaterial2d> {
    pub bindings: Vec<(u32, OwnedBindingResource)>,
    pub bind_group: BindGroup,
    pub key: T::Data,
    pub properties: Material2dProperties,
}

impl<T: TextMaterial2d> PreparedTextMaterial2d<T> {
    pub fn get_bind_group_id(&self) -> Material2dBindGroupId {
        Material2dBindGroupId(Some(self.bind_group.id()))
    }
}

impl<M: TextMaterial2d> RenderAsset for PreparedTextMaterial2d<M> {
    type SourceAsset = M;

    type Param = (
        SRes<RenderDevice>,
        SRes<TextMaterial2dPipeline<M>>,
        M::Param,
    );

    fn prepare_asset(
        material: Self::SourceAsset,
        (render_device, pipeline, material_param): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        match material.as_bind_group(&pipeline.material2d_layout, render_device, material_param) {
            Ok(prepared) => {
                let mut mesh_pipeline_key_bits = Mesh2dPipelineKey::empty();
                mesh_pipeline_key_bits.insert(alpha_mode_pipeline_key(material.alpha_mode()));
                Ok(PreparedTextMaterial2d {
                    bindings: prepared.bindings,
                    bind_group: prepared.bind_group,
                    key: prepared.data,
                    properties: Material2dProperties {
                        depth_bias: material.depth_bias(),
                        alpha_mode: material.alpha_mode(),
                        mesh_pipeline_key_bits,
                    },
                })
            }
            Err(AsBindGroupError::RetryNextUpdate) => {
                Err(PrepareAssetError::RetryNextUpdate(material))
            }
            Err(other) => Err(PrepareAssetError::AsBindGroupError(other)),
        }
    }
}
