use super::bitboard::{Bitboard, Direction};

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PieceType {
    Monomino = 0,
    Domino = 1,

    ITromino = 2,
    LTromino = 3,

    ITetromino = 4,
    LTetromino = 5,
    TTetromino = 6,
    OTetromino = 7,
    ZTetromino = 8,

    FPentomino = 9,
    IPentomino = 10,
    LPentomino = 11,
    NPentomino = 12,
    PPentomino = 13,
    TPentomino = 14,
    UPentomino = 15,
    VPentomino = 16,
    WPentomino = 17,
    XPentomino = 18,
    YPentomino = 19,
    ZPentomino = 20
}

impl PieceType {
    pub fn to_string(&self) -> String {
        match self {
            PieceType::Monomino => "Monomino".to_string(),
            PieceType::Domino => "Domino".to_string(),

            PieceType::ITromino => "I-Tromino".to_string(),
            PieceType::LTromino => "L-Tromino".to_string(),

            PieceType::ITetromino => "I-Tetromino".to_string(),
            PieceType::LTetromino => "L-Tetromino".to_string(),
            PieceType::TTetromino => "T-Tetromino".to_string(),
            PieceType::OTetromino => "O-Tetromino".to_string(),
            PieceType::ZTetromino => "Z-Tetromino".to_string(),

            PieceType::FPentomino => "F-Pentomino".to_string(),
            PieceType::IPentomino => "I-Pentomino".to_string(),
            PieceType::LPentomino => "L-Pentomino".to_string(),
            PieceType::NPentomino => "N-Pentomino".to_string(),
            PieceType::PPentomino => "P-Pentomino".to_string(),
            PieceType::TPentomino => "T-Pentomino".to_string(),
            PieceType::UPentomino => "U-Pentomino".to_string(),
            PieceType::VPentomino => "V-Pentomino".to_string(),
            PieceType::WPentomino => "W-Pentomino".to_string(),
            PieceType::XPentomino => "X-Pentomino".to_string(),
            PieceType::YPentomino => "Y-Pentomino".to_string(),
            PieceType::ZPentomino => "Z-Pentomino".to_string()
        }
    }

    pub fn get_shape(&self, destination: u16) -> Bitboard {
        let to = destination & 511;
        let d = Direction::from_u16((destination >> 9) & 15);
        let mut bit = Bitboard::bit(to);

        match self {
            PieceType::Monomino => bit,
            PieceType::Domino => {
                match d {
                    Direction::RIGHT => bit | bit << 1,
                    Direction::LEFT => bit | bit >> 1,
                    Direction::DOWN => bit | bit << 21,
                    Direction::UP => bit | bit >> 21,
                }
            }
            PieceType::ITromino => {
                match d {
                    Direction::RIGHT => bit | bit << 1 | bit << 2,
                    Direction::LEFT => bit | bit >> 1 | bit >> 2,
                    Direction::DOWN => bit | bit << 21 | bit << 42,
                    Direction::UP => bit | bit >> 21 | bit >> 42,
                }
            }
            PieceType::LTromino => {
                let c = bit.neighbours_in_direction(d.mirror());
                if destination & 32768 == 0 {
                    return bit | c | c.neighbours_in_direction(d.clockwise())
                }
                bit | c | bit.neighbours_in_direction(d.clockwise())
            }
            PieceType::ITetromino => {
                bit |= match d {
                    Direction::RIGHT => bit << 1,
                    Direction::LEFT => bit >> 1,
                    Direction::DOWN => bit << 21,
                    Direction::UP => bit >> 21,
                };
                match d {
                    Direction::RIGHT => bit | bit << 2,
                    Direction::LEFT => bit | bit >> 2,
                    Direction::DOWN => bit | bit << 42,
                    Direction::UP => bit | bit >> 42,
                }
            }
            PieceType::OTetromino => {
                let shape = bit | bit.neighbours_in_direction(d.clockwise());
                shape | shape.neighbours_in_direction(d.mirror())
            }
            /*

            PieceType::LTetromino =>
            PieceType::TTetromino =>
            PieceType::ZTetromino =>
            */
            PieceType::IPentomino => {
                bit |= match d {
                    Direction::RIGHT => bit << 1,
                    Direction::LEFT => bit >> 1,
                    Direction::DOWN => bit << 21,
                    Direction::UP => bit >> 21,
                };
                bit |= match d {
                    Direction::RIGHT => bit << 2,
                    Direction::LEFT => bit >> 2,
                    Direction::DOWN => bit << 42,
                    Direction::UP => bit >> 42,
                };
                match d {
                    Direction::RIGHT => bit | bit << 1,
                    Direction::LEFT => bit | bit >> 1,
                    Direction::DOWN => bit | bit << 21,
                    Direction::UP => bit | bit >> 21,
                }
            }

            _ => bit,
            /*

            PieceType::FPentomino =>
            PieceType::LPentomino =>
            PieceType::NPentomino =>
            PieceType::PPentomino =>
            PieceType::TPentomino =>
            PieceType::UPentomino =>
            PieceType::VPentomino =>
            PieceType::WPentomino =>
            PieceType::XPentomino =>
            PieceType::YPentomino =>
            PieceType::ZPentomino =>*/
        }
    }
}
