use crate::{
    animation::{AnimationController, AnimationPlugin},
    characters::Izzy,
    cutscene::{CutsceneMovement, CutsceneVelocity},
    gfx::camera::{bind_camera, CameraOffset, MainCamera},
    physics::prelude::*,
    TILE_SIZE,
};
use bevy::prelude::*;
use leafwing_input_manager::{
    plugin::InputManagerPlugin,
    prelude::{ActionState, InputMap},
    Actionlike,
};
use std::hash::Hash;

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
    camera: Option<Single<Entity, With<MainCamera>>>,
    player: Option<Single<Entity, Added<Izzy>>>,
) {
    if player.is_some() {
        if camera.is_some() {
            commands.run_system_cached(bind_camera::<Izzy>);
        } else {
            error!("could not bind camera to player on startup: no camera found");
        }
    }
}

#[derive(Default, Component)]
#[require(Izzy, AnimationController<PlayerAnimation>(animation_controller))]
#[require(ActionState<Action>, InputMap<Action>(input_map))]
#[require(TriggerLayer(|| TriggerLayer(0)), DynamicBody, Collider(collider))]
#[require(CameraOffset(|| CameraOffset(Vec2::new(TILE_SIZE, -TILE_SIZE))))]
pub struct Player;

fn animation_controller() -> AnimationController<PlayerAnimation> {
    AnimationController::new(
        15.0,
        [
            (PlayerAnimation::Idle, (50, 54)),
            (PlayerAnimation::Walk(Direction::Up), (20, 40)),
            (PlayerAnimation::Walk(Direction::Right), (45, 50)),
            (PlayerAnimation::Walk(Direction::Left), (40, 45)),
            (PlayerAnimation::Walk(Direction::Down), (0, 20)),
        ],
    )
}

fn input_map() -> InputMap<Action> {
    InputMap::new([
        (Action::Walk(Direction::Up), KeyCode::KeyW),
        (Action::Walk(Direction::Down), KeyCode::KeyS),
        (Action::Walk(Direction::Left), KeyCode::KeyA),
        (Action::Walk(Direction::Right), KeyCode::KeyD),
    ])
    .with_one_to_many(Action::Interact, [KeyCode::KeyE, KeyCode::Space])
}

fn collider() -> Collider {
    Collider::from_circle(Vec2::new(TILE_SIZE, -TILE_SIZE), TILE_SIZE / 2.)
}

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

        const PLAYER_SPEED: f32 = 80.0;
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
