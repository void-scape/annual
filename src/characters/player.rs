use crate::{
    animation::{AnimationController, AnimationPlugin},
    camera::MainCamera,
    characters::Izzy,
    collision::{trigger::TriggerLayer, Collider, DynamicBody},
    cutscene::{CutsceneMovement, CutsceneVelocity},
};
use bevy::prelude::*;
use leafwing_input_manager::{
    plugin::InputManagerPlugin,
    prelude::{ActionState, InputMap},
    Actionlike,
};
use std::hash::Hash;

/// Returns true if and only if the player is interacting
pub fn on_player_interact(query: Option<Single<&ActionState<Action>, With<Player>>>) -> bool {
    query.is_some_and(|a| a.into_inner().just_pressed(&Action::Interact))
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InputManagerPlugin::<Action>::default(),
            AnimationPlugin::<PlayerAnimation>::default(),
        ))
        .add_systems(PreUpdate, init_camera)
        .add_systems(FixedUpdate, (walk, animate_cutscene));
    }
}

fn init_camera(
    mut commands: Commands,
    camera: Query<Entity, With<MainCamera>>,
    player: Query<Entity, Added<Izzy>>,
) {
    if let Ok(player) = player.get_single() {
        if let Ok(camera) = camera.get_single() {
            commands
                .entity(camera)
                .insert(crate::camera::Binded(player));
        } else {
            error!("could not bind camera to player on startup: no camera found");
        }
    }
}

// #[derive(Bundle, Default, LdtkEntity)]
// pub struct NpcBundle {
//     massive: crate::collision::Massive,
//     #[with(init_dyn_body)]
//     dynamic_body: DynamicBodyBundle,
//     // #[with(init_static_body)]
//     // dynamic_body: crate::collision::StaticBodyBundle,
//     #[sprite_sheet_bundle]
//     sprite_sheet: LdtkSpriteSheetBundle,
// }

// fn init_static_body(_: &EntityInstance) -> crate::collision::StaticBodyBundle {
//     crate::collision::StaticBodyBundle {
//         collider: Collider::from_circle(Vec2::ZERO, 10.),
//         // collider: Collider::from_rect(Vec2::ZERO, Vec2::splat(10.)),
//         ..Default::default()
//     }
// }

#[derive(Default, Component)]
#[require(Izzy, AnimationController<PlayerAnimation>(|| AnimationController::new(
        5.0,
        [
            (PlayerAnimation::Idle, (0, 1)),
            (PlayerAnimation::Walk(Direction::Up), (8, 12)),
            (PlayerAnimation::Walk(Direction::Right), (0, 4)),
            (PlayerAnimation::Walk(Direction::Left), (4, 8)),
            (PlayerAnimation::Walk(Direction::Down), (4, 8)),
        ],
    )))]
#[require(ActionState<Action>, InputMap<Action>(|| InputMap::new([
        (Action::Walk(Direction::Up), KeyCode::KeyW),
        (Action::Walk(Direction::Down), KeyCode::KeyS),
        (Action::Walk(Direction::Left), KeyCode::KeyA),
        (Action::Walk(Direction::Right), KeyCode::KeyD),
    ])
    .with_one_to_many(Action::Interact, [KeyCode::KeyE, KeyCode::Space])))]
#[require(TriggerLayer(|| TriggerLayer(0)), DynamicBody, Collider(|| Collider::from_circle(Vec2::ZERO, 10.)))]
pub struct Player;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum PlayerAnimation {
    Walk(Direction),
    Idle,
}

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    Walk(Direction),
    Interact,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn into_unit_vec2(self) -> Vec2 {
        match self {
            Self::Up => Vec2::Y,
            Self::Down => Vec2::NEG_Y,
            Self::Left => Vec2::NEG_X,
            Self::Right => Vec2::X,
        }
    }

    pub fn from_velocity(velocity: Vec2) -> Self {
        #[allow(clippy::collapsible_else_if)]
        if velocity.x.abs() > velocity.y.abs() {
            if velocity.x > 0.0 {
                Direction::Right
            } else {
                Direction::Left
            }
        } else {
            if velocity.y > 0.0 {
                Direction::Up
            } else {
                Direction::Down
            }
        }
    }
}

fn walk(
    mut player: Query<
        (
            &ActionState<Action>,
            &mut Transform,
            &mut AnimationController<PlayerAnimation>,
        ),
        (With<Player>, Without<CutsceneMovement>),
    >,
    time: Res<Time>,
) {
    if let Ok((action_state, mut transform, mut animation)) = player.get_single_mut() {
        let mut vel = Vec2::ZERO;

        for action in action_state.get_released() {
            if let Action::Walk(dir) = action {
                match dir {
                    Direction::Up | Direction::Down => {
                        vel.y = 0.0;
                    }
                    Direction::Left | Direction::Right => {
                        vel.x = 0.0;
                    }
                }
            }
        }

        let actions = action_state.get_pressed();
        for action in actions.iter() {
            if let Action::Walk(dir) = action {
                vel += dir.into_unit_vec2();
            }
        }

        if !actions.is_empty()
            && vel != Vec2::ZERO
            && !actions.iter().any(|a| {
                if let Action::Walk(dir) = a {
                    Some(&PlayerAnimation::Walk(*dir)) == animation.active_animation()
                } else {
                    false
                }
            })
        {
            animation.set_animation(PlayerAnimation::Walk(
                *actions
                    .iter()
                    .find_map(|a| match a {
                        Action::Walk(dir) => Some(dir),
                        _ => None,
                    })
                    .unwrap(),
            ));
        } else if actions.is_empty() || vel == Vec2::ZERO {
            animation.set_animation(PlayerAnimation::Idle);
        }

        const PLAYER_SPEED: f32 = 50.0;
        vel = vel.clamp_length_max(1.0) * PLAYER_SPEED;
        transform.translation.x += vel.x * time.delta_secs();
        transform.translation.y += vel.y * time.delta_secs();
    }
}

fn animate_cutscene(
    mut player: Query<
        (&mut AnimationController<PlayerAnimation>, &CutsceneVelocity),
        (With<Player>, With<CutsceneMovement>),
    >,
    mut last_direction: Local<Option<Direction>>,
) {
    if let Ok((mut animation, velocity)) = player.get_single_mut() {
        let vel = velocity.0.xy();

        if vel == Vec2::ZERO {
            if last_direction.is_some() {
                *last_direction = None;
                animation.set_animation(PlayerAnimation::Idle);
            }
        } else {
            let direction = Direction::from_velocity(vel);

            let update = match *last_direction {
                None => {
                    *last_direction = Some(direction);
                    true
                }
                Some(ld) if ld != direction => {
                    *last_direction = Some(direction);
                    true
                }
                _ => false,
            };

            if update {
                animation.set_animation(PlayerAnimation::Walk(Direction::from_velocity(vel)));
            }
        }
    }
}
