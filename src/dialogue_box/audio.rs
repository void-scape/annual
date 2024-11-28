use crate::{FragmentExt, IntoFragment};
use bevy::prelude::*;
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

#[derive(Clone)]
pub struct TextSfxSettings {
    pub pitch: f32,
    pub pitch_variance: f32,
    pub trigger: Trigger,
}

#[derive(Clone)]
pub enum Trigger {
    /// Audio samples per second
    Rate(f32),
    OnCharacter,
    OnWord,
}

pub trait SetDialogueTextSfx {
    fn reveal_sfx(
        self,
        bundle: AudioBundle,
        settings: TextSfxSettings,
    ) -> impl IntoFragment<bevy_bits::DialogueBoxToken>;

    fn delete_sfx(
        self,
        bundle: AudioBundle,
        settings: TextSfxSettings,
    ) -> impl IntoFragment<bevy_bits::DialogueBoxToken>;
}

impl<T> SetDialogueTextSfx for T
where
    T: IntoFragment<bevy_bits::DialogueBoxToken>,
{
    fn reveal_sfx(
        self,
        bundle: AudioBundle,
        settings: TextSfxSettings,
    ) -> impl IntoFragment<bevy_bits::DialogueBoxToken> {
        self.set_resource(RevealedTextSfx { bundle, settings })
    }

    fn delete_sfx(
        self,
        bundle: AudioBundle,
        settings: TextSfxSettings,
    ) -> impl IntoFragment<bevy_bits::DialogueBoxToken> {
        self.set_resource(DeletedTextSfx { bundle, settings })
    }
}
