use super::{End, Fragment, FragmentData, FragmentNode, IntoFragment, Start};
use crate::dialogue::{evaluate::FragmentStates, FragmentEvent, FragmentId};
use bevy::prelude::*;

pub struct StringFragment<S> {
    string: S,
    id: FragmentId,
}

impl<S> StringFragment<S>
where
    S: AsRef<str>,
{
    pub fn new(string: S) -> Self {
        Self {
            string,
            id: FragmentId::random(),
        }
    }
}

macro_rules! string {
    ($ty:ty) => {
        impl<Data> IntoFragment<Data> for $ty
        where
            Data: FragmentData + for<'s> From<&'s str>,
        {
            type Fragment = StringFragment<$ty>;

            fn into_fragment(self, _: &mut Commands) -> (Self::Fragment, FragmentNode) {
                let fragment = StringFragment::new(self);
                let node = FragmentNode::leaf(fragment.id);

                (fragment, node)
            }
        }
    };
}

string!(&'static str);
string!(String);

impl<S, Data> Fragment<Data> for StringFragment<S>
where
    S: AsRef<str>,
    Data: FragmentData + for<'s> From<&'s str>,
{
    fn start(
        &mut self,
        id: FragmentId,
        state: &mut FragmentStates,
        writer: &mut EventWriter<FragmentEvent<Data>>,
        _commands: &mut Commands,
    ) -> Start {
        if id == self.id {
            writer.send(FragmentEvent {
                data: self.string.as_ref().into(),
                id: self.id,
            });

            let state = state.update(id);
            state.triggered += 1;
            state.active = true;

            Start::Entered
        } else {
            Start::Unvisited
        }
    }

    fn end(&mut self, id: FragmentId, state: &mut FragmentStates, _commands: &mut Commands) -> End {
        if id == self.id {
            let state = state.update(id);
            state.completed += 1;
            state.active = false;

            End::Exited
        } else {
            End::Unvisited
        }
    }

    fn id(&self) -> &FragmentId {
        &self.id
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::dialogue::evaluate::EvaluatedFragments;

    #[test]
    fn test() {
        let world = bevy::app::App::new().add_systems(Startup, |mut world: Commands| {
            let fragment1 = "Hello, world!"
                .eval(|| [true, true])
                .on_visit(|idk: ResMut<EvaluatedFragments>| println!("idk: {:#?}", idk))
                .into_fragment(&mut world);
            let fragment2 = String::from("Hello, world!").into_fragment(&mut world);
        });
    }
}
