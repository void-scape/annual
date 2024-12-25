use bevy::prelude::*;

pub fn srgb_from_hex(color: u32) -> Color {
    Color::srgb_u8(
        ((color >> 16) & 0xff) as u8,
        ((color >> 8) & 0xff) as u8,
        (color & 0xff) as u8,
    )
}
