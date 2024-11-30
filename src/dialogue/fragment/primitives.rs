use super::{FragmentNode, IntoFragment, Threaded};

macro_rules! impl_leaf {
    ($ty:ty) => {
        impl<Data> IntoFragment<Data> for $ty
        where
            Data: FragmentData + From<$ty>,
        {
            type Fragment = crate::dialogue::fragment::Leaf<$ty>;

            fn into_fragment(
                self,
                _: &mut bevy::prelude::Commands,
            ) -> (Self::Fragment, FragmentNode) {
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
