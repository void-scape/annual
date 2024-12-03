use super::{
    spatial::{SpatialData, SpatialHash},
    Collider, CollidesWith,
};
use bevy::prelude::*;

/// Marks an entity as a [`TriggerEvent`] source.
///
/// Can exist in combination with a [`StaticBody`] or [`DynamicBody`].
#[derive(Debug, Default, Clone, Copy, Component)]
pub struct Trigger(pub Collider);

/// Both the [`Trigger`] and the `flipper` need to have the same [`TriggerLayer`] for a
/// [`TriggerEvent`] to be emitted.
#[derive(Debug, Default, Clone, Copy, Component)]
pub struct TriggerLayer(pub usize);

#[derive(Default, Bundle)]
pub struct TriggerBundle {
    pub trigger: Trigger,
    pub layer: TriggerLayer,
}

/// In the case of trigger triggers trigger, two triggers will each trigger, both triggering the
/// other trigger.
#[derive(Debug, Clone, Copy, Event)]
pub struct TriggerEvent {
    pub trigger: Entity,
    pub target: Entity,
}

#[derive(Default, Resource)]
pub(super) struct TriggerLayerRegistry(Vec<usize>);

pub(super) fn register_trigger_layers(
    layers: Query<&TriggerLayer, Added<TriggerLayer>>,
    mut registry: ResMut<TriggerLayerRegistry>,
) {
    for layer in layers.iter() {
        if !registry.0.contains(&layer.0) {
            registry.0.push(layer.0);
        }
    }
}

pub(super) fn handle_triggers(
    triggers: Query<(Entity, &Transform, &Trigger, &TriggerLayer)>,
    dynamic_bodies: Query<(Entity, &Transform, &Collider, &TriggerLayer)>,
    layer_registry: Res<TriggerLayerRegistry>,
    mut writer: EventWriter<TriggerEvent>,
) {
    for layer in layer_registry.0.iter() {
        let layer_triggers = triggers
            .iter()
            .filter(|(_, _, _, l)| l.0 == *layer)
            .collect::<Vec<_>>();

        let trigger_map = SpatialHash::new_with(
            64.,
            layer_triggers
                .iter()
                .map(|(e, t, tg, _)| SpatialData::from_entity(*e, t, &tg.0, ())),
        );

        for (entity, transform, trigger, _) in layer_triggers.iter() {
            let collider = trigger.0.absolute(transform);

            for SpatialData {
                entity: e,
                collider: c,
                ..
            } in trigger_map.nearby_objects(&collider.position())
            {
                if *e != *entity && collider.collides_with(c) {
                    writer.send(TriggerEvent {
                        trigger: *entity,
                        target: *e,
                    });
                }
            }
        }

        let dynamic_body_map = SpatialHash::new_with(
            64.,
            dynamic_bodies
                .iter()
                .filter(|(_, _, _, l)| l.0 == *layer)
                .map(|(e, t, c, _)| SpatialData::from_entity(e, t, c, ())),
        );

        for (entity, transform, trigger, _) in layer_triggers.iter() {
            let collider = trigger.0.absolute(transform);

            for SpatialData {
                entity: e,
                collider: c,
                ..
            } in dynamic_body_map.nearby_objects(&collider.position())
            {
                if collider.collides_with(c) {
                    writer.send(TriggerEvent {
                        trigger: *entity,
                        target: *e,
                    });
                }
            }
        }
    }
}
