use crate::{
    animation::{AnimationController, AnimationPlugin},
    characters::Izzy,
    collision::{trigger::TriggerLayer, Collider, DynamicBody},
    cutscene::{CutsceneMovement, CutsceneVelocity},
    gfx::{
        camera::{bind_camera, CameraOffset, MainCamera},
        post_processing::PostProcessCommand,
    },
    TILE_SIZE,
};
use bevy::{core_pipeline::bloom::Bloom, prelude::*};
use bevy_light_2d::prelude::*;
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
        .add_systems(PreUpdate, (init_camera, init_player))
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

fn from_hex(color: u32) -> Color {
    Color::srgb_u8(
        ((color >> 16) & 0xff) as u8,
        ((color >> 8) & 0xff) as u8,
        (color & 0xff) as u8,
    )
}

fn init_player(mut commands: Commands, player: Query<Entity, Added<Player>>) {
    for player in player.iter() {
        commands.entity(player).with_child((
            PointLight2d {
                color: from_hex(0xffeb57),
                intensity: 2.0,
                radius: 60.0,
                falloff: 100.,
                ..default()
            },
            Transform::from_xyz(TILE_SIZE, -TILE_SIZE, 0.),
        ));
        commands.post_process(AmbientLight2d {
            brightness: 0.4,
            color: from_hex(0x03193f),
            ..Default::default()
        });
        //commands.post_process(Bloom::NATURAL);
    }
}

fn animation_controller() -> AnimationController<PlayerAnimation> {
    AnimationController::new(
        5.0,
        [
            (PlayerAnimation::Idle, (0, 1)),
            (PlayerAnimation::Walk(Direction::Up), (8, 12)),
            (PlayerAnimation::Walk(Direction::Right), (0, 4)),
            (PlayerAnimation::Walk(Direction::Left), (4, 8)),
            (PlayerAnimation::Walk(Direction::Down), (4, 8)),
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

        const PLAYER_SPEED: f32 = 40.0;
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
