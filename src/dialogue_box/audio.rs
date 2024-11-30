use super::IntoBox;
use crate::{FragmentExt, IntoFragment};
use bevy::{audio::PlaybackMode, prelude::*};
use rand::Rng;

/// [`AudioSourceBundle<AudioSource>`] that globally defines `revealed` text sfx for all dialogue
/// boxes.
#[derive(Resource, Clone)]
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

/// [`AudioSourceBundle<AudioSource>`] that globally defines `deleted` text sfx for all dialogue
/// boxes.
#[derive(Resource, Clone)]
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
    fn reveal_sfx(self, bundle: AudioBundle, settings: TextSfxSettings) -> impl IntoBox;

    fn delete_sfx(self, bundle: AudioBundle, settings: TextSfxSettings) -> impl IntoBox;
}

impl<T> SetDialogueTextSfx for T
where
    T: IntoBox,
{
    fn reveal_sfx(self, bundle: AudioBundle, settings: TextSfxSettings) -> impl IntoBox {
        self.set_resource(RevealedTextSfx { bundle, settings })
    }

    fn delete_sfx(self, bundle: AudioBundle, settings: TextSfxSettings) -> impl IntoBox {
        self.set_resource(DeletedTextSfx { bundle, settings })
    }
}
