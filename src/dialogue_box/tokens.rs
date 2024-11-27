use super::text::*;

#[derive(Debug, Clone, macros::Fragment)]
pub enum DialogueBoxToken {
    Section(TextSection),
    Command(TextCommand),
}

impl DialogueBoxToken {
    pub fn parse_command(args: &'static str, cmd: &'static str) -> Self {
        match cmd {
            "red" => Self::Section(TextSection {
                text: RawText::Str(args),
                color: Some(TextColor::Red),
                effects: Vec::new(),
            }),
            "wave" => Self::Section(TextSection {
                text: RawText::Str(args),
                color: None,
                effects: vec![TextEffect::Wave],
            }),
            "pause" => Self::Command(TextCommand::Pause(
                args.parse::<f32>()
                    .unwrap_or_else(|e| panic!("invalid args `{args}` for cmd `{cmd}`: {e}")),
            )),
            "speed" => Self::Command(TextCommand::Speed(
                args.parse::<f32>()
                    .unwrap_or_else(|e| panic!("invalid args `{args}` for cmd `{cmd}`: {e}")),
            )),
            c => panic!("command `{c}` is unimplemented"),
        }
    }
}

#[derive(Debug, Default)]
struct TextWriter {
    font: bevy::asset::Handle<bevy::text::Font>,
    size: f32,
    default_color: bevy::color::Color,
    speed: f32,
}

impl TextWriter {
    pub fn display(&mut self, tokens: Vec<DialogueBoxToken>) {
        println!("{tokens:#?}");
    }
}

pub trait IntoDialogueBoxToken {
    fn into_token(self) -> DialogueBoxToken;

    fn effect(self, effect: TextEffect) -> Effect<Self>
    where
        Self: Sized,
    {
        Effect {
            token: self,
            effect,
        }
    }

    fn color(self, color: TextColor) -> Color<Self>
    where
        Self: Sized,
    {
        Color { token: self, color }
    }
}

pub struct Effect<T> {
    token: T,
    effect: TextEffect,
}

impl<T: IntoDialogueBoxToken> IntoDialogueBoxToken for Effect<T> {
    fn into_token(self) -> DialogueBoxToken {
        let mut token = self.token.into_token();

        if let DialogueBoxToken::Section(section) = &mut token {
            section.effects.push(self.effect);
        }

        token
    }
}

pub struct Color<T> {
    token: T,
    color: TextColor,
}

impl<T: IntoDialogueBoxToken> IntoDialogueBoxToken for Color<T> {
    fn into_token(self) -> DialogueBoxToken {
        let mut token = self.token.into_token();

        if let DialogueBoxToken::Section(section) = &mut token {
            section.color = Some(self.color);
        }

        token
    }
}

impl IntoDialogueBoxToken for &'static str {
    fn into_token(self) -> DialogueBoxToken {
        DialogueBoxToken::Section(TextSection::from(self))
    }
}

impl IntoDialogueBoxToken for DialogueBoxToken {
    fn into_token(self) -> DialogueBoxToken {
        self
    }
}

impl From<String> for DialogueBoxToken {
    fn from(value: String) -> Self {
        DialogueBoxToken::Section(value.into())
    }
}

impl From<&'static str> for DialogueBoxToken {
    fn from(value: &'static str) -> Self {
        DialogueBoxToken::Section(value.into())
    }
}
