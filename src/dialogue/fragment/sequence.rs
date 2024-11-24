use super::{Fragment, IntoFragment};
use crate::dialogue::{DialogueEvent, DialogueId};
use bevy::prelude::*;
use bevy::utils::all_tuples;

pub struct Sequence<F> {
    fragments: F,
    ids: Vec<DialogueId>,
}

macro_rules! seq_frag {
    ($($ty:ident),*) => {
        #[allow(non_snake_case)]
        impl<$($ty),*> IntoFragment for Sequence<($($ty,)*)>
        where
            $($ty: IntoFragment),*
        {
            type Fragment = Sequence<($($ty::Fragment,)*)>;

            fn into_fragment(self, world: &mut World) -> Self::Fragment {
                let mut ids = self.ids;
                let ($($ty,)*) = self.fragments;

                Sequence {
                    fragments: (
                        $({
                            let frag = $ty.into_fragment(world);
                            ids.extend(frag.id());
                            frag
                        },)*
                    ),
                    ids,
                }
            }
        }

        #[allow(non_snake_case)]
        impl<$($ty),*> Fragment for Sequence<($($ty,)*)>
        where
            $($ty: Fragment),*
        {
            fn emit(
                &mut self,
                selected_id: DialogueId,
                writer: &mut EventWriter<DialogueEvent>,
                commands: &mut Commands,
            ) {
                // TODO: I'm not quite sure what to do heere yet.
                todo!();
            }

            fn id(&self) -> &[DialogueId] {
                &self.ids
            }
        }
    };
}

all_tuples!(seq_frag, 0, 15, T);

pub fn sequence<F>(fragments: F) -> Sequence<F> {
    Sequence {
        fragments,
        ids: Vec::new(),
    }
}
