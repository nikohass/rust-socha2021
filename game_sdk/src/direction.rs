#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Direction {
    LEFT = 0,
    UP = 1,
    RIGHT = 2,
    DOWN = 3
}

impl Direction {
    pub fn to_string(&self) -> String {
        match self {
            Direction::LEFT => "LEFT".to_string(),
            Direction::UP => "UP".to_string(),
            Direction::RIGHT => "RIGHT".to_string(),
            Direction::DOWN => "DOWN".to_string()
        }
    }
    pub fn from_u16(n: u16) -> Direction {
        match n {
            0 => Direction::LEFT,
            1 => Direction::UP,
            2 => Direction::RIGHT,
            3 => Direction::DOWN,
            _ => panic!("Invalid direction")
        }
    }
    pub fn clockwise(&self) -> Direction {
        match self {
            Direction::LEFT => Direction::UP,
            Direction::UP => Direction::RIGHT,
            Direction::RIGHT => Direction::DOWN,
            Direction::DOWN => Direction::LEFT
        }
    }
    pub fn anticlockwise(&self) -> Direction {
        match self {
            Direction::LEFT => Direction::DOWN,
            Direction::UP => Direction::LEFT,
            Direction::RIGHT => Direction::UP,
            Direction::DOWN => Direction::RIGHT
        }
    }
    pub fn mirror(&self) -> Direction {
        match self {
            Direction::LEFT => Direction::RIGHT,
            Direction::RIGHT => Direction::LEFT,
            Direction::UP => Direction::DOWN,
            Direction::DOWN => Direction::UP,
        }
    }
}

pub const DIRECTIONS: [Direction; 4] = [
    Direction::LEFT,
    Direction::UP,
    Direction::RIGHT,
    Direction::DOWN,
];
