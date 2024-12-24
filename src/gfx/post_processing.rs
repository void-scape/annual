use std::marker::PhantomData;

use super::camera::MainCamera;
use bevy::{
    core_pipeline::{
        core_2d::graph::{Core2d, Node2d},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    },
    ecs::{component::StorageType, query::QueryItem},
    prelude::*,
    render::{
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice},
        view::ViewTarget,
        RenderApp,
    },
};

/// Apply post processing to the main camera through an [`ApplyPostProcess`].
///
/// All [`Component`] types implement [`ApplyPostProcess`].
pub trait PostProcessCommand {
    /// Applies the post process to the [`MainCamera`].
    fn post_process(&mut self, post_process: impl ApplyPostProcess);

    /// Applies the post process to the [`MainCamera`], then binds the lifetime of the post process
    /// to the provided entity.
    fn bind_post_process(&mut self, post_process: impl ApplyPostProcess + Sync, entity: Entity);

    /// Removes the post process to the [`MainCamera`].
    fn remove_post_process<T: ApplyPostProcess>(&mut self);
}

impl PostProcessCommand for Commands<'_, '_> {
    fn post_process(&mut self, post_process: impl ApplyPostProcess) {
        self.queue(apply(post_process));
    }

    fn bind_post_process(&mut self, post_process: impl ApplyPostProcess + Sync, entity: Entity) {
        self.queue(bind(post_process, entity));
    }

    fn remove_post_process<T: ApplyPostProcess>(&mut self) {
        self.queue(remove::<T>);
    }
}

/// Determines how a post process is inserted and removed from the main camera.
pub trait ApplyPostProcess: 'static + Send {
    fn insert(self, entity: &mut EntityWorldMut<'_>);
    fn remove(entity: &mut EntityWorldMut<'_>);
}

impl<T: Component> ApplyPostProcess for T {
    fn insert(self, entity: &mut EntityWorldMut<'_>) {
        entity.insert(self);
    }

    fn remove(entity: &mut EntityWorldMut<'_>) {
        entity.remove::<T>();
    }
}

pub fn apply(post_process: impl ApplyPostProcess) -> impl FnOnce(&mut World) {
    move |world: &mut World| {
        if let Some(camera) = world
            .query_filtered::<Entity, With<MainCamera>>()
            .iter(&world)
            .next()
        {
            post_process.insert(&mut world.entity_mut(camera));
        }
    }
}

struct PostProcessBinding<T>(PhantomData<T>);

impl<T: ApplyPostProcess + Sync> Component for PostProcessBinding<T> {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_remove(|mut world, _, _| {
            world.commands().queue(remove::<T>);
        });
    }
}

pub fn bind<T: ApplyPostProcess + Sync>(post_process: T, entity: Entity) -> impl FnOnce(&mut World) {
    move |world: &mut World| {
        if let Some(camera) = world
            .query_filtered::<Entity, With<MainCamera>>()
            .iter(&world)
            .next()
        {
            post_process.insert(&mut world.entity_mut(camera));
            world
                .entity_mut(entity)
                .with_child(PostProcessBinding::<T>(PhantomData));
        }
    }
}

pub fn remove<T: ApplyPostProcess>(world: &mut World) {
    if let Some(camera) = world
        .query_filtered::<Entity, With<MainCamera>>()
        .iter(&world)
        .next()
    {
        T::remove(&mut world.entity_mut(camera));
    }
}

//#[derive(Debug, Clone)]
//pub enum PostProcess {
//    Bloom(Bloom),
//}

