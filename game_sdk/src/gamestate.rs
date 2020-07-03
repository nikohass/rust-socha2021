use super::bitboard::Bitboard;
use super::color::Color;

pub struct GameState {
    pub ply: u8,
    pub board: [Bitboard; 2],
    pub current_player: Color,
    pub pieces_left: [[bool; 2]; 20],
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            ply: 0,
            board: [Bitboard::new(), Bitboard::new()],
            current_player: Color::RED,
            pieces_left: [[true; 2]; 20]
        }
    }
}
