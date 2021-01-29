use super::{Bitboard, Color, PieceType, PIECE_ORIENTATIONS};
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
            Action::Set(to, shape_index) => format!("{} {}", to, shape_index),
        }
    }

    pub fn deserialize(string: String) -> Action {
        if string == *"Skip" {
            return Action::Skip;
        }
        let mut entries: Vec<&str> = string.split(' ').collect();
        let to = entries.remove(0).parse::<u16>().unwrap();
        let shape_index = entries.remove(0).parse::<usize>().unwrap();
        Action::Set(to, shape_index)
    }

    pub fn visualize(&self) -> String {
        let board = if let Action::Set(to, shape_index) = *self {
            Bitboard::with_piece(to, shape_index)
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
        // determine shape_index
        for shape_index in 0..91 {
            if Bitboard::with_piece(
                match shape_index {
                    10 => to + 1, // X-Pentomino
                    _ => to,
                },
                shape_index,
            ) == board
            {
                return Action::Set(
                    match shape_index {
                        10 => to + 1, // X-Pentomino
                        _ => to,
                    },
                    shape_index,
                );
            }
        }
        println!(
            "Can't determine action from bitboard.\n{}",
            board.to_string()
        );
        Action::Skip
    }

    pub fn to_xml(&self, color: Color) -> String {
        match self {
            Action::Skip => "<data class=\"sc.plugin2021.SkipMove\"/>".to_string(),
            Action::Set(to, shape_index) => {
                let piece_type = PieceType::from_shape_index(*shape_index);
                let (r, flipped) = PIECE_ORIENTATIONS[*shape_index];
                let rotation = match r {
                    0 => "NONE".to_string(),
                    1 => "RIGHT".to_string(),
                    2 => "MIRROR".to_string(),
                    3 => "LEFT".to_string(),
                    _ => panic!("Invalid rotation"),
                };

                let mut to = *to;
                if piece_type == PieceType::XPentomino {
                    to -= 1;
                }
                let x = to % 21;
                let y = (to - x) / 21;

                let mut xml =
                    "  <data class=\"sc.plugin2021.SetMove\">\n    <piece color=\"".to_string();
                xml.push_str(match color {
                    Color::RED => "RED\" ",
                    Color::BLUE => "BLUE\" ",
                    Color::YELLOW => "YELLOW\" ",
                    Color::GREEN => "GREEN\" ",
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
            Action::Set(to, shape_index) => {
                let piece_type = PieceType::from_shape_index(*shape_index);
                format!("{} {} to {}", piece_type.to_short_name(), shape_index, to)
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
                Action::Set(to, shape_index) => format!(
                    "Set {} to {} (X={} Y={} Shape={})",
                    PieceType::from_shape_index(*shape_index).to_string(),
                    to,
                    to % 21,
                    (to - (to % 21)) / 21,
                    shape_index
                ),
            }
        )
    }
}
