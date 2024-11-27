use super::{FragmentNode, IntoFragment};

macro_rules! impl_leaf {
    ($ty:ty) => {
        impl IntoFragment for $ty {
            type Fragment<Data> = crate::dialogue::fragment::Leaf<$ty>;

            fn into_fragment<Data>(
                self,
                _: &mut bevy::prelude::Commands,
            ) -> (Self::Fragment<Data>, FragmentNode) {
                crate::dialogue::fragment::Leaf::new(self)
            }
        }
    };
}

impl_leaf!(&'static str);
impl_leaf!(String);

impl_leaf!(i8);
impl_leaf!(i16);
impl_leaf!(i32);
impl_leaf!(i64);
impl_leaf!(isize);

impl_leaf!(u8);
impl_leaf!(u16);
impl_leaf!(u32);
impl_leaf!(u64);
impl_leaf!(usize);
