use super::color::Color;
use super::constants::PIECE_ORIENTATIONS;
use super::piece_type::{PieceType, PIECE_TYPES};
use std::fmt::{Display, Formatter, Result};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Action {
    Skip,
    Set(u16, PieceType, usize),
}

impl Action {
    pub fn serialize(&self) -> String {
        match self {
            Action::Skip => "Skip".to_string(),
            Action::Set(to, piece_type, shape_index) => {
                let mut piece_index: usize = 0;
                for (i, pt) in PIECE_TYPES.iter().enumerate() {
                    if pt == piece_type {
                        piece_index = i;
                        break;
                    }
                }
                format!("{} {} {}", to, piece_index, shape_index)
            }
        }
    }

    pub fn deserialize(string: String) -> Action {
        if string == *"Skip" {
            return Action::Skip;
        }
        let mut entries: Vec<&str> = string.split(' ').collect();
        let to = entries.remove(0).parse::<u16>().unwrap();
        let piece_index = entries.remove(0).parse::<usize>().unwrap();
        let piece_type = PIECE_TYPES[piece_index];
        let shape_index = entries.remove(0).parse::<usize>().unwrap();
        Action::Set(to, piece_type, shape_index)
    }

    pub fn to_xml(&self, color: Color) -> String {
        match self {
            Action::Skip => "<data class=\"sc.plugin2021.SkipMove\"/>".to_string(),
            Action::Set(to, piece_type, shape_index) => {
                let (r, flipped) = PIECE_ORIENTATIONS[*shape_index];
                let rotation = match r {
                    0 => "NONE".to_string(),
                    1 => "RIGHT".to_string(),
                    2 => "MIRROR".to_string(),
                    3 => "LEFT".to_string(),
                    _ => panic!("Invalid rotation"),
                };

                let mut to = *to;
                if *piece_type == PieceType::XPentomino {
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
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            match self {
                Action::Skip => "Skip".to_string(),
                Action::Set(to, piece_type, shape_index) => format!(
                    "Set {} to {} (X={}, Y={} Shape={})",
                    piece_type.to_string(),
                    to,
                    to % 21,
                    (to - (to % 21)) / 21,
                    shape_index
                ),
            }
        )
    }
}
