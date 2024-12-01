use crate::{
    animation::{AnimationController, AnimationPlugin},
    asset_loading::loaded,
    cutscene::{CutsceneMovement, CutsceneVelocity},
};
use bevy::prelude::*;
use bevy_ecs_ldtk::app::LdtkEntityAppExt;
use bevy_ecs_ldtk::prelude::*;
use leafwing_input_manager::{
    plugin::InputManagerPlugin,
    prelude::{ActionState, InputMap},
    Actionlike, InputManagerBundle,
};
use std::hash::Hash;

/// Returns true if and only if the player is interacting
pub fn on_player_interact(query: Query<&ActionState<Action>, With<Player>>) -> bool {
    query
        .get_single()
        .is_ok_and(|a| a.just_pressed(&Action::Interact))
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InputManagerPlugin::<Action>::default(),
            AnimationPlugin::<PlayerAnimation>::default(),
        ))
        .register_ldtk_entity::<PlayerBundle>("Player")
        .add_systems(Update, (walk, animate_cutscene).run_if(loaded()));
    }
}

#[derive(Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    player: Player,
    izzy: crate::characters::Izzy,
    #[with(init_animation_controller)]
    animation: AnimationController<PlayerAnimation>,
    #[with(init_input_map)]
    input: InputManagerBundle<Action>,
    #[sprite_sheet_bundle]
    sprite_sheet: LdtkSpriteSheetBundle,
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

fn init_animation_controller(_: &EntityInstance) -> AnimationController<PlayerAnimation> {
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

fn init_input_map(_: &EntityInstance) -> InputManagerBundle<Action> {
    let input_map = InputMap::new([
        (Action::Walk(Direction::Up), KeyCode::KeyW),
        (Action::Walk(Direction::Down), KeyCode::KeyS),
        (Action::Walk(Direction::Left), KeyCode::KeyA),
        (Action::Walk(Direction::Right), KeyCode::KeyD),
        (Action::Interact, KeyCode::KeyE),
    ]);
    InputManagerBundle::with_map(input_map)
}

#[derive(Default, Component)]
pub struct Player;

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
        transform.translation.x += vel.x * time.delta_seconds();
        transform.translation.y += vel.y * time.delta_seconds();
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
