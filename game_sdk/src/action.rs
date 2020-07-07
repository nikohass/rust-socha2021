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
            Action::Set(action, piece) =>
                format!("Set {} to {}", piece.to_string(), action & 511),
        }
    }
}
