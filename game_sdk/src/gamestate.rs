use super::bitboard::{Bitboard, VALID_FIELDS};
use super::color::Color;
use super::action::Action;
use super::actionlist::ActionList;
use super::piece_type::PieceType;
use std::fmt::{Display, Formatter, Result};

pub struct GameState {
    pub ply: u8,
    pub board: [Bitboard; 2],
    pub current_player: Color,
    pub pieces_left: [[bool; 2]; 21],
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            ply: 0,
            board: [Bitboard::new(), Bitboard::new()],
            current_player: Color::RED,
            pieces_left: [[true; 2]; 21]
        }
    }

    pub fn get_possible_actions(&self, actionlist: &mut ActionList) {
        let own_fields = self.board[self.current_player as usize];
        let other_fields = self.board[self.current_player.swap() as usize];
        let illegal_fields = own_fields | other_fields | own_fields.neighbours() | !VALID_FIELDS;
        let must_fields = own_fields.diagonal_neighbours() & !illegal_fields;

        debug_assert!(own_fields & VALID_FIELDS == own_fields, "Own fields are not valid fields!");
        debug_assert!(
            other_fields & VALID_FIELDS == other_fields, "Other fields are not valid! fields"
        );

        // Monomino move generation
        if self.pieces_left[0][self.current_player as usize] {
            let mut to_bit = Bitboard::from(0, 0, 0, 1);
            let mut fields = must_fields;
            while fields.not_zero() {
                if fields & to_bit == to_bit {
                    fields ^= to_bit;
                    actionlist.push(Action::Set(to_bit, PieceType::Monomino));
                }
                to_bit <<= 1;
            }
        }
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut string = String::new();

        string.push_str("╔");
        for _ in 0..40 {
            string.push_str("═");
        }
        string.push_str("╗\n");

        let info = &format!(
            "║Player: {}, Turn: {}",
            self.current_player.to_string(), self.ply
        );
        string.push_str(info);

        for _ in info.len()..43 {
            string.push_str(" ");
        }
        string.push_str("║\n");

        string.push_str("╠");
        for _ in 0..40 {
            string.push_str("═");
        }
        string.push_str("╣");

        for i in 0..20 {
            string.push_str("\n║");
            for j in 0..20 {
                let y = 19-i;
                let x = j;
                let field = x + y * 21;
                let bit = Bitboard::bit(field);
                if self.board[0] & bit == bit {
                    string.push_str("R ");
                } else if self.board[1] & bit == bit {
                    string.push_str("B ");
                } else {
                    string.push_str(". ");
                }
            }
            string.push_str("║");
        }
        string.push_str("\n╚");
        for _ in 0..40 {
            string.push_str("═");
        }
        string.push_str("╝");

        write!(f, "{}", string)
    }
}
