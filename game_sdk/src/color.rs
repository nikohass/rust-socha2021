use std::fmt::{Display, Formatter, Result};

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    BLUE = 0,
    YELLOW = 1,
    RED = 2,
    GREEN = 3,
}

impl Color {
    pub fn next(self) -> Color {
        match self {
            Color::BLUE => Color::YELLOW,
            Color::YELLOW => Color::RED,
            Color::RED => Color::GREEN,
            Color::GREEN => Color::BLUE,
        }
    }

    pub fn previous(self) -> Color {
        match self {
            Color::BLUE => Color::GREEN,
            Color::GREEN => Color::RED,
            Color::RED => Color::YELLOW,
            Color::YELLOW => Color::BLUE,
        }
    }

    #[inline(always)]
    pub fn team_f32(&self) -> f32 {
        // returns 1.0 for team one and -1.0 for team two
        f32::from_bits(0x3F800000 | ((*self as u32 & 1) << 31))
    }

    #[inline(always)]
    pub fn team_i16(&self) -> i16 {
        ((*self as i16 & 0b1) << 1) - 1
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            match self {
                Color::BLUE => "BLUE (Team ONE)".to_string(),
                Color::YELLOW => "YELLOW (Team TWO)".to_string(),
                Color::RED => "RED (Team ONE)".to_string(),
                Color::GREEN => "GREEN (Team TWO)".to_string(),
            }
        )
    }
}
