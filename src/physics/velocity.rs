use bevy::prelude::*;

#[derive(Component)]
pub struct Velocity(pub Vec2);

pub fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += (velocity.0 * time.delta_secs()).extend(0.);
    }
}
