use bevy::prelude::*;
use bevy_sequence::Threaded;

pub trait IntoCurve<C> {
    fn into_curve(&self, start: Vec3, end: Vec3) -> impl Curve<Vec3> + Threaded;
}

impl IntoCurve<EasingCurve<Vec3>> for EaseFunction {
    fn into_curve(&self, start: Vec3, end: Vec3) -> impl Curve<Vec3> + Threaded {
        EasingCurve::new(start, end, *self)
    }
}
