use crate::asset_loading::loaded;
use bevy::prelude::*;

mod debug;

#[derive(Debug)]
pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(debug::ShowCollision(false))
            .add_systems(
                Update,
                (
                    update_player_collision,
                    debug::debug_display_collider_wireframe,
                    debug::update_show_collision,
                    debug::debug_show_collision_color,
                )
                    .run_if(loaded()),
            );
    }
}

#[derive(Debug, Clone, Copy, Event)]
pub struct PlayerCollideEvent {
    /// The entity the player collided with.
    pub with: Entity,
}

/// A marker component that indicates this entity should
/// collide with the player.
#[derive(Debug, Component, Clone, Copy)]
pub struct CollideWithPlayer;

#[derive(Debug, Component, Clone, Copy)]
pub struct RemoveOnPlayerCollision;

pub trait CollidesWith<T> {
    fn collides_with(&self, other: &T) -> bool;
}

#[derive(Debug, Default, Clone, Copy, Component)]
pub struct StaticBody;

#[derive(Debug, Default, Clone, Copy, Component)]
pub struct DynamicBody;

/// To check for collisions, first convert this enum into an [AbsoluteCollider]
/// with [Collider::absolute].
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub enum Collider {
    Rect(RectCollider),
    Circle(CircleCollider),
}

impl Collider {
    pub fn from_rect(rect: RectCollider) -> Self {
        Self::Rect(rect)
    }

    pub fn from_circle(circle: CircleCollider) -> Self {
        Self::Circle(circle)
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
}

impl CollidesWith<CircleCollider> for RectCollider {
    fn collides_with(&self, other: &CircleCollider) -> bool {
        other.collides_with(self)
    }
}

pub fn update_player_collision(
    static_bodies: Query<(&Transform, &Collider), With<StaticBody>>,
    dynamic_bodies: Query<(&Transform, &Collider), With<DynamicBody>>,
) {
    for (dyn_t, dyn_c) in dynamic_bodies.iter() {
        let dyn_c = dyn_c.absolute(dyn_t);
        for (t, c) in static_bodies.iter() {
            if dyn_c.collides_with(&c.absolute(t)) {
                // error!("static body collision with dynamic body!");
            }
        }
    }
}
