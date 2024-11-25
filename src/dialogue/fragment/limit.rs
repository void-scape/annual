use super::{Fragment, IntoFragment};
use crate::dialogue::{
    evaluate::{DialogueStates, EvaluatedDialogue},
    DialogueId,
};
use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct LimitItem {
    id: DialogueId,
    limit: usize,
}

pub fn update_limit_items(
    q: Query<&LimitItem>,
    state: Res<DialogueStates>,
    mut evals: ResMut<EvaluatedDialogue>,
) {
    for LimitItem { id, limit } in q.iter() {
        let eval = state
            .state
            .get(id)
            .map(|c| c.triggered < *limit && !c.active)
            .unwrap_or(true);

        evals.insert(*id, eval);
    }
}

pub struct Limit<F> {
    fragment: F,
    limit: usize,
}

impl<F> Limit<F>
where
    F: IntoFragment,
{
    pub fn new(fragment: F, limit: usize) -> Self {
        Self { fragment, limit }
    }
}

impl<F> IntoFragment for Limit<F>
where
    F: IntoFragment,
{
    type Fragment = F::Fragment;

    fn into_fragment(self, commands: &mut Commands) -> Self::Fragment {
        let fragment = self.fragment.into_fragment(commands);
        commands.spawn(LimitItem {
            id: *fragment.id(),
            limit: self.limit,
        });

        fragment
    }
}
