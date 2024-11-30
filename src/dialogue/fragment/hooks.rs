use super::{End, Fragment, FragmentNode, FragmentStates, IntoFragment, Start, Threaded};
use crate::dialogue::{FragmentEvent, FragmentId};
use bevy::ecs::system::SystemId;
use bevy::prelude::*;

pub struct OnVisit<F, T> {
    pub(super) fragment: F,
    pub(super) on_trigger: T,
}

impl<Context, Data, F, T> IntoFragment<Context, Data> for OnVisit<F, T>
where
    F: IntoFragment<Context, Data>,
    T: System<In = (), Out = ()>,
    Data: Threaded,
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

impl<Context, Data, F> Fragment<Context, Data> for OnVisit<F, SystemId>
where
    F: Fragment<Context, Data>,
    Data: Threaded,
{
    fn start(
        &mut self,
        context: &Context,
        selected_id: FragmentId,
        state: &mut FragmentStates,
        writer: &mut EventWriter<FragmentEvent<Data>>,
        commands: &mut Commands,
    ) -> Start {
        let start = self
            .fragment
            .start(context, selected_id, state, writer, commands);

        // Run triggers whenever any children are selected.
        if start.visited() {
            commands.run_system(self.on_trigger);
        }

        start
    }

    fn end(
        &mut self,
        context: &Context,
        id: FragmentId,
        state: &mut FragmentStates,
        commands: &mut Commands,
    ) -> End {
        self.fragment.end(context, id, state, commands)
    }

    fn id(&self) -> &FragmentId {
        self.fragment.id()
    }
}

pub struct OnStart<F, T> {
    pub(super) fragment: F,
    pub(super) on_trigger: T,
}

impl<Context, Data, F, T> IntoFragment<Context, Data> for OnStart<F, T>
where
    F: IntoFragment<Context, Data>,
    T: System<In = (), Out = ()>,
    Data: Threaded,
    Context: Clone + Threaded,
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

impl<Context, Data, F> Fragment<Context, Data> for OnStart<F, SystemId>
where
    F: Fragment<Context, Data>,
    Data: Threaded,
    Context: Clone + Send,
{
    fn start(
        &mut self,
        context: &Context,
        selected_id: FragmentId,
        state: &mut FragmentStates,
        writer: &mut EventWriter<FragmentEvent<Data>>,
        commands: &mut Commands,
    ) -> Start {
        let start = self
            .fragment
            .start(context, selected_id, state, writer, commands);

        if start.entered() {
            commands.run_system_with_input(self.on_trigger, context.clone());
        }

        start
    }

    fn end(
        &mut self,
        context: &Context,
        id: FragmentId,
        state: &mut FragmentStates,
        commands: &mut Commands,
    ) -> End {
        self.fragment.end(context, id, state, commands)
    }

    fn id(&self) -> &FragmentId {
        self.fragment.id()
    }
}

pub struct OnEnd<F, T> {
    pub(super) fragment: F,
    pub(super) on_trigger: T,
}

impl<Context, Data, F, T> IntoFragment<Context, Data> for OnEnd<F, T>
where
    F: IntoFragment<Context, Data>,
    T: System<In = (), Out = ()>,
    Data: Threaded,
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

impl<Context, Data, F> Fragment<Context, Data> for OnEnd<F, SystemId>
where
    F: Fragment<Context, Data>,
    Data: Threaded,
{
    fn start(
        &mut self,
        context: &Context,
        id: FragmentId,
        state: &mut FragmentStates,
        writer: &mut EventWriter<FragmentEvent<Data>>,
        commands: &mut Commands,
    ) -> Start {
        self.fragment.start(context, id, state, writer, commands)
    }

    fn end(
        &mut self,
        context: &Context,
        id: FragmentId,
        state: &mut FragmentStates,
        commands: &mut Commands,
    ) -> End {
        let end = self.fragment.end(context, id, state, commands);

        if end.exited() {
            commands.run_system(self.on_trigger);
        }

        end
    }

    fn id(&self) -> &FragmentId {
        self.fragment.id()
    }
}
