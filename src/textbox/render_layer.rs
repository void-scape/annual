use bevy::prelude::*;
use bevy::render::view::RenderLayers;

/// https://github.com/bevyengine/bevy/issues/5183#issuecomment-1555946084
#[derive(Default, Component)]
pub struct PropagateRenderLayers;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum RenderLayerSystem {
    RenderLayerPropagation,
}

pub struct RenderLayerPlugin;

impl Plugin for RenderLayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                propagate_render_layers_added,
                propagate_render_layers_changed,
                propagate_render_layers_add_child,
            )
                .in_set(RenderLayerSystem::RenderLayerPropagation),
        )
        .configure_sets(PostUpdate, RenderLayerSystem::RenderLayerPropagation);
    }
}

fn propagate_render_layers_added(
    mut cmds: Commands,
    parents: Query<(&RenderLayers, &Children), (Added<PropagateRenderLayers>, With<Children>)>,
) {
    for (render_layers, children) in &parents {
        for child in children {
            cmds.entity(*child)
                .insert((PropagateRenderLayers, render_layers.clone()));
        }
    }
}

fn propagate_render_layers_changed(
    mut cmds: Commands,
    parents: Query<
        (&RenderLayers, &Children),
        (
            With<PropagateRenderLayers>,
            With<Children>,
            Changed<RenderLayers>,
        ),
    >,
) {
    for (render_layers, children) in &parents {
        for child in children {
            cmds.entity(*child).insert(render_layers.clone());
        }
    }
}

fn propagate_render_layers_add_child(
    mut cmds: Commands,
    parents: Query<(&RenderLayers, &Children), (With<PropagateRenderLayers>, Changed<Children>)>,
) {
    for (render_layers, children) in &parents {
        for child in children {
            cmds.entity(*child)
                .insert((PropagateRenderLayers, render_layers.clone()));
        }
    }
}
