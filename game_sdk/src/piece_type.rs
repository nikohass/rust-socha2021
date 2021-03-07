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
    pub fn random_pentomino() -> PieceType {
        let mut rng = SmallRng::from_entropy();
        START_PIECE_TYPES[rng.next_u64() as usize % 11]
    }

    pub fn from_shape(shape: usize) -> PieceType {
        match shape {
            0 => PieceType::Monomino,
            1 | 2 => PieceType::Domino,
            3 | 4 => PieceType::ITromino,
            5 | 6 => PieceType::ITetromino,
            7 | 8 => PieceType::IPentomino,
            9 => PieceType::OTetromino,
            10 => PieceType::XPentomino,
            11..=14 => PieceType::LTromino,
            15..=22 => PieceType::LTetromino,
            23..=30 => PieceType::LPentomino,
            31..=34 => PieceType::TPentomino,
            35..=38 => PieceType::TTetromino,
            39..=42 => PieceType::ZTetromino,
            43..=46 => PieceType::ZPentomino,
            47..=50 => PieceType::UPentomino,
            51..=58 => PieceType::FPentomino,
            59..=62 => PieceType::WPentomino,
            63..=70 => PieceType::NPentomino,
            71..=74 => PieceType::VPentomino,
            75..=82 => PieceType::PPentomino,
            _ => PieceType::YPentomino,
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
            _ => 5,
        }
    }

    pub fn to_xml_name(&self) -> String {
        match self {
            PieceType::Monomino => "MONO",
            PieceType::Domino => "DOMINO",
            PieceType::ITromino => "TRIO_I",
            PieceType::LTromino => "TRIO_L",
            PieceType::ITetromino => "TETRO_I",
            PieceType::LTetromino => "TETRO_L",
            PieceType::TTetromino => "TETRO_T",
            PieceType::OTetromino => "TETRO_O",
            PieceType::ZTetromino => "TETRO_Z",
            PieceType::FPentomino => "PENTO_R",
            PieceType::IPentomino => "PENTO_I",
            PieceType::LPentomino => "PENTO_L",
            PieceType::NPentomino => "PENTO_S",
            PieceType::PPentomino => "PENTO_P",
            PieceType::TPentomino => "PENTO_T",
            PieceType::UPentomino => "PENTO_U",
            PieceType::VPentomino => "PENTO_V",
            PieceType::WPentomino => "PENTO_W",
            PieceType::XPentomino => "PENTO_X",
            PieceType::YPentomino => "PENTO_Y",
            PieceType::ZPentomino => "PENTO_Z",
        }
        .to_string()
    }

    pub fn to_short_name(&self) -> String {
        match self {
            PieceType::Monomino => "M",
            PieceType::Domino => "D",
            PieceType::ITromino => "I3",
            PieceType::LTromino => "L3",
            PieceType::ITetromino => "I4",
            PieceType::LTetromino => "L4",
            PieceType::TTetromino => "T4",
            PieceType::OTetromino => "O4",
            PieceType::ZTetromino => "Z4",
            PieceType::FPentomino => "F5",
            PieceType::IPentomino => "I5",
            PieceType::LPentomino => "L5",
            PieceType::NPentomino => "N5",
            PieceType::PPentomino => "P5",
            PieceType::TPentomino => "T5",
            PieceType::UPentomino => "U5",
            PieceType::VPentomino => "V5",
            PieceType::WPentomino => "W5",
            PieceType::XPentomino => "X5",
            PieceType::YPentomino => "Y5",
            PieceType::ZPentomino => "Z5",
        }
        .to_string()
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

impl Display for PieceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            match self {
                PieceType::Monomino => "Monomino",
                PieceType::Domino => "Domino",
                PieceType::ITromino => "I-Tromino",
                PieceType::LTromino => "L-Tromino",
                PieceType::ITetromino => "I-Tetromino",
                PieceType::LTetromino => "L-Tetromino",
                PieceType::TTetromino => "T-Tetromino",
                PieceType::OTetromino => "O-Tetromino",
                PieceType::ZTetromino => "Z-Tetromino",
                PieceType::FPentomino => "F-Pentomino",
                PieceType::IPentomino => "I-Pentomino",
                PieceType::LPentomino => "L-Pentomino",
                PieceType::NPentomino => "N-Pentomino",
                PieceType::PPentomino => "P-Pentomino",
                PieceType::TPentomino => "T-Pentomino",
                PieceType::UPentomino => "U-Pentomino",
                PieceType::VPentomino => "V-Pentomino",
                PieceType::WPentomino => "W-Pentomino",
                PieceType::XPentomino => "X-Pentomino",
                PieceType::YPentomino => "Y-Pentomino",
                PieceType::ZPentomino => "Z-Pentomino",
            }
            .to_string()
        )
    }
}
