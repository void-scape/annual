use bevy::prelude::*;
use std::time::Duration;

#[derive(Debug, Default, Component)]
pub struct TypeWriter {
    finished: bool,
    timer: Timer,
    sections: Vec<DialogueTextSection>,
    index: usize,
    pause: Option<f32>,
    tokens: Vec<TypeWriterToken>,
}

impl TypeWriter {
    /// TypeWriters are paused on creation.
    ///
    /// Use [`TypeWriter::start`] to begin typing, or [`TypeWriter::new_start`].
    #[inline]
    pub fn new(sections: Vec<DialogueTextSection>, chars_per_sec: f32) -> Self {
        let mut timer = Timer::from_seconds(1.0 / chars_per_sec, TimerMode::Repeating);
        timer.pause();

        Self {
            tokens: Self::gen_tokens(&sections),
            timer,
            sections,
            index: 0,
            finished: false,
            pause: None,
        }
    }

    #[inline]
    pub fn new_start(sections: Vec<DialogueTextSection>, chars_per_sec: f32) -> Self {
        let mut slf = Self::new(sections, chars_per_sec);
        slf.start();

        slf
    }

    #[inline]
    pub fn awaiting_pause_timeout(&self) -> bool {
        self.pause.is_some()
    }

    #[inline]
    pub fn pause_for(&mut self, duration: f32) {
        self.pause = Some(duration);
    }

    #[inline]
    pub fn sections(&self) -> &[DialogueTextSection] {
        &self.sections
    }

    #[inline]
    fn total_len(&self) -> usize {
        let mut total_len = 0;
        for section in self.sections.iter() {
            total_len += section.section.value.len();
        }

        total_len
    }

    #[inline]
    pub fn tick(&mut self, time: &Time, on_increment: impl FnOnce(&mut TypeWriter)) -> &mut Self {
        if let Some(dur) = &mut self.pause {
            *dur -= time.delta_seconds();

            if *dur <= 0.0 {
                self.pause = None;
            }
        } else {
            self.timer.tick(time.delta());

            if self.timer.just_finished() {
                if self.index >= self.total_len() {
                    self.finished = true;
                    self.timer.pause();
                } else {
                    self.index += 1;
                }

                on_increment(self);
            }
        }

        self
    }

    #[inline]
    pub fn on_finish(&self, on_finish: impl FnOnce()) {
        if self.finished {
            on_finish();
        }
    }

    #[inline]
    pub fn restart(&mut self) {
        self.reset().start();
    }

    #[inline]
    pub fn reset(&mut self) -> &mut Self {
        self.timer.pause();
        self.timer.reset();
        self.index = 0;
        self.finished = false;

        self
    }

    #[inline]
    pub fn with_sections(&mut self, sections: Vec<DialogueTextSection>) -> &mut Self {
        self.sections = sections;
        self
    }

    #[inline]
    pub fn with_speed(&mut self, chars_per_sec: f32) -> &mut Self {
        self.timer
            .set_duration(Duration::from_secs_f32(1.0 / chars_per_sec));
        self
    }

    #[inline]
    pub fn start(&mut self) {
        self.timer.unpause();
    }

    #[inline]
    pub fn finished(&self) -> bool {
        self.finished
    }

    #[inline]
    pub fn reveal_all_text(&mut self) {
        self.index = self.total_len();
    }

    #[inline]
    pub fn revealed_text(&self) -> Vec<DialogueTextSection> {
        let mut remaining_len = self.index;
        let mut sections = Vec::new();

        for section in self.sections.iter() {
            if section.section.value.len() > remaining_len {
                sections.push(DialogueTextSection {
                    section: TextSection::new(
                        section.section.value[..remaining_len].to_owned(),
                        section.section.style.clone(),
                    ),
                    effect: section.effect,
                });

                break;
            } else {
                remaining_len -= section.section.value.len();
                sections.push(section.clone());
            }
        }

        sections
    }

    #[inline]
    pub fn revealed_text_with_line_wrap(&self) -> Vec<TypeWriterToken> {
        let mut len = self.index;
        let mut tokens = Vec::new();

        for token in self.tokens.iter() {
            if let TypeWriterToken::Command(TypeWriterEffect::Pause(dur)) = token {
                if len == 1 {
                    tokens.push(token.clone());
                    break;
                }
            }

            if let TypeWriterToken::Dialogue(section) = &token {
                let section_len = section.section.value.len();

                if len < section_len {
                    let mut section = section.clone();
                    let mut i = 0;
                    while len + i < section_len
                        && section.section.value.as_bytes()[(len + i).saturating_sub(1)] != b' '
                    {
                        i += 1;
                    }
                    section.section.value = section.section.value[..len].to_owned();
                    for _ in 0..i {
                        section.section.value.push(' ');
                    }
                    tokens.push(TypeWriterToken::Dialogue(section));

                    break;
                } else {
                    len -= section_len;
                }
            }

            tokens.push(token.clone());
        }

        tokens
    }

    #[inline]
    fn gen_tokens(sections: &[DialogueTextSection]) -> Vec<TypeWriterToken> {
        let mut tokens = Vec::new();

        for section in sections.iter() {
            if section.effect.is_some_and(|e| e.is_type_writer_effect()) {
                tokens.push(TypeWriterToken::Command(match section.effect.unwrap() {
                    Effect::TypeWriterEffect(e) => e,
                    _ => unreachable!(),
                }));
            } else {
                tokens.push(TypeWriterToken::Dialogue(section.clone()));
            }
        }

        tokens
    }
}

#[derive(Debug, Clone)]
pub enum TypeWriterToken {
    Dialogue(DialogueTextSection),
    Command(TypeWriterEffect),
}
