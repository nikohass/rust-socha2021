use super::bitboard::Bitboard;
use super::piece_type::PieceType;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Action {
    Pass,
    Set(Bitboard, PieceType)
}

impl Action {
    pub fn to_string(&self) -> String {
        match self {
            Action::Pass => "Pass".to_string(),
            Action::Set(board, piece_type) =>
                format!("Set {} to {}", piece_type.to_string(), board.trailing_zeros()),
        }
    }
}
