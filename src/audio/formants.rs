use std::sync::atomic::AtomicI32;

use bevy::prelude::*;
use bevy_seedling::firewheel::clock::ClockSeconds;
use bevy_seedling::firewheel::node::{AudioNodeProcessor, EventData, ProcessStatus};
use bevy_seedling::firewheel::param::{AudioParam, Timeline};
use bevy_seedling::firewheel::{ChannelConfig, ChannelCount};
use bevy_seedling::{firewheel, firewheel::node::AudioNode};
use fundsp::prelude::*;

fn adsr(
    attack: Shared,
    decay: Shared,
    sustain: Shared,
    release: Shared,
) -> An<EnvelopeIn<f32, impl FnMut(f32, &Frame<f32, U1>) -> f32 + Clone, U1, f32>> {
    let neg1 = -1.0;
    let zero = 0.0;

    let a = shared(zero);
    let b = shared(neg1);

    let attack_start = var(&a);
    let release_start = var(&b);
    envelope2(move |time, control| {
        if release_start.value() >= zero && control > zero {
            attack_start.set_value(time);
            release_start.set_value(neg1);
        } else if release_start.value() < zero && control <= zero {
            release_start.set_value(time);
        }
        let ads_value = ads(
            attack.value(),
            decay.value(),
            sustain.value(),
            time - attack_start.value(),
        );
        if release_start.value() < zero {
            ads_value
        } else {
            ads_value
                * clamp01(delerp(
                    release_start.value() + release.value(),
                    release_start.value(),
                    time,
                ))
        }
    })
}

fn ads<F: Float>(attack: F, decay: F, sustain: F, time: F) -> F {
    if time < attack {
        lerp(F::from_f64(0.0), F::from_f64(1.0), time / attack)
    } else {
        let decay_time = time - attack;
        if decay_time < decay {
            lerp(F::from_f64(1.0), sustain, decay_time / decay)
        } else {
            sustain
        }
    }
}

#[derive(bevy_seedling::AudioParam, Clone, Component)]
pub struct VoiceNode {
    pub pitch: Timeline<f32>,
    pub gate: Timeline<f32>,
    pub formant: firewheel::param::Deferred<i32>,
}

impl Default for VoiceNode {
    fn default() -> Self {
        Self::new()
    }
}

impl VoiceNode {
    pub fn new() -> VoiceNode {
        Self {
            pitch: Timeline::new(250.0),
            gate: Timeline::new(0.),
            formant: firewheel::param::Deferred::new(1),
        }
    }
}

impl From<VoiceNode> for Box<dyn AudioNode> {
    fn from(value: VoiceNode) -> Self {
        Box::new(value)
    }
}

impl AudioNode for VoiceNode {
    fn debug_name(&self) -> &'static str {
        "voice synthesizer"
    }

    fn info(&self) -> firewheel::node::AudioNodeInfo {
        firewheel::node::AudioNodeInfo {
            num_min_supported_inputs: ChannelCount::ZERO,
            num_max_supported_inputs: ChannelCount::ZERO,
            num_min_supported_outputs: ChannelCount::MONO,
            num_max_supported_outputs: ChannelCount::MONO,
            equal_num_ins_and_outs: false,
            default_channel_config: ChannelConfig {
                num_inputs: ChannelCount::ZERO,
                num_outputs: ChannelCount::MONO,
            },
            updates: false,
            uses_events: true,
        }
    }

    fn activate(
        &mut self,
        stream_info: &firewheel::StreamInfo,
        _: ChannelConfig,
    ) -> Result<Box<dyn firewheel::node::AudioNodeProcessor>, Box<dyn std::error::Error>> {
        let gate = shared(0.);

        let attack = shared(0.015);
        let decay = shared(0.01);
        let sustain = shared(0.6);
        let release = shared(0.05);

        let frequency = shared(200.);

        let adsr = var(&gate)
            >> adsr(
                attack.clone(),
                decay.clone(),
                sustain.clone(),
                release.clone(),
            );

        let formant_params: Vec<_> = SOPRANO[0]
            .iter()
            .map(|f| {
                let (f, g, b) = f.into_params();

                (shared(f), shared(g), shared(b))
            })
            .collect();

        let formants = busi::<U5, _, _>(|i| {
            let (freq, gain, q) = &formant_params[i as usize];
            (pass() | var(freq) | var(q)) >> (bandpass::<f32>() * var(gain))
        });

        let voice = var(&frequency) >> saw();
        let mut processor = Box::new(voice >> (formants * adsr) >> lowpass_hz(2000., 1.) * 0.75)
            as Box<dyn AudioUnit>;

        processor.set_sample_rate(stream_info.sample_rate as f64);

        let updater = move |params: &VoiceNode| {
            gate.set(params.gate.get());
            frequency.set(params.pitch.get());

            let vowel = params.formant.get().clamp(0, SOPRANO.len() as i32);
            let vowel = &SOPRANO[vowel as usize];

            for (i, (freq, q, gain)) in formant_params.iter().enumerate() {
                let (new_freq, new_q, new_gain) = vowel[i].into_params();

                freq.set(new_freq);
                q.set(new_q);
                gain.set(new_gain);
            }
        };

        Ok(Box::new(VoiceProcessor {
            params: self.clone(),
            graph: processor,
            updater: Box::new(updater),
        }))
    }
}

