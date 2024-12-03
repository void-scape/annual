use crate::{
    characters::portrait::PortraitBundle,
    cutscene::CutsceneMovement,
    dialogue::{evaluate::FragmentStates, fragment::*, FragmentId},
    dialogue_box::{
        despawn_dialogue_box, spawn_dialogue_box, BoxContext, DialogueBox, IntoBox, SpawnBox,
    },
    ldtk::{Entities, Interactions},
    player::{Action, Player},
};
use bevy::prelude::*;
use bevy_ecs_ldtk::{
    app::{LdtkEntity, LdtkEntityAppExt},
    ldtk::FieldValue,
};
use leafwing_input_manager::prelude::ActionState;
use macros::t;

/// Handles behavior associated with interaction dialogue.
///
/// That is, entities that will create dialogue when interacted with.
pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InteractionTrigger>()
            .register_ldtk_entity::<InteractionBundle>(Entities::Interaction.identifier())
            .add_systems(Startup, spawn_interaction_dialogue)
            .add_systems(Update, handle_interactions);
    }
}

#[derive(Debug, Event)]
struct InteractionTrigger(Interactions);

trait SpawnInteraction {
    fn spawn(self, interaction: Interactions, commands: &mut Commands);
}

impl<T> SpawnInteraction for T
where
    T: IntoBox,
{
    fn spawn(self, interaction: Interactions, commands: &mut Commands) {
        // TODO: this is probably terribly inefficient
        // self.eval(
        //     move |id: In<FragmentId>,
        //           mut reader: EventReader<InteractionTrigger>,
        //           states: Res<FragmentStates>| {
        //         let state = states.get(id.0).copied().unwrap_or_default();
        //
        //         reader.read().any(|e| e.0 == interaction) || (state.triggered != state.completed)
        //     },
        // )
        // .spawn_box(commands, &crate::DESC);

        let context = BoxContext::new(
            commands
                .spawn_empty()
                .insert((DialogueBox, crate::dialogue_box::DIALOGUE_BOX_RENDER_LAYER))
                .id(),
        );

        let (fragment, tree) = self
            .eval(
                move |id: In<FragmentId>,
                      mut reader: EventReader<InteractionTrigger>,
                      states: Res<FragmentStates>| {
                    let state = states.get(id.0).copied().unwrap_or_default();

                    reader.read().any(|e| e.0 == interaction)
                        || (state.triggered != state.completed)
                },
            )
            .on_start(spawn_dialogue_box(context.entity(), &crate::DESC))
            .on_end(despawn_dialogue_box(context.entity()))
            .into_fragment(&context, commands);

        let portrait = commands
            .spawn_empty()
            .insert((
                PortraitBundle::new_empty(crate::DESC.portrait),
                crate::dialogue_box::DIALOGUE_BOX_RENDER_LAYER,
            ))
            .id();
        commands.entity(context.entity()).add_child(portrait);
        crate::dialogue::fragment::spawn_fragment(fragment, context, tree, commands);
    }
}

fn spawn_interaction_dialogue(mut commands: Commands) {
    "Wow! What a big tree.".spawn(Interactions::TreeInfo, &mut commands);

    t!("This one's a little [0.5] smaller...").spawn(Interactions::SmallTree, &mut commands);

    "You really like trees, huh?".spawn(Interactions::TwistyTree, &mut commands);
}

/// The source of the interaction.
///
/// This is very deliberately an option. If an error occurs
/// while parsing the entity from the LDtk file, this will
/// simply be `None`.
#[derive(Component)]
struct InteractionTriggerSource(Option<Interactions>);

#[derive(Bundle)]
struct InteractionBundle {
    source: InteractionTriggerSource,
}

impl LdtkEntity for InteractionBundle {
    fn bundle_entity(
        entity_instance: &bevy_ecs_ldtk::EntityInstance,
        _: &bevy_ecs_ldtk::prelude::LayerInstance,
        _: Option<&Handle<Image>>,
        _: Option<&bevy_ecs_ldtk::prelude::TilesetDefinition>,
        _: &AssetServer,
        _: &mut Assets<TextureAtlasLayout>,
    ) -> Self {
        let value = entity_instance.field_instances.iter().find_map(|i| {
            if i.identifier == "Name" {
                Some(&i.value)
            } else {
                None
            }
        });

        let int = match value {
            Some(FieldValue::String(Some(s))) => Interactions::from_identifier(s),
            _ => None,
        };

        InteractionBundle {
            source: InteractionTriggerSource(int),
        }
    }
}

fn handle_interactions(
    player: Query<(&Transform, &ActionState<Action>), (With<Player>, Without<CutsceneMovement>)>,
    interactions: Query<(&Transform, &InteractionTriggerSource)>,
    mut writer: EventWriter<InteractionTrigger>,
) {
    let Ok((player, action)) = player.get_single() else {
        return;
    };

    if action.just_pressed(&Action::Interact) {
        for (int, source) in interactions.iter().filter_map(|i| i.1 .0.map(|s| (i.0, s))) {
            if int.translation.distance(player.translation) < 16. {
                writer.send(InteractionTrigger(source));
            }
        }
    }
}
