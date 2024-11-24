use super::{DialogueEvent, Fragment, IntoFragment};
use crate::evaluate::DialogueId;
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

    fn into_fragment(self, world: &mut World) -> Self::Fragment {
        self
    }
}

macro_rules! string {
    ($ty:ty) => {
        impl IntoFragment for $ty {
            type Fragment = StringFragment<$ty>;

            fn into_fragment(self, _: &mut World) -> Self::Fragment {
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
        writer: &mut EventWriter<DialogueEvent>,
        commands: &mut Commands,
    ) {
        if selected_id == self.id {
            writer.send(DialogueEvent {
                dialogue: self.string.as_ref().to_owned(),
                id: self.id,
            });
        }
    }

    fn id(&self) -> &[DialogueId] {
        core::slice::from_ref(&self.id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::dialogue::EvaluatedDialogue;

    fn test() {
        let world = bevy::app::App::new().add_systems(Startup, |world: &mut World| {
            let fragment1 = "Hello, world!"
                .eval(|| [true, true])
                .on_trigger(|idk: ResMut<EvaluatedDialogue>| println!("idk: {:#?}", idk))
                .into_fragment(world);
            let fragment2 = String::from("Hello, world!").into_fragment(world);
        });
    }
}