struct VoiceProcessor {
    params: VoiceNode,
    graph: Box<dyn AudioUnit>,
    updater: Box<dyn Fn(&VoiceNode) + Send + Sync>,
}

impl AudioNodeProcessor for VoiceProcessor {
    fn process(
        &mut self,
        _: &[&[f32]],
        outputs: &mut [&mut [f32]],
        events: firewheel::node::NodeEventIter,
        info: firewheel::node::ProcInfo,
    ) -> ProcessStatus {
        for event in events {
            if let EventData::Parameter(p) = event {
                let _ = self.params.patch(&p.data, &p.path);
            }
        }

        let time = info.clock_seconds;
        let increment = info.sample_rate_recip;

        for (frame, sample) in outputs[0].iter_mut().enumerate() {
            let time = time + ClockSeconds(increment * frame as f64);
            self.params.tick(time);
            (self.updater)(&self.params);

            *sample = self.graph.get_mono();
        }

        ProcessStatus::outputs_not_silent()
    }
}

fn db(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

#[derive(Clone, Copy)]
struct Formant {
    frequency: f32,
    amplitude: f32,
    bandwidth: f32,
}

impl Formant {
    fn into_params(self) -> (f32, f32, f32) {
        (
            self.frequency,
            db(self.amplitude),
            self.frequency / self.bandwidth,
        )
    }
}

const TENOR: [[Formant; 5]; 5] = [
    [
        Formant {
            frequency: 650.0,
            amplitude: 0.0,
            bandwidth: 80.0,
        },
        Formant {
            frequency: 1080.0,
            amplitude: -6.0,
            bandwidth: 90.0,
        },
        Formant {
            frequency: 2650.0,
            amplitude: -7.0,
            bandwidth: 120.0,
        },
        Formant {
            frequency: 2900.0,
            amplitude: -8.0,
            bandwidth: 130.0,
        },
        Formant {
            frequency: 3250.0,
            amplitude: -22.0,
            bandwidth: 140.0,
        },
    ],
    [
        Formant {
            frequency: 400.0,
            amplitude: 0.0,
            bandwidth: 70.0,
        },
        Formant {
            frequency: 1700.0,
            amplitude: -14.0,
            bandwidth: 80.0,
        },
        Formant {
            frequency: 2600.0,
            amplitude: -12.0,
            bandwidth: 100.0,
        },
        Formant {
            frequency: 3200.0,
            amplitude: -14.0,
            bandwidth: 120.0,
        },
        Formant {
            frequency: 3580.0,
            amplitude: -20.0,
            bandwidth: 120.0,
        },
    ],
    [
        Formant {
            frequency: 290.0,
            amplitude: 0.0,
            bandwidth: 40.0,
        },
        Formant {
            frequency: 1870.0,
            amplitude: -15.0,
            bandwidth: 90.0,
        },
        Formant {
            frequency: 2800.0,
            amplitude: -18.0,
            bandwidth: 100.0,
        },
        Formant {
            frequency: 3250.0,
            amplitude: -20.0,
            bandwidth: 120.0,
        },
        Formant {
            frequency: 3540.0,
            amplitude: -30.0,
            bandwidth: 120.0,
        },
    ],
    [
        Formant {
            frequency: 400.0,
            amplitude: 0.0,
            bandwidth: 40.0,
        },
        Formant {
            frequency: 800.0,
            amplitude: -10.0,
            bandwidth: 80.0,
        },
        Formant {
            frequency: 2600.0,
            amplitude: -12.0,
            bandwidth: 100.0,
        },
        Formant {
            frequency: 2800.0,
            amplitude: -12.0,
            bandwidth: 120.0,
        },
        Formant {
            frequency: 3000.0,
            amplitude: -26.0,
            bandwidth: 120.0,
        },
    ],
    [
        Formant {
            frequency: 350.0,
            amplitude: 0.0,
            bandwidth: 40.0,
        },
        Formant {
            frequency: 600.0,
            amplitude: -20.0,
            bandwidth: 60.0,
        },
        Formant {
            frequency: 2700.0,
            amplitude: -17.0,
            bandwidth: 100.0,
        },
        Formant {
            frequency: 2900.0,
            amplitude: -14.0,
            bandwidth: 120.0,
        },
        Formant {
            frequency: 3300.0,
            amplitude: -26.0,
            bandwidth: 120.0,
        },
    ],
];

const SOPRANO: [[Formant; 5]; 5] = [
    [
        Formant {
            frequency: 800.0,
            amplitude: 0.0,
            bandwidth: 80.0,
        },
        Formant {
            frequency: 1150.0,
            amplitude: -6.0,
            bandwidth: 90.0,
        },
        Formant {
            frequency: 2900.0,
            amplitude: -32.0,
            bandwidth: 120.0,
        },
        Formant {
            frequency: 3900.0,
            amplitude: -20.0,
            bandwidth: 130.0,
        },
        Formant {
            frequency: 4950.0,
            amplitude: -50.0,
            bandwidth: 140.0,
        },
    ],
    [
        Formant {
            frequency: 350.0,
            amplitude: 0.0,
            bandwidth: 60.0,
        },
        Formant {
            frequency: 2000.0,
            amplitude: -20.0,
            bandwidth: 100.0,
        },
        Formant {
            frequency: 2800.0,
            amplitude: -15.0,
            bandwidth: 120.0,
        },
        Formant {
            frequency: 3600.0,
            amplitude: -40.0,
            bandwidth: 150.0,
        },
        Formant {
            frequency: 4950.0,
            amplitude: -56.0,
            bandwidth: 200.0,
        },
    ],
    [
        Formant {
            frequency: 270.0,
            amplitude: 0.0,
            bandwidth: 60.0,
        },
        Formant {
            frequency: 2140.0,
            amplitude: -12.0,
            bandwidth: 90.0,
        },
        Formant {
            frequency: 2950.0,
            amplitude: -26.0,
            bandwidth: 100.0,
        },
        Formant {
            frequency: 3900.0,
            amplitude: -26.0,
            bandwidth: 120.0,
        },
        Formant {
            frequency: 4950.0,
            amplitude: -44.0,
            bandwidth: 120.0,
        },
    ],
    [
        Formant {
            frequency: 450.0,
            amplitude: 0.0,
            bandwidth: 70.0,
        },
        Formant {
            frequency: 800.0,
            amplitude: -11.0,
            bandwidth: 80.0,
        },
        Formant {
            frequency: 2830.0,
            amplitude: -22.0,
            bandwidth: 100.0,
        },
        Formant {
            frequency: 3800.0,
            amplitude: -22.0,
            bandwidth: 130.0,
        },
        Formant {
            frequency: 4950.0,
            amplitude: -50.0,
            bandwidth: 135.0,
        },
    ],
    [
        Formant {
            frequency: 325.0,
            amplitude: 0.0,
            bandwidth: 50.0,
        },
        Formant {
            frequency: 700.0,
            amplitude: -16.0,
            bandwidth: 60.0,
        },
        Formant {
            frequency: 2700.0,
            amplitude: -35.0,
            bandwidth: 170.0,
        },
        Formant {
            frequency: 3800.0,
            amplitude: -40.0,
            bandwidth: 180.0,
        },
        Formant {
            frequency: 4950.0,
            amplitude: -60.0,
            bandwidth: 200.0,
        },
    ],
];
