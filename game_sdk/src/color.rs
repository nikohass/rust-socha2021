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
            Color::RED => Color::GREEN,
            Color::BLUE => Color::YELLOW,
            Color::YELLOW => Color::RED,
            Color::GREEN => Color::BLUE,
        }
    }

    pub fn previous(self) -> Color {
        match self {
            Color::BLUE => Color::GREEN,
            Color::YELLOW => Color::BLUE,
            Color::GREEN => Color::RED,
            Color::RED => Color::YELLOW,
        }
    }

    pub fn to_string(self) -> String {
        match self {
            Color::RED => "RED (Team TWO)".to_string(),
            Color::BLUE => "BLUE (Team ONE)".to_string(),
            Color::YELLOW => "YELLOW (Team ONE)".to_string(),
            Color::GREEN => "GREEN (Team TWO)".to_string(),
        }
    }
}