//#[derive(Debug, Default, Clone, Copy, Component, ExtractComponent, ShaderType)]
//pub struct PostProcess {
//    intensity: f32,
//    // WebGL2 structs must be 16 byte aligned.
//    //#[cfg(feature = "webgl2")]
//    //_webgl2_padding: Vec3,
//}
//
//impl PostProcess {
//    pub fn new(intensity: f32) -> Self {
//        Self {
//            intensity,
//            //#[cfg(feature = "webgl2")]
//            //_webgl2_padding: Vec3::default(),
//        }
//    }
//}
//
//const SHADER_ASSET_PATH: &str = "shaders/post_processing.wgsl";
//
//pub struct PostProcessPlugin;
//
//impl Plugin for PostProcessPlugin {
//    fn build(&self, app: &mut App) {
//        app.add_plugins((
//            ExtractComponentPlugin::<PostProcess>::default(),
//            UniformComponentPlugin::<PostProcess>::default(),
//        ));
//
//        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
//            return;
//        };
//
//        render_app
//            .add_render_graph_node::<ViewNodeRunner<PostProcessNode>>(Core2d, PostProcessLabel)
//            .add_render_graph_edges(
//                Core2d,
//                (
//                    Node2d::Tonemapping,
//                    PostProcessLabel,
//                    Node2d::EndMainPassPostProcessing,
//                ),
//            );
//    }
//
//    fn finish(&self, app: &mut App) {
//        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
//            return;
//        };
//
//        render_app.init_resource::<PostProcessPipeline>();
//    }
//}
//
//#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
//struct PostProcessLabel;
//
//#[derive(Default)]
//struct PostProcessNode;
//
//impl ViewNode for PostProcessNode {
//    type ViewQuery = (
//        &'static ViewTarget,
//        &'static PostProcess,
//        &'static DynamicUniformIndex<PostProcess>,
//    );
//
//    fn run(
//        &self,
//        _graph: &mut RenderGraphContext,
//        render_context: &mut RenderContext,
//        (view_target, _post_process_settings, settings_index): QueryItem<Self::ViewQuery>,
//        world: &World,
//    ) -> Result<(), NodeRunError> {
//        let post_process_pipeline = world.resource::<PostProcessPipeline>();
//        let pipeline_cache = world.resource::<PipelineCache>();
//
//        let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id)
//        else {
//            return Ok(());
//        };
//
//        let settings_uniforms = world.resource::<ComponentUniforms<PostProcess>>();
//        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
//            return Ok(());
//        };
//
//        let post_process = view_target.post_process_write();
//
//        let bind_group = render_context.render_device().create_bind_group(
//            "post_process_bind_group",
//            &post_process_pipeline.layout,
//            &BindGroupEntries::sequential((
//                post_process.source,
//                &post_process_pipeline.sampler,
//                settings_binding.clone(),
//            )),
//        );
//
//        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
//            label: Some("post_process_pass"),
//            color_attachments: &[Some(RenderPassColorAttachment {
//                view: post_process.destination,
//                resolve_target: None,
//                ops: Operations::default(),
//            })],
//            depth_stencil_attachment: None,
//            timestamp_writes: None,
//            occlusion_query_set: None,
//        });
//
//        render_pass.set_render_pipeline(pipeline);
//        render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
//        render_pass.draw(0..3, 0..1);
//
//        Ok(())
//    }
//}
//
//#[derive(Resource)]
//struct PostProcessPipeline {
//    layout: BindGroupLayout,
//    sampler: Sampler,
//    pipeline_id: CachedRenderPipelineId,
//}
//
//impl FromWorld for PostProcessPipeline {
//    fn from_world(world: &mut World) -> Self {
//        let render_device = world.resource::<RenderDevice>();
//
//        let layout = render_device.create_bind_group_layout(
//            "post_process_bind_group_layout",
//            &BindGroupLayoutEntries::sequential(
//                ShaderStages::FRAGMENT,
//                (
//                    texture_2d(TextureSampleType::Float { filterable: true }),
//                    sampler(SamplerBindingType::Filtering),
//                    uniform_buffer::<PostProcess>(true),
//                ),
//            ),
//        );
//
//        let sampler = render_device.create_sampler(&SamplerDescriptor::default());
//        let shader = world.load_asset(SHADER_ASSET_PATH);
//
//        let pipeline_id =
//            world
//                .resource_mut::<PipelineCache>()
//                .queue_render_pipeline(RenderPipelineDescriptor {
//                    label: Some("post_process_pipeline".into()),
//                    layout: vec![layout.clone()],
//                    vertex: fullscreen_shader_vertex_state(),
//                    fragment: Some(FragmentState {
//                        shader,
//                        shader_defs: vec![],
//                        entry_point: "fragment".into(),
//                        targets: vec![Some(ColorTargetState {
//                            // HDR texture format
//                            format: TextureFormat::Rgba16Float,
//                            blend: None,
//                            write_mask: ColorWrites::ALL,
//                        })],
//                    }),
//                    primitive: PrimitiveState::default(),
//                    depth_stencil: None,
//                    multisample: MultisampleState::default(),
//                    push_constant_ranges: vec![],
//                    zero_initialize_workgroup_memory: false,
//                });
//
//        Self {
//            layout,
//            sampler,
//            pipeline_id,
//        }
//    }
//}
