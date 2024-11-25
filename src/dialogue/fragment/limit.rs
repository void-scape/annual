use super::{Fragment, IntoFragment, Unregistered};
use crate::dialogue::{
    evaluate::{DialogueStates, EvaluatedDialogue},
    DialogueEvent, DialogueId,
};
use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct LimitItems {
    ids: Vec<DialogueId>,
    limit: usize,
}

pub fn update_limit_items(
    q: Query<&LimitItems>,
    state: Res<DialogueStates>,
    mut evals: ResMut<EvaluatedDialogue>,
) {
    for items in q.iter() {
        for id in items.ids.iter() {
            let eval = state
                .state
                .get(id)
                .map(|c| c.triggered < items.limit && !c.active)
                .unwrap_or(true);

            evals.insert(*id, eval);
        }
    }
}

pub struct Limit<F> {
    fragment: F,
    limit: usize,
}

impl<F> Limit<Unregistered<F>>
where
    F: IntoFragment,
{
    pub fn new(fragment: F, limit: usize) -> Self {
        Self {
            fragment: Unregistered(fragment),
            limit,
        }
    }
}

impl<F> IntoFragment for Limit<Unregistered<F>>
where
    F: IntoFragment,
{
    type Fragment = Limit<F::Fragment>;

    fn into_fragment(self, world: &mut World) -> Self::Fragment {
        let fragment = self.fragment.0.into_fragment(world);
        world.spawn(LimitItems {
            ids: fragment.id().to_vec(),
            limit: self.limit,
        });

        Limit {
            limit: self.limit,
            fragment,
        }
    }
}

impl<F> IntoFragment for Limit<F>
where
    F: Fragment,
{
    type Fragment = Self;

    fn into_fragment(self, _world: &mut World) -> Self::Fragment {
        self
    }
}

impl<F> Fragment for Limit<F>
where
    F: Fragment,
{
    fn emit(
        &mut self,
        selected_id: DialogueId,
        writer: &mut EventWriter<DialogueEvent>,
        commands: &mut Commands,
    ) {
        self.fragment.emit(selected_id, writer, commands);
    }

    fn id(&self) -> &[DialogueId] {
        self.fragment.id()
    }
}
