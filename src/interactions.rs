use crate::annual::Interactions;
use crate::collision::trigger::TriggerEvent;
use crate::collision::{trigger::Trigger, Collider};
use crate::frags::insert_box;
use crate::player::{Action, Player};
use crate::{CutsceneMovement, IntoBox, TextBoxContext};
use bevy::prelude::*;
use bevy_sequence::prelude::*;
use leafwing_input_manager::prelude::ActionState;

/// Handles behavior associated with interaction dialogue.
///
/// That is, entities that will create dialogue when interacted with.
pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InteractionTrigger>()
            .add_systems(Update, (insert_interaction_source, handle_interactions));
    }
}

pub trait SpawnInteraction: IntoBox + Sized {
    fn spawn_interaction(
        self,
        interaction: Interactions,
        commands: &mut Commands,
    ) {
        let entity = commands.spawn_empty().id();
        let interaction = self
            .eval_id(
                move |In(id): In<FragmentId>,
                      mut reader: EventReader<InteractionTrigger>,
                      fragments: Query<&FragmentState>| {
                    reader.read().any(|e| e.0 == interaction)
                        && fragments
                            .get(id.entity())
                            .ok()
                            .is_none_or(|state| !state.active)
                },
            )
            .on_start(
                move |mut commands: Commands, asset_server: Res<AssetServer>| {
                    insert_box(entity, &asset_server, &mut commands)
                },
            )
            .on_end(move |mut commands: Commands| {
                commands.entity(entity).clear().clear_children();
            });
        spawn_root_with(interaction, commands, TextBoxContext::new(entity));
    }
}

impl<T> SpawnInteraction for T where T: IntoBox {}

/// The source of the interaction.
#[derive(Component)]
#[require(Transform, Trigger(|| Trigger(Collider::from_circle(Vec2::ZERO, 20.))))]
struct InteractionTriggerSource(Interactions);

#[derive(Debug, Event)]
struct InteractionTrigger(Interactions);

fn insert_interaction_source(
    mut commands: Commands,
    entities: Query<(Entity, &crate::annual::Interaction), Added<crate::annual::Interaction>>,
) {
    for (entity, interaction) in entities.iter() {
        commands
            .entity(entity)
            .insert(InteractionTriggerSource(interaction.interactions));
    }
}

fn handle_interactions(
    player: Option<
        Single<(Entity, &ActionState<Action>), (With<Player>, Without<CutsceneMovement>)>,
    >,
    interactions: Query<&InteractionTriggerSource>,
    mut reader: EventReader<TriggerEvent>,
    mut writer: EventWriter<InteractionTrigger>,
) {
    let Some((entity, action)) = player.map(|p| p.into_inner()) else {
        return;
    };

    let interact = action.just_pressed(&Action::Interact);
    for event in reader.read() {
        if interact && event.target == entity {
            if let Ok(int) = interactions.get(event.trigger) {
                writer.send(InteractionTrigger(int.0));
            }
        }
    }
}
