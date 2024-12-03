use bevy::{app::MainScheduleOrder, ecs::schedule::ScheduleLabel, prelude::*};
use spatial::{SpatialHash, StaticBodyData, StaticBodyStorage};
use std::cmp::Ordering;

mod debug;
mod spatial;
pub mod trigger;

#[derive(ScheduleLabel, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Physics;

#[derive(Debug)]
pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(Physics);
        app.world_mut()
            .resource_mut::<MainScheduleOrder>()
            .insert_after(Update, Physics);

        app.add_event::<trigger::TriggerEvent>()
            .insert_resource(trigger::TriggerLayerRegistry::default())
            .insert_resource(debug::ShowCollision(false))
            .add_systems(Startup, spatial::init_static_body_storage)
            .add_systems(
                Physics,
                (
                    (trigger::register_trigger_layers, trigger::handle_triggers),
                    (
                        spatial::store_static_body_in_spatial_map,
                        handle_collisions,
                        handle_dynamic_body_collsions,
                    )
                        .chain(),
                    debug::debug_display_collider_wireframe,
                    debug::update_show_collision,
                    (
                        debug::debug_show_collision_color,
                        debug::debug_show_trigger_color,
                    )
                        .chain(),
                ),
            );
    }
}

/// Marks this entity as having a static position throughout the lifetime of the program.
///
/// All [`StaticBody`] entities are added to a [`spatial::SpatialHash`] after spawning.
///
/// Moving a static body entity will NOT result in their collision being updated.
#[derive(Debug, Default, Clone, Copy, Component)]
pub struct StaticBody;

#[derive(Default, Bundle)]
pub struct StaticBodyBundle {
    pub static_body: StaticBody,
    pub collider: Collider,
}

#[derive(Debug, Default, Clone, Copy, Component)]
pub struct DynamicBody;

/// Prevents a dynamic body entity from being pushed.
#[derive(Debug, Default, Clone, Copy, Component)]
pub struct Massive;

#[derive(Default, Bundle)]
pub struct DynamicBodyBundle {
    pub dynamic_body: DynamicBody,
    pub collider: Collider,
}

