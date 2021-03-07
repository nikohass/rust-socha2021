use super::{Bitboard, PieceType, PIECE_ORIENTATIONS};
use std::fmt::{Display, Formatter, Result};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Action {
    Skip,
    Set(u16, usize),
}

impl Action {
    pub fn serialize(&self) -> String {
        match self {
            Action::Skip => "Skip".to_string(),
            Action::Set(to, shape) => format!("{} {}", to, shape),
        }
    }

    pub fn deserialize(string: String) -> Action {
        if string == *"Skip" {
            return Action::Skip;
        }
        let mut entries: Vec<&str> = string.split(' ').collect();
        let to = entries.remove(0).parse::<u16>().unwrap();
        let shape = entries.remove(0).parse::<usize>().unwrap();
        Action::Set(to, shape)
    }

    pub fn visualize(&self) -> String {
        let board = if let Action::Set(to, shape) = *self {
            Bitboard::with_piece(to, shape)
        } else {
            Bitboard::empty()
        };
        format!("{}\n{}", self.to_string(), board)
    }

    pub fn from_bitboard(board: Bitboard) -> Action {
        if board.is_zero() {
            return Action::Skip;
        }
        // determine top left corner of the piece
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
        let to = left + top * 21;
        // determine shape
        for shape in 0..91 {
            if Bitboard::with_piece(to, shape) == board {
                return Action::Set(to, shape);
            }
        }
        if cfg!(debug_assertions) {
            println!(
                "Can't determine action from bitboard.\n{}",
                board.to_string()
            );
        }
        Action::Skip
    }

    pub fn to_xml(&self, color: u8) -> String {
        match self {
            Action::Skip => "<data class=\"sc.plugin2021.SkipMove\"/>".to_string(),
            Action::Set(to, shape) => {
                let piece_type = PieceType::from_shape(*shape);
                let (r, flipped) = PIECE_ORIENTATIONS[*shape];
                let rotation = match r {
                    0 => "NONE".to_string(),
                    1 => "RIGHT".to_string(),
                    2 => "MIRROR".to_string(),
                    3 => "LEFT".to_string(),
                    _ => panic!("Invalid rotation"),
                };
                let x = *to % 21;
                let y = (*to - x) / 21;
                let mut xml =
                    "  <data class=\"sc.plugin2021.SetMove\">\n    <piece color=\"".to_string();
                xml.push_str(match color as u8 {
                    0 => "BLUE\" ",
                    1 => "YELLOW\" ",
                    2 => "RED\" ",
                    3 => "GREEN\" ",
                    _ => panic!("Invalid color"),
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
    }

    pub fn to_short_name(&self) -> String {
        match self {
            Action::Skip => "Skip".to_string(),
            Action::Set(to, shape) => {
                let piece_type = PieceType::from_shape(*shape);
                format!("{} {} to {}", piece_type.to_short_name(), shape, to)
            }
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            match self {
                Action::Skip => "Skip".to_string(),
                Action::Set(to, shape) => format!(
                    "Set {} to {} (X={} Y={} Shape={})",
                    PieceType::from_shape(*shape).to_string(),
                    to,
                    to % 21,
                    (to - (to % 21)) / 21,
                    shape
                ),
            }
        )
    }
}
