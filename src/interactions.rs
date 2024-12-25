use crate::annual::Interactions;
use crate::physics::prelude::*;
use crate::player::{Action, Player};
use crate::{CutsceneMovement, TILE_SIZE};
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

pub trait BindInteraction<D: Threaded, C>: IntoFragment<D, C> + Sized {
    fn interaction(self, interaction: Interactions) -> impl IntoFragment<D, C> {
        self.eval_id(
            move |In(id): In<FragmentId>,
                  mut reader: EventReader<InteractionTrigger>,
                  fragments: Query<&FragmentState>| {
                reader.read().any(|e| e.0 == interaction)
                    && fragments
                        .get(id.entity())
                        .ok()
                        .is_none_or(|state| state.active_events.is_empty())
            },
        )
    }
}

impl<D: Threaded, C, T> BindInteraction<D, C> for T where T: IntoFragment<D, C> {}

/// The source of the interaction.
#[derive(Component)]
#[require(Transform)]
struct InteractionTriggerSource(Interactions);

#[derive(Debug, Event)]
struct InteractionTrigger(Interactions);

fn insert_interaction_source(
    mut commands: Commands,
    entities: Query<(Entity, &crate::annual::Interaction), Added<crate::annual::Interaction>>,
) {
    for (entity, interaction) in entities.iter() {
        commands.entity(entity).insert((
            InteractionTriggerSource(interaction.interactions),
            Trigger(Collider::from_circle(
                Vec2::new(TILE_SIZE, -TILE_SIZE),
                TILE_SIZE,
            )),
        ));
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
