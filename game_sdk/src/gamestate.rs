use super::bitboard::{
    Bitboard,
    VALID_FIELDS,
    RED_START_FIELD,
    BLUE_START_FIELD,
    DIRECTIONS,
    Direction
};
use super::color::Color;
use super::action::Action;
use super::actionlist::ActionList;
use super::piece_type::{PieceType, PIECE_TYPES};
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

    pub fn check_integrity(&self) -> bool {
        if self.ply % 2 == 0 && self.current_player == Color::BLUE {
            return false
        }
        if self.ply % 2 == 1 && self.current_player == Color::RED {
            return false
        }

        for player in 0..2 {
            let mut should_have: u32 = 0;
            for piece_type in PIECE_TYPES.iter() {
                if !self.pieces_left[*piece_type as usize][player as usize] {
                    should_have += piece_type.piece_size() as u32;
                }
            }
            if should_have != self.board[player].count_ones() {
                return false;
            }
        }
        true
    }

    pub fn do_action(&mut self, action: Action) {
        match action {
            Action::Pass => {},
            Action::Set(action, piece_type) => {
                let piece = Bitboard::with_piece(action);

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
        debug_assert!(self.check_integrity());
    }

    pub fn undo_action(&mut self, action: Action) {
        self.current_player = self.current_player.swap();
        self.ply -= 1;
        match action {
            Action::Pass => {},
            Action::Set(action, piece_type) => {
                let piece = Bitboard::with_piece(action);
                debug_assert!(
                    self.pieces_left[piece_type as usize][self.current_player as usize] == false,
                    "Cannot remove piece that has not been placed."
                );
                self.pieces_left[piece_type as usize][self.current_player as usize] = true;
                self.board[self.current_player as usize] ^= piece;
            }
        };
        debug_assert!(self.check_integrity());
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
            let mut two_in_a_row = legal_fields & must_fields.neighbours_in_direction(*d);
            let mut three_in_a_row = legal_fields & two_in_a_row.neighbours_in_direction(*d);
            let mut four_in_a_row = legal_fields & three_in_a_row.neighbours_in_direction(*d);
            let mut five_in_a_row = legal_fields & four_in_a_row.neighbours_in_direction(*d);

            if self.pieces_left[PieceType::XPentomino as usize][self.current_player as usize] {
                let mut candidates = must_fields;
                while candidates.not_zero() {
                    let to = candidates.trailing_zeros();
                    candidates ^= Bitboard::bit(to);

                    let action = match *d {
                        Direction::UP => to | 10 << 9,
                        Direction::DOWN => if to > 41 {to - 42 | 10 << 9} else {0},
                        Direction::RIGHT => if to > 21 {to - 22 | 10 << 9} else {0},
                        Direction::LEFT => if to > 21 {to - 20 | 10 << 9} else {0},
                    };
                    if action != 0 {
                        let piece = Bitboard::with_piece(action);
                        if piece & legal_fields == piece {
                            actionlist.push(Action::Set(action, PieceType::XPentomino));
                        }
                    }
                }
            }

            if self.pieces_left[PieceType::OTetromino as usize][self.current_player as usize] {
                let mut candidates = must_fields;

                while candidates.not_zero() {
                    let to = candidates.trailing_zeros();
                    let to_bit = Bitboard::bit(to as u16);
                    candidates ^= to_bit;

                    let action = match *d {
                        Direction::RIGHT => to | 9 << 9,
                        Direction::UP => if to != 0 {to - 1 | 9 << 9} else {0},
                        Direction::LEFT => if to > 21 {to - 22 | 9 << 9} else {0},
                        Direction::DOWN => if to > 20 {to - 21 | 9 << 9} else {0}
                    };
                    if action != 0 {
                        let piece = Bitboard::with_piece(action);
                        if piece & legal_fields == piece {
                            actionlist.push(Action::Set(action, PieceType::OTetromino));
                        }
                    }
                }
            }
            if self.pieces_left[PieceType::LTromino as usize][self.current_player as usize] {
                let mut destinations = two_in_a_row &
                    legal_fields.neighbours_in_direction(d.clockwise());

                while destinations.not_zero() {
                    let to = destinations.trailing_zeros();
                    destinations ^= Bitboard::bit(to);

                    let action = match *d {
                        Direction::UP => if to > 20 {to - 21 | 11 << 9} else {0},
                        Direction::DOWN => to | 12 << 9,
                        Direction::LEFT => if to != 0 {to - 1 | 13 << 9} else {0},
                        Direction::RIGHT => to | 14 << 9,
                    };
                    if action != 0 {
                        let piece = Bitboard::with_piece(action);
                        if piece & legal_fields == piece {
                            actionlist.push(Action::Set(action, PieceType::LTromino));
                        }
                    }
                }
                destinations = (two_in_a_row.neighbours_in_direction(d.mirror())
                    & legal_fields.neighbours_in_direction(d.anticlockwise()))
                    .neighbours_in_direction(*d);
                while destinations.not_zero() {
                    let to = destinations.trailing_zeros();
                    destinations ^= Bitboard::bit(to);

                    let action = match *d {
                        Direction::DOWN => to | 11 << 9,
                        Direction::LEFT => if to != 0 {to - 1 | 12 << 9} else {0},
                        Direction::UP => if to > 21 {to - 22 | 13 << 9} else {0},
                        Direction::RIGHT => if to > 20 {to - 21 | 14 << 9} else {0},
                    };
                    if action != 0 {
                        let piece = Bitboard::with_piece(action);
                        if piece & legal_fields == piece {
                            actionlist.push(Action::Set(action, PieceType::LTromino));
                        }
                    }
                }
            }

            if self.pieces_left[PieceType::Domino as usize][self.current_player as usize] {
                while two_in_a_row.not_zero() {
                    let to = two_in_a_row.trailing_zeros();
                    let to_bit = Bitboard::bit(to as u16);
                    two_in_a_row ^= to_bit;
                    actionlist.push(
                        match *d {
                            Direction::RIGHT => Action::Set(to | 1 << 9, PieceType::Domino),
                            Direction::LEFT => Action::Set(to - 1 | 1 << 9, PieceType::Domino),
                            Direction::UP => Action::Set(to - 21 | 2 << 9, PieceType::Domino),
                            Direction::DOWN => Action::Set(to | 2 << 9, PieceType::Domino),
                        }
                    );
                }
            }
            if self.pieces_left[PieceType::ITromino as usize][self.current_player as usize] {
                while three_in_a_row.not_zero() {
                    let to = three_in_a_row.trailing_zeros();
                    let to_bit = Bitboard::bit(to as u16);
                    three_in_a_row ^= to_bit;
                    actionlist.push(
                        match *d {
                            Direction::RIGHT => Action::Set(to | 3 << 9, PieceType::ITromino),
                            Direction::LEFT => Action::Set(to - 2 | 3 << 9, PieceType::ITromino),
                            Direction::UP => Action::Set(to - 42 | 4 << 9, PieceType::ITromino),
                            Direction::DOWN => Action::Set(to | 4 << 9, PieceType::ITromino),
                        }
                    );
                }
            }
            if self.pieces_left[PieceType::ITetromino as usize][self.current_player as usize] {
                while four_in_a_row.not_zero() {
                    let to = four_in_a_row.trailing_zeros();
                    let to_bit = Bitboard::bit(to);
                    four_in_a_row ^= to_bit;
                    actionlist.push(
                        match *d {
                            Direction::RIGHT => Action::Set(to | 5 << 9, PieceType::ITetromino),
                            Direction::LEFT => Action::Set(to - 3 | 5 << 9, PieceType::ITetromino),
                            Direction::UP => Action::Set(to - 63 | 6 << 9, PieceType::ITetromino),
                            Direction::DOWN => Action::Set(to | 6 << 9, PieceType::ITetromino),
                        }
                    );
                }
            }
            if self.pieces_left[PieceType::IPentomino as usize][self.current_player as usize] {
                while five_in_a_row.not_zero() {
                    let to = five_in_a_row.trailing_zeros();
                    let to_bit = Bitboard::bit(to as u16);
                    five_in_a_row ^= to_bit;
                    actionlist.push(
                        match *d {
                            Direction::RIGHT => Action::Set(to | 7 << 9, PieceType::IPentomino),
                            Direction::LEFT => Action::Set(to - 4 | 7 << 9, PieceType::IPentomino),
                            Direction::UP => Action::Set(to - 84 | 8 << 9, PieceType::IPentomino),
                            Direction::DOWN => Action::Set(to | 8 << 9, PieceType::IPentomino),
                        }
                    );
                }
            }
        }

        if self.pieces_left[PieceType::LTetromino as usize][self.current_player as usize] {
            let mut candidates = must_fields;
            let offsets: [[u16; 3]; 8] = [
                [0, 1, 42], [0, 1, 43], [1, 43, 42], [0, 42, 43],
                [0, 21, 23], [0, 2, 21], [0, 2, 23], [2, 21, 23]
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates ^= Bitboard::bit(to);

                let mut shape_index = 15;
                for i in 0..8 {
                    for p in 0..3 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p] | shape_index << 9;
                            let piece = Bitboard::with_piece(action);
                            if piece & legal_fields == piece {
                                actionlist.push(Action::Set(action, PieceType::LTetromino));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::LPentomino as usize][self.current_player as usize] {
            let mut candidates = must_fields;
            let offsets: [[u16; 3]; 8] = [
                [0, 3, 24], [0, 3, 21], [0, 21, 24], [3, 21, 24],
                [0, 1, 63], [0, 63, 64], [0, 1, 64], [1, 63, 64]
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates ^= Bitboard::bit(to);

                let mut shape_index = 23;
                for i in 0..8 {
                    for p in 0..3 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p] | shape_index << 9;
                            let piece = Bitboard::with_piece(action);
                            if piece & legal_fields == piece {
                                actionlist.push(Action::Set(action, PieceType::LPentomino));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::TPentomino as usize][self.current_player as usize] {
            let mut candidates = must_fields;
            let offsets: [[u16; 3]; 4] = [
                [0, 2, 43], [1, 42, 44], [0, 23, 42], [2, 21, 44],
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates ^= Bitboard::bit(to);

                let mut shape_index = 31;
                for i in 0..4 {
                    for p in 0..3 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p] | shape_index << 9;
                            let piece = Bitboard::with_piece(action);
                            if piece & legal_fields == piece {
                                actionlist.push(Action::Set(action, PieceType::TPentomino));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::TTetromino as usize][self.current_player as usize] {
            let mut candidates = must_fields;
            let offsets: [[u16; 3]; 4] = [
                [0, 2, 22], [1, 21, 23], [0, 22, 42], [1, 21, 43],
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates ^= Bitboard::bit(to);

                let mut shape_index = 35;
                for i in 0..4 {
                    for p in 0..3 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p] | shape_index << 9;
                            let piece = Bitboard::with_piece(action);
                            if piece & legal_fields == piece {
                                actionlist.push(Action::Set(action, PieceType::TTetromino));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::ZTetromino as usize][self.current_player as usize] {
            let mut candidates = must_fields;
            let offsets: [[u16; 4]; 4] = [
                [1, 2, 21, 22], [0, 1, 22, 23], [1, 21, 22, 42], [0, 21, 22, 43]
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates ^= Bitboard::bit(to);

                let mut shape_index = 39;
                for i in 0..4 {
                    for p in 0..4 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p] | shape_index << 9;
                            let piece = Bitboard::with_piece(action);
                            if piece & legal_fields == piece {
                                actionlist.push(Action::Set(action, PieceType::ZTetromino));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::ZPentomino as usize][self.current_player as usize] {
            let mut candidates = must_fields;
            let offsets: [[u16; 4]; 4] = [
                [0, 21, 23, 44], [2, 21, 23, 42], [1, 2, 42, 43], [0, 1, 43, 44]
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates ^= Bitboard::bit(to);

                let mut shape_index = 43;
                for i in 0..4 {
                    for p in 0..4 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p] | shape_index << 9;
                            let piece = Bitboard::with_piece(action);
                            if piece & legal_fields == piece {
                                actionlist.push(Action::Set(action, PieceType::ZPentomino));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::UPentomino as usize][self.current_player as usize] {
            let mut candidates = must_fields;
            let offsets: [[u16; 4]; 4] = [
                [0, 2, 21, 23], [0, 2, 21, 23], [0, 1, 42, 43], [0, 1, 42, 43]
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates ^= Bitboard::bit(to);

                let mut shape_index = 47;
                for i in 0..4 {
                    for p in 0..4 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p] | shape_index << 9;
                            let piece = Bitboard::with_piece(action);
                            if piece & legal_fields == piece {
                                actionlist.push(Action::Set(action, PieceType::UPentomino));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::FPentomino as usize][self.current_player as usize] {
            let mut candidates = must_fields;
            let offsets: [[u16; 4]; 8] = [
                [1, 23, 42, 43], [1, 21, 43, 44], [1, 2, 21, 43], [0, 1, 23, 43],
                [2, 21, 23, 43], [0, 21, 23, 43], [1, 21, 23, 44], [1, 21, 23, 42]
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates ^= Bitboard::bit(to);

                let mut shape_index = 51;
                for i in 0..8 {
                    for p in 0..4 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p] | shape_index << 9;
                            let piece = Bitboard::with_piece(action);
                            if piece & legal_fields == piece {
                                actionlist.push(Action::Set(action, PieceType::FPentomino));
                            }
                        }
                    }
                    shape_index += 1;
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
                    string.push_str("🟥");
                } else if self.board[1] & bit == bit {
                    string.push_str("🟦");
                } else {
                    string.push_str("▪️");
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
