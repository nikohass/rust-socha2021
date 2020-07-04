use super::bitboard::{Bitboard, VALID_FIELDS, RED_START_FIELD, BLUE_START_FIELD};
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

    pub fn do_action(&mut self, action: Action) {
        match action {
            Action::Pass => {},
            Action::Set(board, piece_type) => {
                debug_assert!(
                    !((self.board[0] | self.board[1]) & board).not_zero(),
                    "Piece can´t be placed on other pieces."
                );
                debug_assert!(
                    self.pieces_left[piece_type as usize][self.current_player as usize] == true,
                    "Cannot place piece that has already been placed."
                );
                self.pieces_left[piece_type as usize][self.current_player as usize] = false;
                self.board[self.current_player as usize] ^= board;
            }
        };
        self.current_player = self.current_player.swap();
        self.ply += 1;
    }

    pub fn undo_action(&mut self, action: Action) {
        self.current_player = self.current_player.swap();
        self.ply -= 1;
        match action {
            Action::Pass => {},
            Action::Set(board, piece_type) => {
                debug_assert!(
                    self.pieces_left[piece_type as usize][self.current_player as usize] == false,
                    "Cannot remove piece that has not been placed."
                );
                self.pieces_left[piece_type as usize][self.current_player as usize] = true;
                self.board[self.current_player as usize] ^= board;
            }
        };
    }

    pub fn get_possible_actions(&self, actionlist: &mut ActionList) {
        let own_fields = self.board[self.current_player as usize];
        let other_fields = self.board[self.current_player.swap() as usize];
        let legal_fields = !(own_fields | other_fields | own_fields.neighbours()) & VALID_FIELDS;
        let must_fields = if self.ply > 1 {
            own_fields.diagonal_neighbours() & legal_fields
        } else if self.ply == 0 {
            RED_START_FIELD
        } else {
            BLUE_START_FIELD
        };

        debug_assert!(own_fields & VALID_FIELDS == own_fields, "Own fields are not valid fields.");
        debug_assert!(
            other_fields & VALID_FIELDS == other_fields, "Other fields are not valid fields."
        );

        let mut to_bit = Bitboard::from(0, 0, 0, 1);
        let mut fields = must_fields;
        while fields.not_zero() {
            if fields & to_bit == to_bit {
                fields ^= to_bit;
                if self.pieces_left[0][self.current_player as usize] {
                    // Monomino move generation
                    actionlist.push(Action::Set(to_bit, PieceType::Monomino));
                }
                // Left move generation
                if (legal_fields & (to_bit << 1)).not_zero() {
                    if self.pieces_left[1][self.current_player as usize] {
                        actionlist.push(Action::Set(to_bit | to_bit << 1, PieceType::Domino));
                    }
                    if self.pieces_left[2][self.current_player as usize]
                        && (legal_fields & (to_bit << 2)).not_zero()
                    {
                        actionlist.push(
                            Action::Set(to_bit | to_bit << 1 | to_bit << 2, PieceType::ITromino)
                        );
                    }
                }
                // Right move generation
                if (legal_fields & (to_bit >> 1)).not_zero() {
                    if self.pieces_left[1][self.current_player as usize] {
                        actionlist.push(Action::Set(to_bit | to_bit >> 1, PieceType::Domino));
                    }
                    if self.pieces_left[2][self.current_player as usize]
                        &&(legal_fields & (to_bit >> 2)).not_zero()
                    {
                        actionlist.push(
                            Action::Set(to_bit | to_bit >> 1 | to_bit >> 2, PieceType::ITromino)
                        );
                    }
                }
                // Lower move generation
                if (legal_fields & (to_bit << 21)).not_zero() {
                    if self.pieces_left[1][self.current_player as usize] {
                        actionlist.push(Action::Set(to_bit | to_bit << 21, PieceType::Domino));
                    }
                    if self.pieces_left[2][self.current_player as usize]
                        &&(legal_fields & (to_bit << 42)).not_zero()
                    {
                        actionlist.push(
                            Action::Set(to_bit | to_bit << 21 | to_bit << 42, PieceType::ITromino)
                        );
                    }
                }
                // Higher move generation
                if (legal_fields & (to_bit >> 21)).not_zero() {
                    if self.pieces_left[1][self.current_player as usize] {
                        actionlist.push(Action::Set(to_bit | to_bit >> 21, PieceType::Domino));
                    }
                    if self.pieces_left[2][self.current_player as usize]
                        &&(legal_fields & (to_bit >> 42)).not_zero()
                    {
                        actionlist.push(
                            Action::Set(to_bit | to_bit >> 21 | to_bit >> 42, PieceType::ITromino)
                        );
                    }
                }
            }
            to_bit <<= 1;
        }


        if actionlist.size == 0 {
            actionlist.push(Action::Pass);
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
