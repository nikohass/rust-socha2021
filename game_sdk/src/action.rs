use super::{Bitboard, PieceType, PIECE_ORIENTATIONS};
use std::fmt::{Display, Formatter, Result};

const SKIP: u16 = std::u16::MAX;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Action(u16);

impl Action {
    #[inline(always)]
    pub fn set(destination: u16, shape: u16) -> Self {
        Self(destination << 7 | shape)
    }

    #[inline(always)]
    pub fn skip() -> Self {
        Self(SKIP)
    }

    #[inline(always)]
    pub fn get_shape(self) -> u16 {
        self.0 & 0b1111111
    }

    #[inline(always)]
    pub fn get_destination(self) -> u16 {
        self.0 >> 7
    }

    #[inline(always)]
    pub fn is_skip(self) -> bool {
        self.0 == SKIP
    }

    #[inline(always)]
    pub fn is_set(self) -> bool {
        self.0 != SKIP
    }

    pub fn serialize(self) -> String {
        self.0.to_string()
    }

    pub fn deserialize(string: String) -> Self {
        Self(string.parse::<u16>().unwrap())
    }

    pub fn from_bitboard(board: Bitboard) -> Self {
        if board.is_zero() {
            return Self::skip();
        }
        let mut board_copy = board;
        let mut left = 21;
        let mut top = 21;
        while board_copy.not_zero() {
            let field_index = board_copy.trailing_zeros();
            board_copy.flip_bit(field_index);
            let x = field_index % 21;
            let y = (field_index - x) / 21;
            if x < left {
                left = x;
            }
            if y < top {
                top = y;
            }
        }
        let destination = left + top * 21;
        for shape in 0..91 {
            if Bitboard::with_piece(destination, shape) == board {
                return Self::set(destination, shape as u16);
            }
        }
        if cfg!(debug_assertions) {
            println!(
                "Can't determine action from bitboard.\n{}",
                board.to_string()
            );
        }
        Self::skip()
    }

    pub fn to_xml(self, color: u8) -> String {
        if self.is_skip() {
            "<data class=\"sc.plugin2021.SkipMove\"/>".to_string()
        } else {
            let destination = self.get_destination();
            let shape = self.get_shape() as usize;
            let piece_type = PieceType::from_shape(shape);
            let (r, flipped) = PIECE_ORIENTATIONS[shape];
            let rotation = match r {
                0 => "NONE".to_string(),
                1 => "RIGHT".to_string(),
                2 => "MIRROR".to_string(),
                _ => "LEFT".to_string(),
            };
            let x = destination % 21;
            let y = (destination - x) / 21;
            let mut xml =
                "  <data class=\"sc.plugin2021.SetMove\">\n    <piece color=\"".to_string();
            xml.push_str(match color as u8 {
                0 => "BLUE\" ",
                1 => "YELLOW\" ",
                2 => "RED\" ",
                _ => "GREEN\" ",
            });
            xml.push_str(&format!(
                "kind=\"{}\" rotation=\"{}\" isFlipped=\"",
                &piece_type.to_xml_name(),
                &rotation,
            ));
            xml.push_str(&format!(
                "{}\">\n      <position x=\"{}\" y=\"{}\"/>\n    </piece>\n  </data>",
                &flipped.to_string(),
                x,
                y
            ));
            xml
        }
    }

    pub fn to_short_name(self) -> String {
        if self.is_skip() {
            "Skip".to_string()
        } else {
            let shape = self.get_shape() as usize;
            let piece_type = PieceType::from_shape(shape);
            format!("{}_{}", piece_type.to_short_name(), self.0)
        }
    }

    pub fn visualize(self) -> String {
        let board = if self.is_set() {
            let destination = self.get_destination();
            let shape = self.get_shape();
            Bitboard::with_piece(destination, shape as usize)
        } else {
            Bitboard::empty()
        };
        format!("{}\n{}", self.to_string(), board)
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            if self.is_skip() {
                "Skip".to_string()
            } else {
                let destination = self.get_destination();
                let shape = self.get_shape() as usize;
                let piece_type = PieceType::from_shape(shape).to_string();
                let x = destination % 21;
                let y = (destination - x) / 21;
                format!(
                    "Set {} to {} (X={} Y={} S={} V={})",
                    piece_type, destination, x, y, shape, self.0
                )
            }
        )
    }
}
