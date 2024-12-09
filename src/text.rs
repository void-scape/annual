use bevy::prelude::*;
use std::borrow::Cow;
use std::ops::Range;

pub struct TypeWriterSection {
    pub text: Text,
    pub commands: &'static [IndexedCommand],
}

#[derive(Debug, Clone, Copy)]
pub struct IndexedCommand {
    pub index: usize,
    pub command: TypeWriterCommand,
}

#[derive(Debug, Clone, Copy)]
pub enum TypeWriterCommand {
    Clear,
    AwaitClear,
    ClearAfter(f32),
    /// Relative speed
    Speed(f32),
    Pause(f32),
    Delete(usize),
}

/// String with a collection of modifiers.
#[derive(Debug, Clone)]
pub struct Text {
    pub value: Cow<'static, str>,
    pub modifiers: &'static [TextMod],
}

impl Text {
    pub fn from_value(value: String) -> Self {
        Self {
            value: Cow::Owned(value),
            modifiers: &[],
        }
    }
}

impl From<String> for Text {
    fn from(value: String) -> Self {
        Self {
            value: Cow::Owned(value),
            modifiers: &[],
        }
    }
}

impl From<&'static str> for Text {
    fn from(value: &'static str) -> Self {
        Self {
            value: Cow::Borrowed(value),
            modifiers: &[],
        }
    }
}

/// Text modifier that applies to a [`Text`] section.
#[derive(Debug, Clone)]
pub struct TextModSection {
    pub start: usize,
    pub end: usize,
    pub text_mod: TextMod,
}

/// Modifies visual qualities of [`Text`].
#[derive(Debug, Clone, Copy)]
pub enum TextMod {
    Color(Srgba),
    Wave,
    Shake,
}
