use super::{Emitted, Fragment, IntoFragment, StackList};
use crate::dialogue::{DialogueEvent, DialogueId};
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

impl<S> IntoFragment for StringFragment<S>
where
    Self: Fragment,
{
    type Fragment = Self;

    fn into_fragment(self, _world: &mut Commands) -> Self::Fragment {
        self
    }
}

macro_rules! string {
    ($ty:ty) => {
        impl IntoFragment for $ty {
            type Fragment = StringFragment<$ty>;

            fn into_fragment(self, _: &mut Commands) -> Self::Fragment {
                StringFragment::new(self)
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
    fn emit(
        &mut self,
        selected_id: DialogueId,
        parent: Option<&StackList<DialogueId>>,
        writer: &mut EventWriter<DialogueEvent>,
        _commands: &mut Commands,
    ) -> Emitted {
        if selected_id == self.id {
            let node = StackList::new(parent, self.id());

            writer.send(DialogueEvent {
                dialogue: self.string.as_ref().to_owned(),
                id_path: node.into(),
            });

            Emitted::Emitted
        } else {
            Emitted::NotEmitted
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
                .on_trigger(|idk: ResMut<EvaluatedDialogue>| println!("idk: {:#?}", idk))
                .into_fragment(&mut world);
            let fragment2 = String::from("Hello, world!").into_fragment(&mut world);
        });
    }
}
