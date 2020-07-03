
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    RED = 0,
    BLUE = 1
}

impl Color {
    pub fn swap(self) -> Color {
        match self {
            Color::RED => Color::BLUE,
            Color::BLUE => Color::RED,
        }
    }
}
