#[derive(Debug, Default)]
struct TextWriter {
    font: bevy::asset::Handle<bevy::text::Font>,
    size: f32,
    default_color: bevy::color::Color,
    speed: f32,
}

impl TextWriter {
    pub fn display(&mut self, tokens: Vec<TextToken>) {
        println!("{tokens:#?}");
    }
}

pub trait IntoTextToken {
    fn into_token(self) -> TextToken;

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

impl<T: IntoTextToken> IntoTextToken for Effect<T> {
    fn into_token(self) -> TextToken {
        let mut token = self.token.into_token();

        if let TextToken::Section(section) = &mut token {
            section.effects.push(self.effect);
        }

        token
    }
}

pub struct Color<T> {
    token: T,
    color: TextColor,
}

impl<T: IntoTextToken> IntoTextToken for Color<T> {
    fn into_token(self) -> TextToken {
        let mut token = self.token.into_token();

        if let TextToken::Section(section) = &mut token {
            section.color = Some(self.color);
        }

        token
    }
}

impl IntoTextToken for &'static str {
    fn into_token(self) -> TextToken {
        TextToken::Section(TextSection::from(self))
    }
}

#[derive(Debug)]
pub enum TextToken {
    Section(TextSection),
    Command(TextCommand),
}

impl TextToken {
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
            _ => unimplemented!(),
        }
    }
}

impl IntoTextToken for TextToken {
    fn into_token(self) -> TextToken {
        self
    }
}

impl From<String> for TextToken {
    fn from(value: String) -> Self {
        TextToken::Section(TextSection::from(value))
    }
}

#[derive(Debug)]
pub enum TextCommand {
    Wave,
    Speed(f32),
}

#[derive(Debug, Clone)]
pub struct TextSection {
    pub text: RawText,
    pub color: Option<TextColor>,
    pub effects: Vec<TextEffect>,
}

impl From<&'static str> for TextSection {
    fn from(value: &'static str) -> Self {
        TextSection {
            text: RawText::Str(value),
            color: None,
            effects: Vec::new(),
        }
    }
}

impl From<String> for TextSection {
    fn from(value: String) -> Self {
        TextSection {
            text: RawText::String(value),
            color: None,
            effects: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RawText {
    Str(&'static str),
    String(String),
}

#[derive(Debug, Clone)]
pub enum TextEffect {
    Wave,
}

#[derive(Debug, Clone)]
pub enum TextColor {
    Red,
    Green,
    Blue,
}
