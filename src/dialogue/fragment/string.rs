use super::{End, Fragment, FragmentNode, IntoFragment, Start};
use crate::dialogue::{evaluate::DialogueStates, DialogueEvent, DialogueId};
use bevy::prelude::*;

pub struct StringFragment<S> {
    string: S,
    id: DialogueId,
}

impl<S> StringFragment<S>
where
    S: AsRef<str>,
{
    pub fn new(string: S) -> Self {
        Self {
            string,
            id: DialogueId::random(),
        }
    }
}

macro_rules! string {
    ($ty:ty) => {
        impl IntoFragment for $ty {
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

impl<S> Fragment for StringFragment<S>
where
    S: AsRef<str>,
{
    fn start(
        &mut self,
        id: DialogueId,
        state: &mut DialogueStates,
        writer: &mut EventWriter<DialogueEvent>,
        _commands: &mut Commands,
    ) -> Start {
        if id == self.id {
            writer.send(DialogueEvent {
                dialogue: self.string.as_ref().to_owned(),
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

    fn end(&mut self, id: DialogueId, state: &mut DialogueStates, _commands: &mut Commands) -> End {
        if id == self.id {
            let state = state.update(id);
            state.completed += 1;
            state.active = false;

            End::Exited
        } else {
            End::Unvisited
        }
    }

    fn id(&self) -> &DialogueId {
        &self.id
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::dialogue::evaluate::EvaluatedDialogue;

    #[test]
    fn test() {
        let world = bevy::app::App::new().add_systems(Startup, |mut world: Commands| {
            let fragment1 = "Hello, world!"
                .eval(|| [true, true])
                .on_visit(|idk: ResMut<EvaluatedDialogue>| println!("idk: {:#?}", idk))
                .into_fragment(&mut world);
            let fragment2 = String::from("Hello, world!").into_fragment(&mut world);
        });
    }
}
