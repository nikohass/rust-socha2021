use super::bitboard::{
    Bitboard,
    VALID_FIELDS,
    RED_START_FIELD,
    BLUE_START_FIELD,
    Direction,
    DIRECTIONS
};
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
            Action::Set(position, piece_type) => {
                let piece = piece_type.get_shape(position);

                debug_assert!(
                    !((self.board[0] | self.board[1]) & piece).not_zero(),
                    "Piece can´t be placed on other pieces."
                );
                debug_assert!(
                    self.pieces_left[piece_type as usize][self.current_player as usize] == true,
                    "Cannot place piece that has already been placed."
                );
                self.pieces_left[piece_type as usize][self.current_player as usize] = false;
                self.board[self.current_player as usize] ^= piece;
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
            Action::Set(position, piece_type) => {
                let piece = piece_type.get_shape(position);
                debug_assert!(
                    self.pieces_left[piece_type as usize][self.current_player as usize] == false,
                    "Cannot remove piece that has not been placed."
                );
                self.pieces_left[piece_type as usize][self.current_player as usize] = true;
                self.board[self.current_player as usize] ^= piece;
            }
        };
    }

    pub fn get_possible_actions(&self, actionlist: &mut ActionList) {
        let own_fields = self.board[self.current_player as usize];
        let other_fields = self.board[self.current_player.swap() as usize];
        let legal_fields = !(own_fields | other_fields | own_fields.neighbours()) & VALID_FIELDS;
        let mut must_fields = if self.ply > 1 {
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

        for d in DIRECTIONS.iter() {
            let mut two_in_a_row = match *d {
                Direction::LEFT => legal_fields & must_fields << 1,
                Direction::RIGHT => legal_fields & must_fields >> 1,
                Direction::UP => legal_fields & must_fields << 21,
                Direction::DOWN => legal_fields & must_fields >> 21,
            };
            let mut three_in_a_row = match *d {
                Direction::LEFT => legal_fields & two_in_a_row << 1,
                Direction::RIGHT => legal_fields & two_in_a_row >> 1,
                Direction::UP => legal_fields & two_in_a_row << 21,
                Direction::DOWN => legal_fields & two_in_a_row >> 21,
            };
            let mut four_in_a_row = match *d {
                Direction::LEFT => legal_fields & three_in_a_row << 1,
                Direction::RIGHT => legal_fields & three_in_a_row >> 1,
                Direction::UP => legal_fields & three_in_a_row << 21,
                Direction::DOWN => legal_fields & three_in_a_row >> 21,
            };
            let mut five_in_a_row = match *d {
                Direction::LEFT => legal_fields & four_in_a_row << 1,
                Direction::RIGHT => legal_fields & four_in_a_row >> 1,
                Direction::UP => legal_fields & four_in_a_row << 21,
                Direction::DOWN => legal_fields & four_in_a_row >> 21,
            };

            if self.pieces_left[PieceType::Domino as usize][self.current_player as usize] {
                while two_in_a_row.not_zero() {
                    let to = two_in_a_row.trailing_zeros();
                    let to_bit = Bitboard::bit(to as u16);
                    two_in_a_row ^= to_bit;
                    actionlist.push(Action::Set(to | (*d as u16) << 9, PieceType::Domino));
                }
            }
            if self.pieces_left[PieceType::ITromino as usize][self.current_player as usize] {
                while three_in_a_row.not_zero() {
                    let to = three_in_a_row.trailing_zeros();
                    let to_bit = Bitboard::bit(to as u16);
                    three_in_a_row ^= to_bit;
                    actionlist.push(Action::Set(to | (*d as u16) << 9, PieceType::ITromino));
                }
            }
            if self.pieces_left[PieceType::ITetromino as usize][self.current_player as usize] {
                while four_in_a_row.not_zero() {
                    let to = four_in_a_row.trailing_zeros();
                    let to_bit = Bitboard::bit(to);
                    four_in_a_row ^= to_bit;
                    actionlist.push(Action::Set(to | (*d as u16) << 9, PieceType::ITetromino));
                }
            }
            if self.pieces_left[PieceType::IPentomino as usize][self.current_player as usize] {
                while five_in_a_row.not_zero() {
                    let to = five_in_a_row.trailing_zeros();
                    let to_bit = Bitboard::bit(to as u16);
                    five_in_a_row ^= to_bit;
                    actionlist.push(Action::Set(to | (*d as u16) << 9, PieceType::IPentomino));
                }
            }
        }

        if self.pieces_left[PieceType::Monomino as usize][self.current_player as usize] {
            while must_fields.not_zero() {
                let to = must_fields.trailing_zeros();
                let to_bit = Bitboard::bit(to as u16);
                must_fields ^= to_bit;
                actionlist.push(Action::Set(to, PieceType::Monomino));
            }
        }

        if actionlist.size == 0 {
            actionlist.push(Action::Pass);
        }
    }

    pub fn game_result(&self) -> i16 {
        let red = self.board[0].count_ones();
        let blue = self.board[1].count_ones();

        if red > blue {
            return 1;
        }
        if blue > red {
            return -1
        }
        0
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
