use rand::{rngs::SmallRng, RngCore, SeedableRng};
use std::fmt::{Display, Formatter, Result};

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
    ZPentomino = 20,
}

impl PieceType {
    pub fn random_pentomino() -> Self {
        let mut rng = SmallRng::from_entropy();
        START_PIECE_TYPES[rng.next_u64() as usize % 11]
    }

    #[inline(always)]
    pub fn from_shape(shape: usize) -> Self {
        FROM_SHAPE[shape]
    }

    pub fn to_xml_name(&self) -> String {
        NAMES[*self as usize].1.to_string()
    }

    pub fn to_short_name(&self) -> String {
        NAMES[*self as usize].2.to_string()
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
            _ => 5,
        }
    }
}

impl Display for PieceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", NAMES[*self as usize].0.to_string())
    }
}

// full name, xml name, short name
pub const NAMES: [(&str, &str, &str); 21] = [
    ("Monomino", "MONO", "M"),
    ("Domino", "DOMINO", "D"),
    ("I-Tromino", "TRIO_I", "I3"),
    ("L-Tromino", "TRIO_L", "L3"),
    ("I-Tetromino", "TETRO_I", "I4"),
    ("L-Tetromino", "TETRO_L", "L4"),
    ("T-Tetromino", "TETRO_T", "T4"),
    ("O-Tetromino", "TETRO_O", "O"),
    ("Z-Tetromino", "TETRO_Z", "Z4"),
    ("F-Pentomino", "PENTO_R", "F"),
    ("I-Pentomino", "PENTO_I", "I5"),
    ("L-Pentomino", "PENTO_L", "L5"),
    ("N-Pentomino", "PENTO_S", "N"),
    ("P-Pentomino", "PENTO_P", "P"),
    ("T-Pentomino", "PENTO_T", "T5"),
    ("U-Pentomino", "PENTO_U", "U"),
    ("V-Pentomino", "PENTO_V", "V"),
    ("W-Pentomino", "PENTO_W", "W"),
    ("X-Pentomino", "PENTO_X", "X"),
    ("Y-Pentomino", "PENTO_Y", "Y"),
    ("Z-Pentomino", "PENTO_Z", "Z5"),
];

pub const FROM_SHAPE: [PieceType; 91] = [
    PieceType::Monomino,
    PieceType::Domino,
    PieceType::Domino,
    PieceType::ITromino,
    PieceType::ITromino,
    PieceType::ITetromino,
    PieceType::ITetromino,
    PieceType::IPentomino,
    PieceType::IPentomino,
    PieceType::OTetromino,
    PieceType::XPentomino,
    PieceType::LTromino,
    PieceType::LTromino,
    PieceType::LTromino,
    PieceType::LTromino,
    PieceType::LTetromino,
    PieceType::LTetromino,
    PieceType::LTetromino,
    PieceType::LTetromino,
    PieceType::LTetromino,
    PieceType::LTetromino,
    PieceType::LTetromino,
    PieceType::LTetromino,
    PieceType::LPentomino,
    PieceType::LPentomino,
    PieceType::LPentomino,
    PieceType::LPentomino,
    PieceType::LPentomino,
    PieceType::LPentomino,
    PieceType::LPentomino,
    PieceType::LPentomino,
    PieceType::TPentomino,
    PieceType::TPentomino,
    PieceType::TPentomino,
    PieceType::TPentomino,
    PieceType::TTetromino,
    PieceType::TTetromino,
    PieceType::TTetromino,
    PieceType::TTetromino,
    PieceType::ZTetromino,
    PieceType::ZTetromino,
    PieceType::ZTetromino,
    PieceType::ZTetromino,
    PieceType::ZPentomino,
    PieceType::ZPentomino,
    PieceType::ZPentomino,
    PieceType::ZPentomino,
    PieceType::UPentomino,
    PieceType::UPentomino,
    PieceType::UPentomino,
    PieceType::UPentomino,
    PieceType::FPentomino,
    PieceType::FPentomino,
    PieceType::FPentomino,
    PieceType::FPentomino,
    PieceType::FPentomino,
    PieceType::FPentomino,
    PieceType::FPentomino,
    PieceType::FPentomino,
    PieceType::WPentomino,
    PieceType::WPentomino,
    PieceType::WPentomino,
    PieceType::WPentomino,
    PieceType::NPentomino,
    PieceType::NPentomino,
    PieceType::NPentomino,
    PieceType::NPentomino,
    PieceType::NPentomino,
    PieceType::NPentomino,
    PieceType::NPentomino,
    PieceType::NPentomino,
    PieceType::VPentomino,
    PieceType::VPentomino,
    PieceType::VPentomino,
    PieceType::VPentomino,
    PieceType::PPentomino,
    PieceType::PPentomino,
    PieceType::PPentomino,
    PieceType::PPentomino,
    PieceType::PPentomino,
    PieceType::PPentomino,
    PieceType::PPentomino,
    PieceType::PPentomino,
    PieceType::YPentomino,
    PieceType::YPentomino,
    PieceType::YPentomino,
    PieceType::YPentomino,
    PieceType::YPentomino,
    PieceType::YPentomino,
    PieceType::YPentomino,
    PieceType::YPentomino,
];

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

pub const START_PIECE_TYPES: [PieceType; 11] = [
    PieceType::FPentomino,
    PieceType::IPentomino,
    PieceType::LPentomino,
    PieceType::NPentomino,
    PieceType::PPentomino,
    PieceType::TPentomino,
    PieceType::UPentomino,
    PieceType::VPentomino,
    PieceType::WPentomino,
    PieceType::YPentomino,
    PieceType::ZPentomino,
];
