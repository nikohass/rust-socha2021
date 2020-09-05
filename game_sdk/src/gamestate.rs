use super::action::Action;
use super::actionlist::ActionList;
use super::bitboard::Bitboard;
use super::color::Color;
use super::constants::{START_FIELDS, VALID_FIELDS};
use super::direction::{Direction, DIRECTIONS};
use super::piece_type::{PieceType, PIECE_TYPES};
use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Eq, PartialEq)]
pub struct GameState {
    pub ply: u8,
    pub board: [Bitboard; 4],
    pub current_player: Color,
    pub pieces_left: [[bool; 4]; 21],
    pub monomino_placed_last: [bool; 4],
    pub skipped: u8,
    pub start_piece_type: PieceType,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            ply: 0,
            board: [Bitboard::new(); 4],
            current_player: Color::BLUE,
            pieces_left: [[true; 4]; 21],
            monomino_placed_last: [false; 4],
            skipped: 0,
            start_piece_type: PieceType::random_pentomino(),
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
            Action::Skip => {
                self.skipped |= 1 << self.current_player as usize;
            }
            Action::Set(to, piece_type, piece_shape) => {
                let piece = Bitboard::with_piece(to, piece_shape);
                self.skipped &= !1 << self.current_player as usize;

                debug_assert!(
                    !((self.board[0] | self.board[1] | self.board[2] | self.board[3]) & piece)
                        .not_zero(),
                    "Piece can't be placed on other pieces."
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
            Action::Skip => {
                self.skipped &= !1 << self.current_player as usize;
            }
            Action::Set(to, piece_type, piece_shape) => {
                self.skipped |= 1 << self.current_player as usize;
                let piece = Bitboard::with_piece(to, piece_shape);
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

    pub fn get_possible_actions(&self, action_list: &mut ActionList) {
        // fields of the current player
        let own_fields = self.board[self.current_player as usize];
        let other_fields =
            (self.board[0] | self.board[1] | self.board[2] | self.board[3]) & !own_fields;

        // all fields that are empty and aren't next to own fields
        let legal_fields = !(own_fields | other_fields | own_fields.neighbours()) & VALID_FIELDS;
        // every placed piece has to touch at least one of these fields
        let mut placement_fields = if self.ply > 3 {
            own_fields.diagonal_neighbours() & legal_fields
        } else {
            START_FIELDS & !other_fields
        };

        let with_two_in_a_row = placement_fields
            & (legal_fields << 1 | legal_fields >> 1 | legal_fields << 21 | legal_fields >> 21);
        let with_three_in_a_row = with_two_in_a_row
            & (legal_fields << 2 | legal_fields >> 2 | legal_fields << 42 | legal_fields >> 42);

        debug_assert!(
            own_fields & VALID_FIELDS == own_fields,
            "Own fields are not valid fields."
        );
        debug_assert!(
            other_fields & VALID_FIELDS == other_fields,
            "Other fields are not valid fields."
        );

        if self.pieces_left[PieceType::XPentomino as usize][self.current_player as usize] {
            let mut candidates = with_three_in_a_row;

            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates.flip_bit(to);

                for offset in [0, 20, 22, 42].iter() {
                    if to >= *offset {
                        let to = to - *offset;
                        let piece = Bitboard::with_piece(to, 10);
                        if piece & legal_fields == piece {
                            action_list.push(Action::Set(to, PieceType::XPentomino, 10));
                        }
                    }
                }
            }
        }

        for d in DIRECTIONS.iter() {
            let mut two_in_a_row = legal_fields & placement_fields.neighbours_in_direction(*d);
            let mut three_in_a_row = legal_fields & two_in_a_row.neighbours_in_direction(*d);
            let mut four_in_a_row = legal_fields & three_in_a_row.neighbours_in_direction(*d);
            let mut five_in_a_row = legal_fields & four_in_a_row.neighbours_in_direction(*d);

            if self.pieces_left[PieceType::OTetromino as usize][self.current_player as usize] {
                let mut candidates = with_two_in_a_row;

                while candidates.not_zero() {
                    let to = candidates.trailing_zeros();
                    candidates.flip_bit(to);

                    let action = match *d {
                        Direction::RIGHT => to,
                        Direction::UP => {
                            if to != 0 {
                                to - 1
                            } else {
                                0
                            }
                        }
                        Direction::LEFT => {
                            if to > 21 {
                                to - 22
                            } else {
                                0
                            }
                        }
                        Direction::DOWN => {
                            if to > 20 {
                                to - 21
                            } else {
                                0
                            }
                        }
                    };
                    if action != 0 {
                        let piece = Bitboard::with_piece(action, 9);
                        if piece & legal_fields == piece {
                            action_list.push(Action::Set(action, PieceType::OTetromino, 9));
                        }
                    }
                }
            }

            if self.pieces_left[PieceType::LTromino as usize][self.current_player as usize] {
                let mut candidates =
                    two_in_a_row & legal_fields.neighbours_in_direction(d.clockwise());

                while candidates.not_zero() {
                    let to = candidates.trailing_zeros();
                    candidates.flip_bit(to);

                    let action = match *d {
                        Direction::UP => {
                            if to > 20 {
                                to - 21 | 11 << 9
                            } else {
                                0
                            }
                        }
                        Direction::DOWN => to | 12 << 9,
                        Direction::LEFT => {
                            if to != 0 {
                                to - 1 | 13 << 9
                            } else {
                                0
                            }
                        }
                        Direction::RIGHT => to | 14 << 9,
                    };
                    if action != 0 {
                        let piece = Bitboard::with_piece(action & 511, action as usize >> 9);
                        if piece & legal_fields == piece {
                            action_list.push(Action::Set(
                                action & 511,
                                PieceType::LTromino,
                                action as usize >> 9,
                            ));
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
                        Direction::LEFT => {
                            if to != 0 {
                                to - 1 | 12 << 9
                            } else {
                                0
                            }
                        }
                        Direction::UP => {
                            if to > 21 {
                                to - 22 | 13 << 9
                            } else {
                                0
                            }
                        }
                        Direction::RIGHT => {
                            if to > 20 {
                                to - 21 | 14 << 9
                            } else {
                                0
                            }
                        }
                    };
                    if action != 0 {
                        let piece = Bitboard::with_piece(action & 511, action as usize >> 9);
                        if piece & legal_fields == piece {
                            action_list.push(Action::Set(
                                action & 511,
                                PieceType::LTromino,
                                action as usize >> 9,
                            ));
                        }
                    }
                }
            }

            if self.pieces_left[PieceType::Domino as usize][self.current_player as usize] {
                while two_in_a_row.not_zero() {
                    let to = two_in_a_row.trailing_zeros();
                    two_in_a_row.flip_bit(to);
                    action_list.push(match *d {
                        Direction::RIGHT => Action::Set(to, PieceType::Domino, 1),
                        Direction::LEFT => Action::Set(to - 1, PieceType::Domino, 1),
                        Direction::UP => Action::Set(to - 21, PieceType::Domino, 2),
                        Direction::DOWN => Action::Set(to, PieceType::Domino, 2),
                    });
                }
            }

            if self.pieces_left[PieceType::ITromino as usize][self.current_player as usize] {
                while three_in_a_row.not_zero() {
                    let to = three_in_a_row.trailing_zeros();
                    three_in_a_row.flip_bit(to);
                    action_list.push(match *d {
                        Direction::RIGHT => Action::Set(to, PieceType::ITromino, 3),
                        Direction::LEFT => Action::Set(to - 2, PieceType::ITromino, 3),
                        Direction::UP => Action::Set(to - 42, PieceType::ITromino, 4),
                        Direction::DOWN => Action::Set(to, PieceType::ITromino, 4),
                    });
                }
            }

            if self.pieces_left[PieceType::ITetromino as usize][self.current_player as usize] {
                while four_in_a_row.not_zero() {
                    let to = four_in_a_row.trailing_zeros();
                    four_in_a_row.flip_bit(to);
                    action_list.push(match *d {
                        Direction::RIGHT => Action::Set(to, PieceType::ITetromino, 5),
                        Direction::LEFT => Action::Set(to - 3, PieceType::ITetromino, 5),
                        Direction::UP => Action::Set(to - 63, PieceType::ITetromino, 6),
                        Direction::DOWN => Action::Set(to, PieceType::ITetromino, 6),
                    });
                }
            }

            if self.pieces_left[PieceType::IPentomino as usize][self.current_player as usize] {
                while five_in_a_row.not_zero() {
                    let to = five_in_a_row.trailing_zeros();
                    five_in_a_row.flip_bit(to);
                    action_list.push(match *d {
                        Direction::RIGHT => Action::Set(to, PieceType::IPentomino, 7),
                        Direction::LEFT => Action::Set(to - 4, PieceType::IPentomino, 7),
                        Direction::UP => Action::Set(to - 84, PieceType::IPentomino, 8),
                        Direction::DOWN => Action::Set(to, PieceType::IPentomino, 8),
                    });
                }
            }
        }

        if self.pieces_left[PieceType::LTetromino as usize][self.current_player as usize] {
            let mut candidates = with_two_in_a_row;
            let offsets: [[u16; 3]; 8] = [
                [0, 1, 42],
                [0, 1, 43],
                [1, 43, 42],
                [0, 42, 43],
                [0, 21, 23],
                [0, 2, 21],
                [0, 2, 23],
                [2, 21, 23],
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates.flip_bit(to);

                let mut shape_index: usize = 15;
                for i in 0..8 {
                    for p in 0..3 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p];
                            let piece = Bitboard::with_piece(action, shape_index);
                            if piece & legal_fields == piece {
                                action_list.push(Action::Set(
                                    action,
                                    PieceType::LTetromino,
                                    shape_index,
                                ));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::LPentomino as usize][self.current_player as usize] {
            let mut candidates = with_two_in_a_row;
            let offsets: [[u16; 3]; 8] = [
                [0, 3, 24],
                [0, 3, 21],
                [0, 21, 24],
                [3, 21, 24],
                [0, 1, 63],
                [0, 63, 64],
                [0, 1, 64],
                [1, 63, 64],
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates.flip_bit(to);

                let mut shape_index: usize = 23;
                for i in 0..8 {
                    for p in 0..3 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p];
                            let piece = Bitboard::with_piece(action, shape_index);
                            if piece & legal_fields == piece {
                                action_list.push(Action::Set(
                                    action,
                                    PieceType::LPentomino,
                                    shape_index,
                                ));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::TPentomino as usize][self.current_player as usize] {
            let mut candidates = with_three_in_a_row;
            let offsets: [[u16; 3]; 4] = [[0, 2, 43], [1, 42, 44], [0, 23, 42], [2, 21, 44]];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates.flip_bit(to);

                let mut shape_index: usize = 31;
                for i in 0..4 {
                    for p in 0..3 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p];
                            let piece = Bitboard::with_piece(action, shape_index);
                            if piece & legal_fields == piece {
                                action_list.push(Action::Set(
                                    action,
                                    PieceType::TPentomino,
                                    shape_index,
                                ));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::TTetromino as usize][self.current_player as usize] {
            let mut candidates = with_two_in_a_row;
            let offsets: [[u16; 3]; 4] = [[0, 2, 22], [1, 21, 23], [0, 22, 42], [1, 21, 43]];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates.flip_bit(to);

                let mut shape_index: usize = 35;
                for i in 0..4 {
                    for p in 0..3 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p];
                            let piece = Bitboard::with_piece(action, shape_index);
                            if piece & legal_fields == piece {
                                action_list.push(Action::Set(
                                    action,
                                    PieceType::TTetromino,
                                    shape_index,
                                ));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::ZTetromino as usize][self.current_player as usize] {
            let mut candidates = with_two_in_a_row;
            let offsets: [[u16; 4]; 4] = [
                [1, 2, 21, 22],
                [0, 1, 22, 23],
                [1, 21, 22, 42],
                [0, 21, 22, 43],
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates.flip_bit(to);

                let mut shape_index: usize = 39;
                for i in 0..4 {
                    for p in 0..4 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p];
                            let piece = Bitboard::with_piece(action, shape_index);
                            if piece & legal_fields == piece {
                                action_list.push(Action::Set(
                                    action,
                                    PieceType::ZTetromino,
                                    shape_index,
                                ));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::ZPentomino as usize][self.current_player as usize] {
            let mut candidates = with_two_in_a_row;
            let offsets: [[u16; 4]; 4] = [
                [0, 21, 23, 44],
                [2, 21, 23, 42],
                [1, 2, 42, 43],
                [0, 1, 43, 44],
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates.flip_bit(to);

                let mut shape_index: usize = 43;
                for i in 0..4 {
                    for p in 0..4 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p];
                            let piece = Bitboard::with_piece(action, shape_index);
                            if piece & legal_fields == piece {
                                action_list.push(Action::Set(
                                    action,
                                    PieceType::ZPentomino,
                                    shape_index,
                                ));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::UPentomino as usize][self.current_player as usize] {
            let mut candidates = with_two_in_a_row;
            let offsets: [[u16; 4]; 4] = [
                [0, 2, 21, 23],
                [0, 2, 21, 23],
                [0, 1, 42, 43],
                [0, 1, 42, 43],
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates.flip_bit(to);

                let mut shape_index: usize = 47;
                for i in 0..4 {
                    for p in 0..4 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p];
                            let piece = Bitboard::with_piece(action, shape_index);
                            if piece & legal_fields == piece {
                                action_list.push(Action::Set(
                                    action,
                                    PieceType::UPentomino,
                                    shape_index,
                                ));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::FPentomino as usize][self.current_player as usize] {
            let mut candidates = with_two_in_a_row;
            let offsets: [[u16; 4]; 8] = [
                [1, 23, 42, 43],
                [1, 21, 43, 44],
                [1, 2, 21, 43],
                [0, 1, 23, 43],
                [2, 21, 23, 43],
                [0, 21, 23, 43],
                [1, 21, 23, 44],
                [1, 21, 23, 42],
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates.flip_bit(to);

                let mut shape_index: usize = 51;
                for i in 0..8 {
                    for p in 0..4 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p];
                            let piece = Bitboard::with_piece(action, shape_index);
                            if piece & legal_fields == piece {
                                action_list.push(Action::Set(
                                    action,
                                    PieceType::FPentomino,
                                    shape_index,
                                ));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::WPentomino as usize][self.current_player as usize] {
            let mut candidates = with_two_in_a_row;
            let offsets: [[u16; 5]; 4] = [
                [0, 21, 22, 43, 44],
                [2, 22, 23, 42, 43],
                [0, 1, 22, 23, 44],
                [1, 2, 21, 22, 42],
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates.flip_bit(to);

                let mut shape_index: usize = 59;
                for i in 0..4 {
                    for p in 0..5 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p];
                            let piece = Bitboard::with_piece(action, shape_index);
                            if piece & legal_fields == piece {
                                action_list.push(Action::Set(
                                    action,
                                    PieceType::WPentomino,
                                    shape_index,
                                ));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::NPentomino as usize][self.current_player as usize] {
            let mut candidates = with_two_in_a_row;
            let offsets: [[u16; 4]; 8] = [
                [1, 42, 43, 63],
                [0, 42, 43, 64],
                [1, 21, 22, 63],
                [0, 21, 22, 64],
                [2, 3, 21, 23],
                [0, 2, 23, 24],
                [0, 1, 22, 24],
                [1, 3, 21, 22],
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates.flip_bit(to);

                let mut shape_index: usize = 63;
                for i in 0..8 {
                    for p in 0..4 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p];
                            let piece = Bitboard::with_piece(action, shape_index);
                            if piece & legal_fields == piece {
                                action_list.push(Action::Set(
                                    action,
                                    PieceType::NPentomino,
                                    shape_index,
                                ));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::VPentomino as usize][self.current_player as usize] {
            let mut candidates = with_three_in_a_row;
            let offsets: [[u16; 3]; 4] = [[0, 2, 42], [2, 42, 44], [0, 2, 44], [0, 42, 44]];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates.flip_bit(to);

                let mut shape_index: usize = 71;
                for i in 0..4 {
                    for p in 0..3 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p];
                            let piece = Bitboard::with_piece(action, shape_index);
                            if piece & legal_fields == piece {
                                action_list.push(Action::Set(
                                    action,
                                    PieceType::VPentomino,
                                    shape_index,
                                ));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::PPentomino as usize][self.current_player as usize] {
            let mut candidates = with_two_in_a_row;
            let offsets: [[u16; 4]; 8] = [
                [0, 1, 22, 42],
                [0, 1, 21, 43],
                [0, 1, 21, 23],
                [0, 2, 21, 22],
                [0, 2, 22, 23],
                [1, 2, 21, 23],
                [1, 21, 42, 43],
                [0, 22, 42, 43],
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates.flip_bit(to);

                let mut shape_index: usize = 75;
                for i in 0..8 {
                    for p in 0..4 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p];
                            let piece = Bitboard::with_piece(action, shape_index);
                            if piece & legal_fields == piece {
                                action_list.push(Action::Set(
                                    action,
                                    PieceType::PPentomino,
                                    shape_index,
                                ));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::YPentomino as usize][self.current_player as usize] {
            let mut candidates = with_two_in_a_row;
            let offsets: [[u16; 4]; 8] = [
                [0, 22, 42, 63],
                [0, 21, 43, 63],
                [1, 22, 42, 64],
                [1, 21, 43, 64],
                [0, 1, 3, 23],
                [0, 2, 3, 22],
                [2, 21, 22, 24],
                [1, 21, 23, 24],
            ];
            while candidates.not_zero() {
                let to = candidates.trailing_zeros();
                candidates.flip_bit(to);

                let mut shape_index: usize = 83;
                for i in 0..8 {
                    for p in 0..4 {
                        if to >= offsets[i][p] {
                            let action = to - offsets[i][p];
                            let piece = Bitboard::with_piece(action, shape_index);
                            if piece & legal_fields == piece {
                                action_list.push(Action::Set(
                                    action,
                                    PieceType::YPentomino,
                                    shape_index,
                                ));
                            }
                        }
                    }
                    shape_index += 1;
                }
            }
        }

        if self.pieces_left[PieceType::Monomino as usize][self.current_player as usize] {
            while placement_fields.not_zero() {
                let to = placement_fields.trailing_zeros();
                placement_fields.flip_bit(to);
                action_list.push(Action::Set(to, PieceType::Monomino, 0));
            }
        }

        if self.ply / 4 == 0 {
            let mut idx = 0;
            for i in 0..action_list.size {
                match action_list[i] {
                    Action::Set(_, piece_type, _) => {
                        if piece_type == self.start_piece_type {
                            action_list.swap(idx, i);
                            idx += 1;
                        }
                    }
                    _ => {}
                };
            }
            action_list.size = idx;
        }

        if action_list.size == 0 {
            action_list.push(Action::Skip);
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.skipped == 15 || self.ply / 4 == 26 // the game is over after round 25 or when all players skipped
    }

    pub fn game_result(&self) -> i16 {
        let mut blue_score = self.board[Color::BLUE as usize].count_ones() as i16;
        let mut yellow_score = self.board[Color::YELLOW as usize].count_ones() as i16;
        let mut red_score = self.board[Color::RED as usize].count_ones() as i16;
        let mut green_score = self.board[Color::GREEN as usize].count_ones() as i16;

        if blue_score == 89 {
            if self.monomino_placed_last[Color::BLUE as usize] {
                blue_score += 20;
            } else {
                blue_score += 15;
            }
        }
        if yellow_score == 89 {
            if self.monomino_placed_last[Color::BLUE as usize] {
                yellow_score += 20;
            } else {
                yellow_score += 15;
            }
        }
        if red_score == 89 {
            if self.monomino_placed_last[Color::BLUE as usize] {
                red_score += 20;
            } else {
                red_score += 15;
            }
        }
        if green_score == 89 {
            if self.monomino_placed_last[Color::BLUE as usize] {
                green_score += 20;
            } else {
                green_score += 15;
            }
        }

        blue_score + yellow_score - red_score - green_score
    }

    pub fn piece_info_to_int(&self) -> u128 {
        let mut info: u128 = 0;
        for player_index in 0..4 {
            if self.monomino_placed_last[player_index as usize] {
                info |= 1 << player_index;
            }
            for i in 0..21 {
                if self.pieces_left[i as usize][player_index as usize] {
                    info |= 1 << (i + 21 * player_index + 4);
                }
            }
        }
        for start_piece_index in 0..21 {
            if PIECE_TYPES[start_piece_index] == self.start_piece_type {
                info |= (start_piece_index as u128) << 110;
                break;
            }
        }
        info | (self.skipped as u128) << 120
    }

    pub fn int_to_piece_info(&mut self, info: u128) {
        self.skipped = (info >> 120) as u8;
        for player_index in 0..4 {
            self.monomino_placed_last[player_index as usize] = (1 << player_index) & info != 0;
            for i in 0..21 {
                self.pieces_left[i as usize][player_index as usize] =
                    info & 1 << (i + 21 * player_index + 4) != 0;
            }
        }
        let start_piece_index = info >> 110 & 31;
        self.start_piece_type = PIECE_TYPES[start_piece_index as usize];
    }

    pub fn to_fen(&self) -> String {
        let mut string = String::new();
        string.push_str(&format!(
            "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            self.ply,
            self.board[0].one,
            self.board[0].two,
            self.board[0].three,
            self.board[0].four,
            self.board[1].one,
            self.board[1].two,
            self.board[1].three,
            self.board[1].four,
            self.board[2].one,
            self.board[2].two,
            self.board[2].three,
            self.board[2].four,
            self.board[3].one,
            self.board[3].two,
            self.board[3].three,
            self.board[3].four,
            self.piece_info_to_int()
        ));
        string
    }

    pub fn from_fen(string: String) -> GameState {
        let mut entries: Vec<&str> = string.split(" ").collect();
        let mut state = GameState::new();
        state.ply = entries.remove(0).parse::<u8>().unwrap();
        state.current_player = match state.ply % 4 {
            0 => Color::BLUE,
            1 => Color::YELLOW,
            2 => Color::RED,
            _ => Color::GREEN,
        };

        for board_index in 0..4 {
            state.board[board_index].one = entries.remove(0).parse::<u128>().unwrap();
            state.board[board_index].two = entries.remove(0).parse::<u128>().unwrap();
            state.board[board_index].three = entries.remove(0).parse::<u128>().unwrap();
            state.board[board_index].four = entries.remove(0).parse::<u128>().unwrap();
        }
        state.int_to_piece_info(entries.remove(0).parse::<u128>().unwrap());
        state
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
            "║ {} Turn: {} Round: {}",
            self.current_player.to_string(),
            self.ply,
            self.ply / 4
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
                let y = 19 - i;
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
