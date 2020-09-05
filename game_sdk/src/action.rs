use super::piece_type::{PieceType, PIECE_TYPES};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Action {
    Skip,
    Set(u16, PieceType, usize),
}

impl Action {
    pub fn to_string(&self) -> String {
        match self {
            Action::Skip => "Skip".to_string(),
            Action::Set(to, piece_type, shape_index) => format!(
                "Set {} to {} (Shape {})",
                piece_type.to_string(),
                to,
                shape_index
            ),
        }
    }

    pub fn serialize(&self) -> String {
        match self {
            Action::Skip => "Skip".to_string(),
            Action::Set(to, piece_type, shape_index) => {
                let mut piece_index: usize = 0;
                for i in 0..21 {
                    if PIECE_TYPES[i] == *piece_type {
                        piece_index = i;
                        break;
                    }
                }
                format!("{} {} {}", to, piece_index, shape_index)
            }
        }
    }

    pub fn deserialize(string: String) -> Action {
        if string == "Skip".to_string() {
            return Action::Skip;
        }
        let mut entries: Vec<&str> = string.split(" ").collect();
        //println!("{:?}", entries);
        let to = entries.remove(0).parse::<u16>().unwrap();
        let piece_index = entries.remove(0).parse::<usize>().unwrap();
        let piece_type = PIECE_TYPES[piece_index];
        let shape_index = entries.remove(0).parse::<usize>().unwrap();
        Action::Set(to, piece_type, shape_index)
    }
}
