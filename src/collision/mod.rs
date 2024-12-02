use crate::asset_loading::loaded;
use bevy::prelude::*;

mod debug;

#[derive(Debug)]
pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TriggerEvent>()
            .insert_resource(debug::ShowCollision(false))
            .add_systems(
                PostUpdate,
                (
                    handle_collisions,
                    debug::debug_display_collider_wireframe,
                    debug::update_show_collision,
                    debug::debug_show_collision_color,
                )
                    .run_if(loaded())
                    .before(TransformSystem::TransformPropagate),
            );
    }
}

/// Emitted when a [`Trigger`] entity's collider is entered by a [`DynamicBody`] entity's collider.
#[derive(Debug, Clone, Copy, Event)]
pub struct TriggerEvent {
    pub receiver: Entity,
    /// The entity who `pulled` the trigger
    pub emitter: Entity,
}

pub trait CollidesWith<T> {
    fn collides_with(&self, other: &T) -> bool;
    fn resolution(&self, other: &T) -> Vec2;
}

#[derive(Debug, Default, Clone, Copy, Component)]
pub struct StaticBody;

#[derive(Default, Bundle)]
pub struct StaticBodyBundle {
    pub static_body: StaticBody,
    pub collider: Collider,
}

#[derive(Debug, Default, Clone, Copy, Component)]
pub struct DynamicBody;

#[derive(Default, Bundle)]
pub struct DynamicBodyBundle {
    pub dynamic_body: DynamicBody,
    pub collider: Collider,
}

#[derive(Debug, Default, Clone, Copy, Component)]
pub struct Trigger;

#[derive(Default, Bundle)]
pub struct TriggerBundle {
    pub trigger: Trigger,
    pub collider: Collider,
}

/// To check for collisions, first convert this enum into an [AbsoluteCollider]
/// with [Collider::absolute].
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub enum Collider {
    Rect(RectCollider),
    Circle(CircleCollider),
}

impl Default for Collider {
    fn default() -> Self {
        Self::from_rect(Vec2::ZERO, Vec2::ZERO)
    }
}

impl Collider {
    pub fn from_rect(tl: Vec2, size: Vec2) -> Self {
        Self::Rect(RectCollider { tl, size })
    }

    pub fn from_circle(position: Vec2, radius: f32) -> Self {
        Self::Circle(CircleCollider { position, radius })
    }