/// To check for collisions, first convert this enum into an [`AbsoluteCollider`]
/// with [`Collider::absolute`].
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

    pub fn max_x(&self) -> f32 {
        match self {
            Self::Rect(rect) => rect.tl.x + rect.size.x,
            Self::Circle(circle) => circle.position.x + circle.radius,
        }
    }

    pub fn min_x(&self) -> f32 {
        match self {
            Self::Rect(rect) => rect.tl.x,
            Self::Circle(circle) => circle.position.x - circle.radius,
        }
    }

    pub fn max_y(&self) -> f32 {
        match self {
            Self::Rect(rect) => rect.tl.y + rect.size.y,
            Self::Circle(circle) => circle.position.y + circle.radius,
        }
    }

    pub fn min_y(&self) -> f32 {
        match self {
            Self::Rect(rect) => rect.tl.y,
            Self::Circle(circle) => circle.position.y - circle.radius,
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

pub trait CollidesWith<T> {
    fn collides_with(&self, other: &T) -> bool;
    fn resolution(&self, other: &T) -> Vec2;
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

    // TODO: this jitters like a bitch
    fn resolution(&self, other: &Self) -> Vec2 {
        let self_br = self.tl + self.size;
        let other_br = other.tl + other.size;

        // Calculate overlap in both dimensions
        let x_overlap = (self_br.x.min(other_br.x) - self.tl.x.max(other.tl.x)).max(0.);
        let y_overlap = (self_br.y.min(other_br.y) - self.tl.y.max(other.tl.y)).max(0.);

        // Calculate the center points of both rectangles
        let self_center = self.tl + self.size * 0.5;
        let other_center = other.tl + other.size * 0.5;

        // If no overlap in either dimension, return zero
        if x_overlap == 0. || y_overlap == 0. {
            return Vec2::ZERO;
        }

        // Determine which axis to resolve on (the one with smaller overlap)
        if x_overlap < y_overlap {
            // Resolve horizontally
            let dir = (self_center.x - other_center.x).signum();
            Vec2::new(x_overlap * dir, 0.)
        } else {
            // Resolve vertically
            let dir = (self_center.y - other_center.y).signum();
            Vec2::new(0., y_overlap * dir)
        }
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
        let combined_radii = self.radius + other.radius;
        distance <= combined_radii.powi(2)
    }

    fn resolution(&self, other: &Self) -> Vec2 {
        let diff = self.position - other.position;
        let distance_squared = diff.length_squared();
        let combined_radii = self.radius + other.radius;
        let combined_radii_squared = combined_radii * combined_radii;

        // If not overlapping, return zero vector
        if distance_squared >= combined_radii_squared {
            return Vec2::ZERO;
        }

        // Handle the case where circles are very close to same position
        const EPSILON: f32 = 0.0001;
        if distance_squared <= EPSILON {
            // Push to the right by combined radii
            return Vec2::new(combined_radii, 0.0);
        }

        let distance = distance_squared.sqrt();
        let overlap = combined_radii - distance;

        // Normalize diff without a separate division
        let direction = diff * (1.0 / distance);

        direction * (overlap + EPSILON)
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

fn handle_collisions(
    static_body_storage: Query<&SpatialHash<StaticBodyData>, With<StaticBodyStorage>>,
    mut dynamic_bodies: Query<(&mut Transform, &Collider), With<DynamicBody>>,
) {
    let Ok(map) = static_body_storage.get_single() else {
        error!("could not find static body storage");
        return;
    };

    for (mut transform, collider) in dynamic_bodies.iter_mut() {
        let original_collider = &collider;
        let mut collider = collider.absolute(&transform);

        for spatial::SpatialData { collider: sc, .. } in map.nearby_objects(&collider.position()) {
            if collider.collides_with(sc) {
                let res_v = collider.resolution(sc);
                transform.translation += Vec3::new(res_v.x, res_v.y, 0.);
                collider = original_collider.absolute(&transform);
            }
        }
    }
}

pub fn handle_dynamic_body_collsions(
    mut dynamic_bodies: Query<
        (Entity, &mut Transform, &Collider, Option<&Massive>),
        With<DynamicBody>,
    >,
) {
    let mut dynamic_bodies = dynamic_bodies.iter_mut().collect::<Vec<_>>();
    dynamic_bodies.sort_by_key(|(_, _, _, m)| {
        if m.is_some() {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    });

    let mut spatial = spatial::SpatialHash::new(32.);

    for (entity, transform, collider, massive) in dynamic_bodies.iter() {
        let absolute = collider.absolute(transform);
        spatial.insert(spatial::SpatialData {
            collider: absolute,
            data: (massive.cloned(), *collider),
            entity: *entity,
        });
    }

    for (entity, transform, collider, massive) in dynamic_bodies.iter_mut() {
        let original_collider = &collider;
        let mut collider = collider.absolute(transform);

        let mut update_active = false;

        // TODO: this shit is awful
        //
        // For some reason god forsaken, this will update twice even though the position in the hash is updated
        // before the other, overlapping entity updates itself.
        for spatial::SpatialData {
            entity: se,
            collider: sc,
            data: d,
            ..
        } in spatial.nearby_objects(&collider.position())
        {
            if *entity != *se && collider.collides_with(sc) && massive.is_none() {
                let res_v = collider.resolution(sc);
                transform.translation += Vec3::new(res_v.x, res_v.y, 0.);
                collider = original_collider.absolute(transform);
                update_active = true;

                if d.0.is_some() && massive.is_some() {
                    warn!("resolving collision between two massive bodies");
                }
            }
        }

        if update_active {
            for spatial::SpatialData {
                entity: se,
                collider: sc,
                ..
            } in spatial.objects_in_cell_mut(&collider.position())
            {
                if *se == *entity {
                    *sc = collider;
                    break;
                }
            }
        }
    }
}
