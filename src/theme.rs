use ratatui::style::Color;

use crate::app::Phase;

pub const BASE: Color = Color::Rgb(0x1e, 0x1e, 0x2e);
pub const SURFACE0: Color = Color::Rgb(0x31, 0x32, 0x44);
pub const SURFACE1: Color = Color::Rgb(0x45, 0x47, 0x5a);
pub const OVERLAY0: Color = Color::Rgb(0x6c, 0x70, 0x86);
pub const OVERLAY1: Color = Color::Rgb(0x7f, 0x84, 0x9c);
pub const OVERLAY2: Color = Color::Rgb(0x93, 0x99, 0xb2);
pub const SUBTEXT0: Color = Color::Rgb(0xa6, 0xad, 0xc8);
pub const LAVENDER: Color = Color::Rgb(0xb4, 0xbe, 0xfe);
pub const TEAL: Color = Color::Rgb(0x94, 0xe2, 0xd5);
pub const GREEN: Color = Color::Rgb(0xa6, 0xe3, 0xa1);
pub const RED: Color = Color::Rgb(0xf3, 0x8b, 0xa8);

pub fn accent(phase: Phase) -> Color {
    match phase {
        Phase::Focus => RED,
        Phase::ShortBreak => GREEN,
        Phase::LongBreak => TEAL,
    }
}