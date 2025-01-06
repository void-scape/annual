use crate::{
    animation::{AnimationController, AnimationPlugin},
    characters::Izzy,
    cutscene::{CutsceneMovement, CutsceneVelocity},
    gfx::{
        camera::{bind_camera, CameraOffset, MainCamera},
        zorder::YOrigin,
    },
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
        .add_systems(
            FixedUpdate,
            ((walk, smooth_camera_offset).chain(), animate_cutscene),
        );
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

const PLAYER_CAM_OFFSET: Vec2 = Vec2::new(TILE_SIZE, -TILE_SIZE);

#[derive(Default, Component)]
#[require(Izzy, AnimationController<PlayerAnimation>(animation_controller), Direction)]
#[require(ActionState<Action>, InputMap<Action>(input_map))]
#[require(TriggerLayer(|| TriggerLayer(0)), DynamicBody, Collider(collider))]
#[require(CameraOffset(|| CameraOffset(PLAYER_CAM_OFFSET)))]
#[require(YOrigin(|| YOrigin(-TILE_SIZE * 1.9)))]
pub struct Player;

fn animation_controller() -> AnimationController<PlayerAnimation> {
    AnimationController::new(
        12.,
        [
            (PlayerAnimation::Idle(Direction::Right), (0, 1)),
            (PlayerAnimation::Walk(Direction::Right), (1, 11)),
            (PlayerAnimation::Idle(Direction::Left), (11, 12)),
            (PlayerAnimation::Walk(Direction::Left), (12, 22)),
            (PlayerAnimation::Idle(Direction::Down), (22, 23)),
            (PlayerAnimation::Walk(Direction::Down), (23, 33)),
            (PlayerAnimation::Idle(Direction::Up), (33, 34)),
            (PlayerAnimation::Walk(Direction::Up), (34, 44)),
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
    Collider::from_circle(Vec2::new(TILE_SIZE, -TILE_SIZE * 1.75), TILE_SIZE / 3.5)
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum PlayerAnimation {
    Walk(Direction),
    Idle(Direction),
}

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    Walk(Direction),
    Interact,
}

#[derive(Default, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect, Component)]
pub enum Direction {
    Up,
    #[default]
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

fn smooth_camera_offset(player: Single<(&Direction, &mut CameraOffset)>) {
    let (direction, mut cam_offset) = player.into_inner();

    let target = PLAYER_CAM_OFFSET + direction.into_unit_vec2() * TILE_SIZE;

    // gradually approach the target offset
    let delta = (target - cam_offset.0) * 0.05;
    cam_offset.0 += delta;
}

fn walk(
    mut player: Query<
        (
            &ActionState<Action>,
            &mut Transform,
            &mut AnimationController<PlayerAnimation>,
            &mut Direction,
        ),
        (With<Player>, Without<CutsceneMovement>),
    >,
    time: Res<Time>,
) {
    if let Ok((action_state, mut transform, mut animation, mut last_dir)) = player.get_single_mut()
    {
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
            let dir = *actions
                .iter()
                .find_map(|a| match a {
                    Action::Walk(dir) => Some(dir),
                    _ => None,
                })
                .unwrap();
            animation.set_animation(PlayerAnimation::Walk(dir));
            *last_dir = dir;
        } else if actions.is_empty() || vel == Vec2::ZERO {
            animation.set_animation(PlayerAnimation::Idle(*last_dir));
        }

        const PLAYER_SPEED: f32 = 80.0;
        vel = vel.clamp_length_max(1.0) * PLAYER_SPEED;
        transform.translation.x += vel.x * time.delta_secs();
        transform.translation.y += vel.y * time.delta_secs();
    }
}

// TODO: need some notion of look direction
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
            if let Some(dir) = *last_direction {
                animation.set_animation(PlayerAnimation::Idle(dir));
                *last_direction = None;
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
