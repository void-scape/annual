#![allow(unused)]
use super::{BoxContext, DialogueBox, IntoBox};
use crate::FragmentExt;
use bevy::{audio::PlaybackMode, prelude::*};
use rand::Rng;

/// [`AudioSourceBundle<AudioSource>`] that defines `revealed` text sfx for a dialogue box.
#[derive(Component, Default, Clone)]
pub struct RevealedTextSfx {
    pub bundle: bevy::audio::AudioBundle,
    pub settings: TextSfxSettings,
}

impl RevealedTextSfx {
    pub fn bundle(&self) -> bevy::audio::AudioBundle {
        let mut bundle = self.bundle.clone();
        bundle.settings.mode = PlaybackMode::Despawn;
        bundle.settings.speed = self.settings.pitch
            + if self.settings.pitch_variance != 0.0 {
                rand::thread_rng()
                    .gen_range(-self.settings.pitch_variance..self.settings.pitch_variance)
            } else {
                0.0
            };

        bundle
    }
}

/// [`AudioSourceBundle<AudioSource>`] that defines `deleted` text sfx for a dialogue box.
#[derive(Component, Default, Clone)]
pub struct DeletedTextSfx {
    pub bundle: bevy::audio::AudioBundle,
    pub settings: TextSfxSettings,
}

impl DeletedTextSfx {
    pub fn bundle(&self) -> bevy::audio::AudioBundle {
        let mut bundle = self.bundle.clone();
        bundle.settings.mode = PlaybackMode::Despawn;
        bundle.settings.speed = self.settings.pitch
            + if self.settings.pitch_variance != 0.0 {
                rand::thread_rng()
                    .gen_range(-self.settings.pitch_variance..self.settings.pitch_variance)
            } else {
                0.0
            };

        bundle
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextSfxSettings {
    pub pitch: f32,
    pub pitch_variance: f32,
    pub trigger: Trigger,
}

impl Default for TextSfxSettings {
    fn default() -> Self {
        Self {
            pitch: 1.0,
            pitch_variance: 0.0,
            trigger: Trigger::Rate(1.0 / 18.0),
        }
    }
}

impl TextSfxSettings {
    pub fn from_trigger(trigger: Trigger) -> Self {
        Self {
            trigger,
            ..Default::default()
        }
    }

    pub fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch;
        self
    }

    pub fn with_variance(mut self, pitch_variance: f32) -> Self {
        self.pitch_variance = pitch_variance;
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Trigger {
    /// Audio samples per second
    Rate(f32),
    OnCharacter,
    OnWord,
}

pub trait SetDialogueTextSfx {
    fn reveal_sfx<C>(self, bundle: AudioBundle, settings: TextSfxSettings) -> impl IntoBox<C>
    where
        C: Component,
        Self: IntoBox<C>;

    fn delete_sfx<C>(self, bundle: AudioBundle, settings: TextSfxSettings) -> impl IntoBox<C>
    where
        C: Component,
        Self: IntoBox<C>;
}

impl<T> SetDialogueTextSfx for T {
    fn reveal_sfx<C>(self, bundle: AudioBundle, settings: TextSfxSettings) -> impl IntoBox<C>
    where
        C: Component,
        Self: IntoBox<C>,
    {
        self.on_start_ctx(
            move |ctx: In<BoxContext<C>>,
                  mut boxes: Query<&mut RevealedTextSfx, With<DialogueBox>>| {
                if let Ok(mut sfx) = boxes.get_mut(ctx.entity()) {
                    sfx.bundle = bundle.clone();
                    sfx.settings = settings;
                } else {
                    warn!("could not set reveal sfx for box: {}", ctx.entity());
                }
            },
        )
    }

    fn delete_sfx<C>(self, bundle: AudioBundle, settings: TextSfxSettings) -> impl IntoBox<C>
    where
        C: Component,
        Self: IntoBox<C>,
    {
        self.on_start_ctx(
            move |ctx: In<BoxContext<C>>,
                  mut boxes: Query<&mut DeletedTextSfx, With<DialogueBox>>| {
                if let Ok(mut sfx) = boxes.get_mut(ctx.entity()) {
                    sfx.bundle = bundle.clone();
                    sfx.settings = settings;
                } else {
                    warn!("could not set delete sfx for box: {}", ctx.entity());
                }
            },
        )
    }
}
