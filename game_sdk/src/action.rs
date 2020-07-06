use super::piece_type::PieceType;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Action {
    Pass,
    Set(u16, PieceType)
}

impl Action {
    pub fn to_string(&self) -> String {
        match self {
            Action::Pass => "Pass".to_string(),
            Action::Set(to, piece_type) =>
                format!("Set {} to {}",
                    piece_type.to_string(), to & 511
                ),
        }
    }
}