    // TODO: make this work in bevy
    pub fn absolute(&self, transform: &Transform) -> AbsoluteCollider {
        match self {
            Self::Rect(rect) => AbsoluteCollider::Rect(RectCollider {
                tl: rect.tl + transform.translation.xy(),
                size: rect.size,
            }),
            Self::Circle(circle) => AbsoluteCollider::Circle(CircleCollider {
                position: circle.position + transform.translation.xy(),
                radius: circle.radius,
            }),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AbsoluteCollider {
    Rect(RectCollider),
    Circle(CircleCollider),
}

impl AbsoluteCollider {
    pub fn position(&self) -> Vec2 {
        match self {
            Self::Rect(rect) => rect.tl,
            Self::Circle(circle) => circle.position,
        }
    }
}

impl CollidesWith<Self> for AbsoluteCollider {
    fn collides_with(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Rect(s), Self::Rect(o)) => s.collides_with(o),
            (Self::Rect(s), Self::Circle(o)) => s.collides_with(o),
            (Self::Circle(s), Self::Rect(o)) => s.collides_with(o),
            (Self::Circle(s), Self::Circle(o)) => s.collides_with(o),
        }
    }

    fn resolution(&self, other: &Self) -> Vec2 {
        match (self, other) {
            (Self::Rect(s), Self::Rect(o)) => s.resolution(o),
            (Self::Rect(s), Self::Circle(o)) => s.resolution(o),
            (Self::Circle(s), Self::Rect(o)) => s.resolution(o),
            (Self::Circle(s), Self::Circle(o)) => s.resolution(o),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Component)]
pub struct RectCollider {
    pub tl: Vec2,
    pub size: Vec2,
}

impl RectCollider {
    pub fn br(&self) -> Vec2 {
        self.tl + self.size
    }
}

impl CollidesWith<Self> for RectCollider {
    fn collides_with(&self, other: &Self) -> bool {
        let not_collided = other.tl.y > self.br().y
            || other.tl.x > self.br().x
            || other.br().y < self.tl.y
            || other.br().x < self.tl.x;

        !not_collided
    }

    // TODO: this does not work
    fn resolution(&self, other: &Self) -> Vec2 {
        // Calculate the center points of both rectangles
        let self_center = self.tl + self.size * 0.5;
        let other_center = other.tl + other.size * 0.5;

        // Determine push direction based on relative positions
        let diff = self_center - other_center;
        Vec2::new(
            if diff.x == 0.0 { 0.0 } else { diff.x.signum() },
            if diff.y == 0.0 { 0.0 } else { diff.y.signum() },
        )
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Component)]
pub struct CircleCollider {
    pub position: Vec2,
    pub radius: f32,
}

impl CollidesWith<Self> for CircleCollider {
    fn collides_with(&self, other: &Self) -> bool {
        let distance = self.position.distance_squared(other.position);
        let combined_radii = self.radius.powi(2) + other.radius.powi(2);

        distance <= combined_radii
    }

    fn resolution(&self, other: &Self) -> Vec2 {
        let diff = self.position - other.position;
        let distance = diff.length();

        // If not overlapping, return zero vector
        if distance >= self.radius + other.radius {
            return Vec2::ZERO;
        }

        // Handle the case where circles are at the same position
        if distance == 0.0 {
            return Vec2::new(self.radius + other.radius, 0.0); // Arbitrary direction
        }

        // Calculate how much the circles overlap
        let overlap = (self.radius + other.radius) - distance;

        // Calculate the direction to move
        let direction = diff / distance; // Normalized direction vector

        // Return the vector that will move self out of overlap
        direction * overlap
    }
}

impl CollidesWith<RectCollider> for CircleCollider {
    fn collides_with(&self, other: &RectCollider) -> bool {
        let dist_x = (self.position.x - (other.tl.x + other.size.x * 0.5)).abs();
        let dist_y = (self.position.y - (other.tl.y + other.size.y * 0.5)).abs();

        if dist_x > other.size.x * 0.5 + self.radius {
            return false;
        }

        if dist_y > other.size.y * 0.5 + self.radius {
            return false;
        }

        if dist_x <= other.size.x * 0.5 {
            return true;
        }

        if dist_y <= other.size.y * 0.5 {
            return true;
        }

        let corner_dist =
            (dist_x - other.size.x * 0.5).powi(2) + (dist_y - other.size.y * 0.5).powi(2);

        corner_dist <= self.radius.powi(2)
    }

    fn resolution(&self, other: &RectCollider) -> Vec2 {
        // Find the closest point on the rectangle to the circle's center
        let closest = Vec2::new(
            self.position.x.clamp(other.tl.x, other.tl.x + other.size.x),
            self.position.y.clamp(other.tl.y, other.tl.y + other.size.y),
        );

        let diff = self.position - closest;
        let distance = diff.length();

        // If not overlapping, return zero vector
        if distance >= self.radius {
            return Vec2::ZERO;
        }

        // Handle case where circle center is exactly on rectangle edge
        if distance == 0.0 {
            // Find which edge we're closest to and push out accordingly
            let to_left = self.position.x - other.tl.x;
            let to_right = (other.tl.x + other.size.x) - self.position.x;
            let to_top = self.position.y - other.tl.y;
            let to_bottom = (other.tl.y + other.size.y) - self.position.y;

            let min_dist = to_left.min(to_right).min(to_top).min(to_bottom);

            if min_dist == to_left {
                return Vec2::new(-self.radius, 0.0);
            }
            if min_dist == to_right {
                return Vec2::new(self.radius, 0.0);
            }
            if min_dist == to_top {
                return Vec2::new(0.0, -self.radius);
            }
            return Vec2::new(0.0, self.radius);
        }

        // Calculate the overlap and direction
        let overlap = self.radius - distance;
        let direction = diff / distance; // Normalized direction vector

        // Return the vector that will move the circle out of overlap
        direction * overlap
    }
}

impl CollidesWith<CircleCollider> for RectCollider {
    fn collides_with(&self, other: &CircleCollider) -> bool {
        other.collides_with(self)
    }

    fn resolution(&self, other: &CircleCollider) -> Vec2 {
        other.resolution(self)
    }
}

pub fn handle_collisions(
    bodies: Query<
        (
            Entity,
            &Transform,
            &Collider,
            Option<&Trigger>,
            Option<&StaticBody>,
        ),
        (Or<(With<Trigger>, With<StaticBody>)>, Without<DynamicBody>),
    >,
    mut dynamic_bodies: Query<(Entity, &mut Transform, &Collider), With<DynamicBody>>,
    mut writer: EventWriter<TriggerEvent>,
) {
    for (dyn_entity, mut dyn_trans, dyn_collider) in dynamic_bodies.iter_mut() {
        let dyn_collider = dyn_collider.absolute(&dyn_trans);
        for (body_entity, trans, c, trigger, static_body) in bodies.iter() {
            let c = c.absolute(trans);
            if dyn_collider.collides_with(&c) {
                match (trigger, static_body) {
                    (Some(_), Some(_)) => {
                        warn!("StaticBody entity has Trigger component, this will not send trigger events!");
                    }
                    (Some(_), None) => {
                        writer.send(TriggerEvent {
                            receiver: body_entity,
                            emitter: dyn_entity,
                        });
                    }
                    (None, Some(_)) => {}
                    (None, None) => unreachable!(),
                }

                if static_body.is_some() {
                    let res_v = dyn_collider.resolution(&c);
                    dyn_trans.translation += Vec3::new(res_v.x, res_v.y, 0.);
                }
            }
        }
    }
}
