
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

    pub fn piece_size(&self) -> u8 {
        match self {
            PieceType::Monomino => 1,
            PieceType::Domino => 2,
            PieceType::ITromino => 3,
            PieceType::LTromino => 3,
            PieceType::ITetromino => 4,
            PieceType::LTetromino => 4,
            PieceType::TTetromino => 4,
            PieceType::OTetromino => 4,
            PieceType::ZTetromino => 4,
            PieceType::FPentomino => 5,
            PieceType::IPentomino => 5,
            PieceType::LPentomino => 5,
            PieceType::NPentomino => 5,
            PieceType::PPentomino => 5,
            PieceType::TPentomino => 5,
            PieceType::UPentomino => 5,
            PieceType::VPentomino => 5,
            PieceType::WPentomino => 5,
            PieceType::XPentomino => 5,
            PieceType::YPentomino => 5,
            PieceType::ZPentomino => 5,
        }
    }
}

pub const PIECE_TYPES: [PieceType; 21] = [
    PieceType::Monomino,
    PieceType::Domino,
    PieceType::ITromino,
    PieceType::LTromino,
    PieceType::ITetromino,
    PieceType::LTetromino,
    PieceType::TTetromino,
    PieceType::OTetromino,
    PieceType::ZTetromino,
    PieceType::FPentomino,
    PieceType::IPentomino,
    PieceType::LPentomino,
    PieceType::NPentomino,
    PieceType::PPentomino,
    PieceType::TPentomino,
    PieceType::UPentomino,
    PieceType::VPentomino,
    PieceType::WPentomino,
    PieceType::XPentomino,
    PieceType::YPentomino,
    PieceType::ZPentomino,
];