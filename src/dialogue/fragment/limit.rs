use super::{FragmentNode, IntoFragment};
use crate::dialogue::{
    evaluate::{EvaluatedFragments, FragmentStates},
    FragmentId,
};
use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct LimitItem {
    id: FragmentId,
    limit: usize,
}

pub fn update_limit_items(
    q: Query<&LimitItem>,
    state: Res<FragmentStates>,
    mut evals: ResMut<EvaluatedFragments>,
) {
    for LimitItem { id, limit } in q.iter() {
        let eval = state
            .state
            .get(id)
            .map(|c| c.completed < *limit)
            .unwrap_or(true);

        evals.insert(*id, eval);
    }
}

pub struct Limit<F> {
    fragment: F,
    limit: usize,
}

impl<F> Limit<F> {
    pub fn new(fragment: F, limit: usize) -> Self {
        Self { fragment, limit }
    }
}

impl<F> IntoFragment for Limit<F>
where
    F: IntoFragment,
{
    type Fragment<Data> = F::Fragment<Data>;

    fn into_fragment<Data>(self, commands: &mut Commands) -> (Self::Fragment<Data>, FragmentNode) {
        let (fragment, node) = self.fragment.into_fragment(commands);
        commands.spawn(LimitItem {
            id: node.id,
            limit: self.limit,
        });

        (fragment, node)
    }
}
