
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    RED = 0,
    BLUE = 1,
    YELLOW = 2,
    GREEN = 3,
}

impl Color {
    pub fn next(self) -> Color {
        match self {
            Color::RED => Color::BLUE,
            Color::BLUE => Color::YELLOW,
            Color::YELLOW => Color::GREEN,
            Color::GREEN => Color::RED
        }
    }

    pub fn previous(self) -> Color {
        match self {
            Color::BLUE => Color::RED,
            Color::YELLOW => Color::BLUE,
            Color::GREEN => Color::YELLOW,
            Color::RED => Color::GREEN
        }
    }

    pub fn to_string(self) -> String {
        match self {
            Color::RED => "RED".to_string(),
            Color::BLUE => "BLUE".to_string(),
            Color::YELLOW => "YELLOW".to_string(),
            Color::GREEN => "GREEN".to_string()
        }
    }
}
