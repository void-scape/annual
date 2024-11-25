use super::{DialogueStates, End, Fragment, FragmentNode, IntoFragment, Start};
use crate::dialogue::{DialogueEvent, DialogueId};
use bevy::ecs::system::SystemId;
use bevy::prelude::*;

pub struct OnVisit<F, T> {
    pub(super) fragment: F,
    pub(super) on_trigger: T,
}

impl<F, T> IntoFragment for OnVisit<F, T>
where
    F: IntoFragment,
    T: System<In = (), Out = ()>,
{
    type Fragment = OnVisit<F::Fragment, SystemId>;

    fn into_fragment(self, commands: &mut Commands) -> (Self::Fragment, FragmentNode) {
        let (fragment, node) = self.fragment.into_fragment(commands);

        (
            OnVisit {
                fragment,
                on_trigger: commands.register_one_shot_system(self.on_trigger),
            },
            node,
        )
    }
}

impl<F> Fragment for OnVisit<F, SystemId>
where
    F: Fragment,
{
    fn start(
        &mut self,
        selected_id: DialogueId,
        state: &mut DialogueStates,
        writer: &mut EventWriter<DialogueEvent>,
        commands: &mut Commands,
    ) -> Start {
        let start = self.fragment.start(selected_id, state, writer, commands);

        // Run triggers whenever any children are selected.
        if start.visited() {
            commands.run_system(self.on_trigger);
        }

        start
    }

    fn end(&mut self, id: DialogueId, state: &mut DialogueStates, commands: &mut Commands) -> End {
        self.fragment.end(id, state, commands)
    }

    fn id(&self) -> &DialogueId {
        self.fragment.id()
    }
}

pub struct OnStart<F, T> {
    pub(super) fragment: F,
    pub(super) on_trigger: T,
}

impl<F, T> IntoFragment for OnStart<F, T>
where
    F: IntoFragment,
    T: System<In = (), Out = ()>,
{
    type Fragment = OnStart<F::Fragment, SystemId>;

    fn into_fragment(self, commands: &mut Commands) -> (Self::Fragment, FragmentNode) {
        let (fragment, node) = self.fragment.into_fragment(commands);

        (
            OnStart {
                fragment,
                on_trigger: commands.register_one_shot_system(self.on_trigger),
            },
            node,
        )
    }
}

impl<F> Fragment for OnStart<F, SystemId>
where
    F: Fragment,
{
    fn start(
        &mut self,
        selected_id: DialogueId,
        state: &mut DialogueStates,
        writer: &mut EventWriter<DialogueEvent>,
        commands: &mut Commands,
    ) -> Start {
        let start = self.fragment.start(selected_id, state, writer, commands);

        if start.entered() {
            commands.run_system(self.on_trigger);
        }

        start
    }

    fn end(&mut self, id: DialogueId, state: &mut DialogueStates, commands: &mut Commands) -> End {
        self.fragment.end(id, state, commands)
    }

    fn id(&self) -> &DialogueId {
        self.fragment.id()
    }
}

pub struct OnEnd<F, T> {
    pub(super) fragment: F,
    pub(super) on_trigger: T,
}

impl<F, T> IntoFragment for OnEnd<F, T>
where
    F: IntoFragment,
    T: System<In = (), Out = ()>,
{
    type Fragment = OnEnd<F::Fragment, SystemId>;

    fn into_fragment(self, commands: &mut Commands) -> (Self::Fragment, FragmentNode) {
        let (fragment, node) = self.fragment.into_fragment(commands);

        (
            OnEnd {
                fragment,
                on_trigger: commands.register_one_shot_system(self.on_trigger),
            },
            node,
        )
    }
}

impl<F> Fragment for OnEnd<F, SystemId>
where
    F: Fragment,
{
    fn start(
        &mut self,
        id: DialogueId,
        state: &mut DialogueStates,
        writer: &mut EventWriter<DialogueEvent>,
        commands: &mut Commands,
    ) -> Start {
        self.fragment.start(id, state, writer, commands)
    }

    fn end(&mut self, id: DialogueId, state: &mut DialogueStates, commands: &mut Commands) -> End {
        let end = self.fragment.end(id, state, commands);

        if end.exited() {
            commands.run_system(self.on_trigger);
        }

        end
    }

    fn id(&self) -> &DialogueId {
        self.fragment.id()
    }
}
