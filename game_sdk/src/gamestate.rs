use super::bitboard::Bitboard;
use super::direction::{Direction, DIRECTIONS};
use super::color::Color;
use super::action::Action;
use super::actionlist::ActionList;
use super::piece_type::{PieceType, PIECE_TYPES};
use super::constants::{START_FIELDS, VALID_FIELDS};
use std::fmt::{Display, Formatter, Result};

pub struct GameState {
    pub ply: u8,
    pub board: [Bitboard; 4],
    pub current_player: Color,
    pub pieces_left: [[bool; 4]; 21],
    pub monomino_placed_last: [bool; 4],
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            ply: 0,
            board: [Bitboard::new(); 4],
            current_player: Color::BLUE,
            pieces_left: [[true; 4]; 21],
            monomino_placed_last: [false; 4],
        }
    }

    pub fn check_integrity(&self) -> bool {
        for color in 0..4 {
            if self.ply % 4 == color && self.current_player as u8 != color {
                return false;
            }
        }

        for player in 0..4 {
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
                    !((self.board[0] | self.board[1] | self.board[2] | self.board[3]) & piece)
                    .not_zero(),
                    "Piece can´t be placed on other pieces."
                );
                debug_assert!(
                    self.pieces_left[piece_type as usize][self.current_player as usize],
                    "Cannot place piece that has already been placed."
                );
                self.pieces_left[piece_type as usize][self.current_player as usize] = false;
                self.board[self.current_player as usize] ^= piece;
                self.monomino_placed_last[self.current_player as usize] =
                    piece_type == PieceType::Monomino;
            }
        };
        self.current_player = self.current_player.next();
        self.ply += 1;
        debug_assert!(self.check_integrity());
    }

    pub fn undo_action(&mut self, action: Action) {
        self.current_player = self.current_player.previous();
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
                //self.monomino_placed_last[self.current_player as usize] = false;
            }
        };
        debug_assert!(self.check_integrity());
    }

    pub fn get_possible_actions(&self, actionlist: &mut ActionList) {
        let own_fields = self.board[self.current_player as usize];
        let other_fields = (self.board[0] | self.board[1] | self.board[2] | self.board[3])
            & !own_fields;

        let legal_fields = !(own_fields | other_fields | own_fields.neighbours()) & VALID_FIELDS;
        let mut must_fields = if self.ply > 3 {
            own_fields.diagonal_neighbours() & legal_fields
        } else {
            START_FIELDS & !other_fields
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
                    candidates.flip_bit(to);

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
                    candidates.flip_bit(to);

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
                let mut candidates = two_in_a_row &
                    legal_fields.neighbours_in_direction(d.clockwise());

                while candidates.not_zero() {
                    let to = candidates.trailing_zeros();
                    candidates.flip_bit(to);

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
                candidates = (two_in_a_row.neighbours_in_direction(d.mirror())
                    & legal_fields.neighbours_in_direction(d.anticlockwise()))
                    .neighbours_in_direction(*d);
                while candidates.not_zero() {
                    let to = candidates.trailing_zeros();
                    candidates.flip_bit(to);

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
                    two_in_a_row.flip_bit(to);
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
                    three_in_a_row.flip_bit(to);
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
                    four_in_a_row.flip_bit(to);
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
                    five_in_a_row.flip_bit(to);
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
                candidates.flip_bit(to);

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
                candidates.flip_bit(to);

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
                candidates.flip_bit(to);

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
                candidates.flip_bit(to);

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
                candidates.flip_bit(to);

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
                candidates.flip_bit(to);

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
                candidates.flip_bit(to);

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
                candidates.flip_bit(to);

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

        if self.pieces_left[PieceType::WPentomino as usize][self.current_player as usize] {
            let mut candidates = must_fields;
            let offsets: [[u16; 5]; 4] = [
                [0, 21, 22, 43, 44], [2, 22, 23, 42, 43], [0, 1, 22, 23, 44], [1, 2, 21, 22, 42],
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates.flip_bit(to);

                let mut shape_index = 59;
                for i in 0..4 {
                    for p in 0..5 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p] | shape_index << 9;
                            let piece = Bitboard::with_piece(action);
                            if piece & legal_fields == piece {
                                actionlist.push(Action::Set(action, PieceType::WPentomino));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::NPentomino as usize][self.current_player as usize] {
            let mut candidates = must_fields;
            let offsets: [[u16; 4]; 8] = [
                [1, 42, 43, 63], [0, 42, 43, 64], [1, 21, 22, 63], [0, 21, 22, 64],
                [2, 3, 21, 23], [0, 2, 23, 24], [0, 1, 22, 24], [1, 3, 21, 22]
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates.flip_bit(to);

                let mut shape_index = 63;
                for i in 0..8 {
                    for p in 0..4 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p] | shape_index << 9;
                            let piece = Bitboard::with_piece(action);
                            if piece & legal_fields == piece {
                                actionlist.push(Action::Set(action, PieceType::NPentomino));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::VPentomino as usize][self.current_player as usize] {
            let mut candidates = must_fields;
            let offsets: [[u16; 3]; 4] = [
                [0, 2, 42], [2, 42, 44], [0, 2, 44], [0, 42, 44]
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates.flip_bit(to);

                let mut shape_index = 71;
                for i in 0..4 {
                    for p in 0..3 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p] | shape_index << 9;
                            let piece = Bitboard::with_piece(action);
                            if piece & legal_fields == piece {
                                actionlist.push(Action::Set(action, PieceType::VPentomino));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::PPentomino as usize][self.current_player as usize] {
            let mut candidates = must_fields;
            let offsets: [[u16; 4]; 8] = [
                [0, 1, 22, 42], [0, 1, 21, 43], [0, 1, 21, 23], [0, 2, 21, 22],
                [0, 2, 22, 23], [1, 2, 21, 23], [1, 21, 42, 43], [0, 22, 42, 43]
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates.flip_bit(to);

                let mut shape_index = 75;
                for i in 0..8 {
                    for p in 0..4 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p] | shape_index << 9;
                            let piece = Bitboard::with_piece(action);
                            if piece & legal_fields == piece {
                                actionlist.push(Action::Set(action, PieceType::PPentomino));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::YPentomino as usize][self.current_player as usize] {
            let mut candidates = must_fields;
            let offsets: [[u16; 4]; 8] = [
                [0, 22, 42, 63], [0, 21, 43, 63], [1, 22, 42, 64], [1, 21, 43, 64],
                [0, 1, 3, 23], [0, 2, 3, 22], [2, 21, 22, 24], [1, 21, 23, 24]
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates.flip_bit(to);

                let mut shape_index = 83;
                for i in 0..8 {
                    for p in 0..4 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p] | shape_index << 9;
                            let piece = Bitboard::with_piece(action);
                            if piece & legal_fields == piece {
                                actionlist.push(Action::Set(action, PieceType::YPentomino));
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
                must_fields.flip_bit(to);
                actionlist.push(Action::Set(to, PieceType::Monomino));
            }
        }
        if actionlist.size == 0 {
            actionlist.push(Action::Pass);
        }
    }

    pub fn game_result(&self) -> i16 {
        let mut blue_score = self.board[Color::BLUE as usize].count_ones() as i16;
        let mut yellow_score = self.board[Color::YELLOW as usize].count_ones() as i16;
        let mut red_score = self.board[Color::RED as usize].count_ones() as i16;
        let mut green_score = self.board[Color::GREEN as usize].count_ones() as i16;

        blue_score += (blue_score == 89) as i16 * 15
            + self.monomino_placed_last[Color::BLUE as usize] as i16 * 5;
        yellow_score += (yellow_score == 89) as i16 * 15
            + self.monomino_placed_last[Color::YELLOW as usize] as i16 * 5;
        red_score += (red_score == 89) as i16 * 15
            + self.monomino_placed_last[Color::RED as usize] as i16 * 5;
        green_score += (green_score == 89) as i16 * 15
            + self.monomino_placed_last[Color::GREEN as usize] as i16 * 5;

        blue_score + yellow_score - red_score - green_score
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
                    string.push_str("🟦");
                } else if self.board[1] & bit == bit {
                    string.push_str("🟨");
                } else if self.board[2] & bit == bit {
                    string.push_str("🟥");
                } else if self.board[3] & bit == bit {
                    string.push_str("🟩");
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
