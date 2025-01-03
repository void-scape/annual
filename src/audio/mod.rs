use bevy::prelude::*;
use bevy_pretty_text::type_writer::sound::WordEvent;
use bevy_seedling::{
    firewheel::{
        clock::ClockSeconds,
        param::{DeferredEvent, TimelineEvent},
    },
    AudioContext, RegisterParamsNode,
};

mod formants;

pub use formants::VoiceNode;
use rand::Rng;

pub struct AnnualAudioPlugin;

impl Plugin for AnnualAudioPlugin {
    fn build(&self, app: &mut App) {
        app.register_params_node::<VoiceNode>()
            .add_systems(Update, play_voice);
    }
}

fn play_voice(
    mut reader: EventReader<WordEvent>,
    mut voice: Single<&mut VoiceNode>,
    mut context: ResMut<AudioContext>,
) {
    for _ in reader.read() {
        let now = context.now();

        voice
            .gate
            .push(TimelineEvent::Deferred {
                value: 1.,
                time: now,
            })
            .unwrap();
        voice
            .gate
            .push(TimelineEvent::Deferred {
                value: 0.,
                time: now + ClockSeconds(0.15),
            })
            .unwrap();

        let freq = 400.;

        let mut rng = rand::thread_rng();
        let variation = rng.gen_range(-50f32..100f32);

        let _ = voice.pitch.push_curve(
            freq + variation,
            now,
            now + ClockSeconds(0.15),
            EaseFunction::Linear,
        );

        voice.formant.push(DeferredEvent::Deferred {
            value: rng.gen_range(0..5),
            time: now,
        });
    }
}
