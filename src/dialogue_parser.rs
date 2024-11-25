use crate::dialogue_box::WAVE_MATERIAL_LAYER;
use bevy::{prelude::*, render::view::RenderLayers};
use winnow::{
    error::{ErrMode, ErrorKind, ParserError},
    prelude::*,
    stream::Stream,
    token::{take_until, take_while},
};

#[derive(Debug, Clone)]
pub struct DialogueTextSection {
    pub section: TextSection,
    pub effect: Option<Effect>,
}

/// Effects can be applied to text using the format: [<text-to-affect>](<effect>), where the effect
/// can be an [`Effect`] (snake_case, e.g. (<text>)[[wave]]).
pub fn parse_dialogue(input: &mut &str, style: TextStyle) -> Vec<DialogueTextSection> {
    let mut sections = Vec::new();
    while let Ok(section) = parse_text(input, style.clone()) {
        sections.push(section);
    }

    sections
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Effect {
    ColorEffect(ColorEffect),
    ShaderEffect(ShaderEffect),
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(unused)]
pub enum ColorEffect {
    Color(f32, f32, f32),
    Red,
    Green,
    Blue,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShaderEffect {
    Wave,
}

impl Effect {
    pub fn requires_shader(&self) -> bool {
        match self {
            Self::ShaderEffect(_) => true,
            _ => false,
        }
    }

    pub fn render_layer(&self) -> RenderLayers {
        match self {
            Self::ShaderEffect(effect) => match effect {
                ShaderEffect::Wave => RenderLayers::layer(WAVE_MATERIAL_LAYER),
            },
            val => panic!("should not request layer for non shader effects: {val:?}"),
        }
    }
}

fn parse_effect_text<'s>(input: &mut &'s str) -> PResult<&'s str> {
    take_until(1.., "]").parse_next(input)
}

fn parse_effect_desc(input: &mut &str) -> PResult<Effect> {
    let desc = take_until(1.., ")").parse_next(input)?;
    if desc.contains("(") {
        todo!();
    } else {
        Ok(match desc {
            "red" => Effect::ColorEffect(ColorEffect::Red),
            "green" => Effect::ColorEffect(ColorEffect::Green),
            "blue" => Effect::ColorEffect(ColorEffect::Blue),
            "wave" => Effect::ShaderEffect(ShaderEffect::Wave),
            _ => unimplemented!(),
        })
    }
}

fn parse_effect(
    input: &mut &str,
    font: Handle<Font>,
    font_size: f32,
) -> PResult<DialogueTextSection> {
    '['.parse_next(input)?;
    let effect_text = parse_effect_text(input)?;
    ']'.parse_next(input)?;
    '('.parse_next(input)?;
    let effect = parse_effect_desc(input)?;
    ')'.parse_next(input)?;

    let mut color = Color::WHITE;
    match effect {
        Effect::ColorEffect(effect) => match effect {
            ColorEffect::Red => color = Color::linear_rgb(1.0, 0.0, 0.0),
            ColorEffect::Green => color = Color::linear_rgb(0.0, 1.0, 0.0),
            ColorEffect::Blue => color = Color::linear_rgb(0.0, 0.0, 1.0),
            ColorEffect::Color(r, g, b) => color = Color::linear_rgb(r, g, b),
        },
        Effect::ShaderEffect(_) => {}
    }

    Ok(DialogueTextSection {
        section: TextSection::new(
            effect_text,
            TextStyle {
                font,
                font_size,
                color,
            },
        ),
        effect: Some(effect),
    })
}

fn parse_text(input: &mut &str, style: TextStyle) -> PResult<DialogueTextSection> {
    if let Some((_, token)) = input.peek_token() {
        match token {
            '[' => parse_effect(input, style.font.clone(), style.font_size),
            _ => Ok(DialogueTextSection {
                section: TextSection::new(
                    take_while(1.., |token: char| token != '[')
                        .parse_next(input)?
                        .to_owned(),
                    style,
                ),
                effect: None,
            }),
        }
    } else {
        Err(ErrMode::from_error_kind(input, ErrorKind::Complete))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let mut text = "Hello, World! [BLAHH](Red), I am joe.";
        let result = parse_dialogue(
            &mut text,
            TextStyle {
                font: Handle::weak_from_u128(0),
                font_size: 32.0,
                color: Color::WHITE,
            },
        );
        println!("{result:#?}");
    }
}
