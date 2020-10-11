use super::action::Action;
use super::actionlist::ActionList;
use super::bitboard::Bitboard;
use super::color::Color;
use super::constants::{START_FIELDS, VALID_FIELDS};
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
    pub hash: u64,
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
            hash: 0,
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
        debug_assert!(self.validate_action(&action), "Action is invalid");
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
                    "Piece can't be placed on other pieces. Move was {}\n{}",
                    action.to_string(),
                    Bitboard::with_piece(to, piece_shape).to_string(),
                );
                debug_assert!(
                    self.pieces_left[piece_type as usize][self.current_player as usize],
                    "Cannot place piece that has already been placed."
                );
                self.pieces_left[piece_type as usize][self.current_player as usize] = false;
                self.board[self.current_player as usize] ^= piece;
                self.hash += ((to as u64) + 1) * ((piece_shape as u64) * 419 + 1);
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
                let piece = Bitboard::with_piece(to, piece_shape);
                debug_assert!(
                    !self.pieces_left[piece_type as usize][self.current_player as usize],
                    "Cannot remove piece that has not been placed."
                );
                self.pieces_left[piece_type as usize][self.current_player as usize] = true;
                self.board[self.current_player as usize] ^= piece;
                self.hash -= ((to as u64) + 1) * ((piece_shape as u64) * 419 + 1);
            }
        };
        debug_assert!(self.check_integrity());
    }

    pub fn validate_action(&self, action: &Action) -> bool {
        match action {
            Action::Skip => true,
            Action::Set(to, piece_type, shape_index) => {
                let mut is_valid = true;
                if !self.pieces_left[*piece_type as usize][self.current_player as usize] {
                    println!("Cannot place piece that has already been placed.");
                    return false;
                }
                let piece = Bitboard::with_piece(*to, *shape_index);
                let own_fields = self.board[self.current_player as usize];
                let other_fields =
                    (self.board[0] | self.board[1] | self.board[2] | self.board[3]) & !own_fields;
                let legal_fields =
                    !(own_fields | other_fields | own_fields.neighbours()) & VALID_FIELDS;
                let placement_fields = if self.ply > 3 {
                    own_fields.diagonal_neighbours() & legal_fields
                } else {
                    START_FIELDS & !other_fields
                };
                if (piece & placement_fields).is_zero() {
                    println!("Piece does not touch a corner");
                    is_valid = false;
                }
                if piece & legal_fields != piece {
                    println!("Piece destination is not valid");
                    is_valid = false;
                }
                if piece_type.piece_size() != piece.count_ones() as u8 {
                    println!("Piece is shifted of the board");
                    is_valid = false;
                }
                if !is_valid {
                    println!("{}", action.to_string());
                    println!("{}", piece.to_string());
                }
                is_valid
            }
        }
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

        debug_assert!(
            own_fields & VALID_FIELDS == own_fields,
            "Own fields are not valid fields."
        );
        debug_assert!(
            other_fields & VALID_FIELDS == other_fields,
            "Other fields are not valid fields."
        );

        let two_right = legal_fields & (legal_fields >> 1 & VALID_FIELDS);
        let two_left = legal_fields & (legal_fields << 1 & VALID_FIELDS);
        let two_down = legal_fields & (legal_fields >> 21 & VALID_FIELDS);
        let two_up = legal_fields & (legal_fields << 21 & VALID_FIELDS);

        let square = two_right & two_right >> 21;

        let three_right = two_right & (legal_fields >> 2 & VALID_FIELDS);
        let three_left = two_left & (legal_fields << 2 & VALID_FIELDS);
        let three_down = two_down & (legal_fields >> 42 & VALID_FIELDS);
        let three_up = two_up & (legal_fields << 42 & VALID_FIELDS);

        let four_right = three_right & (legal_fields >> 3 & VALID_FIELDS);
        let four_left = three_left & (legal_fields << 3 & VALID_FIELDS);
        let four_down = three_down & (legal_fields >> 63 & VALID_FIELDS);
        let four_up = three_up & (legal_fields << 63 & VALID_FIELDS);

        if self.pieces_left[PieceType::Domino as usize][self.current_player as usize] {
            let mut destinations =
                (two_right & placement_fields) | (two_left & placement_fields) >> 1;
            while destinations.not_zero() {
                let to = destinations.trailing_zeros();
                destinations.flip_bit(to);
                action_list.push(Action::Set(to, PieceType::Domino, 1));
            }
            destinations = (two_down & placement_fields) | (two_up & placement_fields) >> 21;
            while destinations.not_zero() {
                let to = destinations.trailing_zeros();
                destinations.flip_bit(to);
                action_list.push(Action::Set(to, PieceType::Domino, 2));
            }
        }

        if self.pieces_left[PieceType::ITromino as usize][self.current_player as usize] {
            let mut destinations =
                (three_right & placement_fields) | (three_left & placement_fields) >> 2;
            while destinations.not_zero() {
                let to = destinations.trailing_zeros();
                destinations.flip_bit(to);
                action_list.push(Action::Set(to, PieceType::ITromino, 3));
            }
            destinations = (three_up & placement_fields) >> 42 | (three_down & placement_fields);
            while destinations.not_zero() {
                let to = destinations.trailing_zeros();
                destinations.flip_bit(to);
                action_list.push(Action::Set(to, PieceType::ITromino, 4));
            }
        }

        if self.pieces_left[PieceType::ITetromino as usize][self.current_player as usize] {
            let mut destinations =
                (four_right & placement_fields) | (four_left & placement_fields) >> 3;
            while destinations.not_zero() {
                let to = destinations.trailing_zeros();
                destinations.flip_bit(to);
                action_list.push(Action::Set(to, PieceType::ITetromino, 5));
            }
            destinations = (four_down & placement_fields) | (four_up & placement_fields) >> 63;
            while destinations.not_zero() {
                let to = destinations.trailing_zeros();
                destinations.flip_bit(to);
                action_list.push(Action::Set(to, PieceType::ITetromino, 6));
            }
        }

        if self.pieces_left[PieceType::IPentomino as usize][self.current_player as usize] {
            let mut destinations = (four_right & legal_fields >> 4 & placement_fields)
                | (four_left & legal_fields << 4 & placement_fields) >> 4;
            while destinations.not_zero() {
                let to = destinations.trailing_zeros();
                destinations.flip_bit(to);
                action_list.push(Action::Set(to, PieceType::IPentomino, 7));
            }
            destinations = (four_down & legal_fields >> 84 & placement_fields)
                | (four_up & legal_fields << 84 & placement_fields) >> 84;
            while destinations.not_zero() {
                let to = destinations.trailing_zeros();
                destinations.flip_bit(to);
                action_list.push(Action::Set(to, PieceType::IPentomino, 8));
            }
        }

        if self.pieces_left[PieceType::XPentomino as usize][self.current_player as usize] {
            let mut destinations = (three_right >> 20 & three_down)
                & (placement_fields
                    | placement_fields >> 20
                    | placement_fields >> 22
                    | placement_fields >> 42);
            while destinations.not_zero() {
                let to = destinations.trailing_zeros();
                destinations.flip_bit(to);
                action_list.push(Action::Set(to, PieceType::XPentomino, 10));
            }
        }

        if self.pieces_left[PieceType::LTromino as usize][self.current_player as usize] {
            for shape_index in 11..15 {
                let mut destinations = match shape_index {
                    11 => {
                        (two_up & two_right) >> 21
                            & (placement_fields | placement_fields >> 21 | placement_fields >> 22)
                    }
                    12 => {
                        (two_down & two_right)
                            & (placement_fields | placement_fields >> 1 | placement_fields >> 21)
                    }
                    13 => {
                        (two_down >> 1 & two_right)
                            & (placement_fields | placement_fields >> 1 | placement_fields >> 22)
                    }
                    _ => {
                        (two_down >> 1 & two_right >> 21)
                            & (placement_fields >> 1
                                | placement_fields >> 21
                                | placement_fields >> 22)
                    }
                };
                while destinations.not_zero() {
                    let to = destinations.trailing_zeros();
                    destinations.flip_bit(to);
                    action_list.push(Action::Set(to, PieceType::LTromino, shape_index));
                }
            }
        }

        if self.pieces_left[PieceType::LPentomino as usize][self.current_player as usize] {
            for shape_index in 23..31 {
                let mut destinations = match shape_index {
                    23 => {
                        (four_right & legal_fields >> 24)
                            & (placement_fields | placement_fields >> 3 | placement_fields >> 24)
                    }
                    24 => {
                        (four_right & two_down)
                            & (placement_fields | placement_fields >> 3 | placement_fields >> 21)
                    }
                    25 => {
                        (legal_fields & four_right >> 21)
                            & (placement_fields | placement_fields >> 21 | placement_fields >> 24)
                    }
                    26 => {
                        (four_left & two_up) >> 24
                            & (placement_fields >> 3
                                | placement_fields >> 21
                                | placement_fields >> 24)
                    }
                    27 => {
                        (two_right & four_down)
                            & (placement_fields | placement_fields >> 1 | placement_fields >> 63)
                    }
                    28 => {
                        (four_down & legal_fields >> 64)
                            & (placement_fields | placement_fields >> 63 | placement_fields >> 64)
                    }
                    29 => {
                        (two_right & four_down >> 1)
                            & (placement_fields | placement_fields >> 1 | placement_fields >> 64)
                    }
                    _ => {
                        (four_up & two_left) >> 64
                            & (placement_fields >> 1
                                | placement_fields >> 63
                                | placement_fields >> 64)
                    }
                };
                while destinations.not_zero() {
                    let to = destinations.trailing_zeros();
                    destinations.flip_bit(to);
                    action_list.push(Action::Set(to, PieceType::LPentomino, shape_index));
                }
            }
        }

        if self.pieces_left[PieceType::TPentomino as usize][self.current_player as usize] {
            for shape_index in 31..35 {
                let mut destinations = match shape_index {
                    31 => {
                        (three_right & three_down >> 1)
                            & (placement_fields | placement_fields >> 2 | placement_fields >> 43)
                    }
                    32 => {
                        (two_left & two_right & three_up) >> 43
                            & (placement_fields >> 1
                                | placement_fields >> 42
                                | placement_fields >> 44)
                    }
                    33 => {
                        (three_down & three_right >> 21)
                            & (placement_fields | placement_fields >> 23 | placement_fields >> 42)
                    }
                    _ => {
                        (three_left & two_up & two_down) >> 23
                            & (placement_fields >> 2
                                | placement_fields >> 21
                                | placement_fields >> 44)
                    }
                };
                while destinations.not_zero() {
                    let to = destinations.trailing_zeros();
                    destinations.flip_bit(to);
                    action_list.push(Action::Set(to, PieceType::TPentomino, shape_index));
                }
            }
        }

        if self.pieces_left[PieceType::ZPentomino as usize][self.current_player as usize] {
            for shape_index in 43..47 {
                let mut destinations = match shape_index {
                    43 => {
                        (legal_fields & (three_left & two_down) >> 23)
                            & (placement_fields
                                | placement_fields >> 21
                                | placement_fields >> 23
                                | placement_fields >> 44)
                    }
                    44 => {
                        ((legal_fields & (three_right & two_down) >> 19)
                            & (placement_fields
                                | placement_fields >> 19
                                | placement_fields >> 21
                                | placement_fields >> 40)) >> 2
                    }
                    45 => {
                        (legal_fields >> 2 & (three_up & two_left) >> 43)
                            & (placement_fields >> 1
                                | placement_fields >> 2
                                | placement_fields >> 42
                                | placement_fields >> 43)
                    }
                    _ => {
                        (two_right & (two_right & three_up) >> 43)
                            & (placement_fields
                                | placement_fields >> 1
                                | placement_fields >> 43
                                | placement_fields >> 44)
                    }
                };
                while destinations.not_zero() {
                    let to = destinations.trailing_zeros();
                    destinations.flip_bit(to);
                    action_list.push(Action::Set(to, PieceType::ZPentomino, shape_index));
                }
            }
        }

        if self.pieces_left[PieceType::UPentomino as usize][self.current_player as usize] {
            for shape_index in 47..51 {
                let mut destinations = match shape_index {
                    47 => {
                        (three_right & two_down & legal_fields >> 23)
                            & (placement_fields
                                | placement_fields >> 2
                                | placement_fields >> 21
                                | placement_fields >> 23)
                    }
                    48 => {
                        (legal_fields & (three_left & two_up) >> 23)
                            & (placement_fields
                                | placement_fields >> 2
                                | placement_fields >> 21
                                | placement_fields >> 23)
                    }
                    49 => {
                        (three_down & two_right & legal_fields >> 43)
                            & (placement_fields
                                | placement_fields >> 1
                                | placement_fields >> 42
                                | placement_fields >> 43)
                    }
                    _ => {
                        (two_right & (two_left & three_up) >> 43)
                            & (placement_fields
                                | placement_fields >> 1
                                | placement_fields >> 42
                                | placement_fields >> 43)
                    }
                };
                while destinations.not_zero() {
                    let to = destinations.trailing_zeros();
                    destinations.flip_bit(to);
                    action_list.push(Action::Set(to, PieceType::UPentomino, shape_index));
                }
            }
        }

        if self.pieces_left[PieceType::FPentomino as usize][self.current_player as usize] {
            for shape_index in 51..59 {
                let mut destinations = match shape_index {
                    51 => {
                        ((three_up & two_left) >> 43 & legal_fields >> 23)
                            & (placement_fields >> 1
                                | placement_fields >> 23
                                | placement_fields >> 42
                                | placement_fields >> 43)
                    }
                    52 => {
                        (legal_fields >> 21 & (three_up & two_right) >> 43)
                            & (placement_fields >> 1
                                | placement_fields >> 21
                                | placement_fields >> 43
                                | placement_fields >> 44)
                    }
                    53 => {
                        (((three_down & two_right) & legal_fields >> 20)
                            & (placement_fields
                                | placement_fields >> 1
                                | placement_fields >> 20
                                | placement_fields >> 42))
                            >> 1
                    }
                    54 => {
                        ((three_down & two_left) >> 1 & legal_fields >> 23)
                            & (placement_fields
                                | placement_fields >> 1
                                | placement_fields >> 23
                                | placement_fields >> 43)
                    }
                    55 => {
                        ((three_left & two_up) >> 23 & legal_fields >> 43)
                            & (placement_fields >> 2
                                | placement_fields >> 21
                                | placement_fields >> 23
                                | placement_fields >> 43)
                    }
                    56 => {
                        ((three_right & two_up) >> 21 & legal_fields >> 43)
                            & (placement_fields
                                | placement_fields >> 21
                                | placement_fields >> 23
                                | placement_fields >> 43)
                    }
                    57 => {
                        ((legal_fields & (three_left & two_down) >> 22)
                            & (placement_fields
                                | placement_fields >> 20
                                | placement_fields >> 22
                                | placement_fields >> 43)) >> 1
                    }
                    _ => {
                        ((legal_fields & (three_right & two_down) >> 20)
                            & (placement_fields
                                | placement_fields >> 20
                                | placement_fields >> 22
                                | placement_fields >> 41)) >> 1
                    }
                };
                while destinations.not_zero() {
                    let to = destinations.trailing_zeros();
                    destinations.flip_bit(to);
                    action_list.push(Action::Set(to, PieceType::FPentomino, shape_index));
                }
            }
        }

        if self.pieces_left[PieceType::WPentomino as usize][self.current_player as usize] {
            for shape_index in 59..63 {
                let mut destinations = match shape_index {
                    59 => {
                        (two_down & (two_up & two_right) >> 43)
                            & (placement_fields
                                | placement_fields >> 21
                                | placement_fields >> 22
                                | placement_fields >> 43
                                | placement_fields >> 44)
                    }
                    60 => {
                        ((two_up & two_left) >> 23 & two_right >> 42)
                            & (placement_fields >> 2
                                | placement_fields >> 22
                                | placement_fields >> 23
                                | placement_fields >> 42
                                | placement_fields >> 43)
                    }
                    61 => {
                        (two_right & (two_down & two_left) >> 23)
                            & (placement_fields
                                | placement_fields >> 1
                                | placement_fields >> 22
                                | placement_fields >> 23
                                | placement_fields >> 44)
                    }
                    _ => {
                        ((two_right & (two_right & two_down) >> 20)
                            & (placement_fields
                                | placement_fields >> 1
                                | placement_fields >> 20
                                | placement_fields >> 21
                                | placement_fields >> 41))
                            >> 1
                    }
                };
                while destinations.not_zero() {
                    let to = destinations.trailing_zeros();
                    destinations.flip_bit(to);
                    action_list.push(Action::Set(to, PieceType::WPentomino, shape_index));
                }
            }
        }

        if self.pieces_left[PieceType::NPentomino as usize][self.current_player as usize] {
            for shape_index in 63..71 {
                let mut destinations = match shape_index {
                    63 => {
                        ((three_down & two_down >> 41)
                            & (placement_fields
                                | placement_fields >> 41
                                | placement_fields >> 42
                                | placement_fields >> 62))
                            >> 1
                    }
                    64 => {
                        (three_down & two_down >> 43)
                            & (placement_fields
                                | placement_fields >> 42
                                | placement_fields >> 43
                                | placement_fields >> 64)
                    }
                    65 => {
                        ((two_down & three_down >> 20)
                            & (placement_fields
                                | placement_fields >> 20
                                | placement_fields >> 21
                                | placement_fields >> 62))
                            >> 1
                    }
                    66 => {
                        (two_down & three_down >> 22)
                            & (placement_fields
                                | placement_fields >> 21
                                | placement_fields >> 22
                                | placement_fields >> 64)
                    }
                    67 => {
                        ((two_right & three_right >> 19)
                            & (placement_fields
                                | placement_fields >> 1
                                | placement_fields >> 19
                                | placement_fields >> 21))
                            >> 2
                    }
                    68 => {
                        (three_right & two_right >> 23)
                            & (placement_fields
                                | placement_fields >> 2
                                | placement_fields >> 23
                                | placement_fields >> 24)
                    }
                    69 => {
                        (two_right & three_right >> 22)
                            & (placement_fields
                                | placement_fields >> 1
                                | placement_fields >> 22
                                | placement_fields >> 24)
                    }
                    _ => {
                        ((three_right & two_right >> 20)
                            & (placement_fields
                                | placement_fields >> 2
                                | placement_fields >> 20
                                | placement_fields >> 21))
                            >> 1
                    }
                };
                while destinations.not_zero() {
                    let to = destinations.trailing_zeros();
                    destinations.flip_bit(to);
                    action_list.push(Action::Set(to, PieceType::NPentomino, shape_index));
                }
            }
        }

        if self.pieces_left[PieceType::VPentomino as usize][self.current_player as usize] {
            for shape_index in 71..75 {
                let mut destinations = match shape_index {
                    71 => {
                        (three_right & three_down)
                            & (placement_fields | placement_fields >> 2 | placement_fields >> 42)
                    }
                    72 => {
                        (three_up & three_left) >> 44
                            & (placement_fields >> 2
                                | placement_fields >> 42
                                | placement_fields >> 44)
                    }
                    73 => {
                        (three_right & three_down >> 2)
                            & (placement_fields | placement_fields >> 2 | placement_fields >> 44)
                    }
                    _ => {
                        (three_down & three_right >> 42)
                            & (placement_fields | placement_fields >> 42 | placement_fields >> 44)
                    }
                };
                while destinations.not_zero() {
                    let to = destinations.trailing_zeros();
                    destinations.flip_bit(to);
                    action_list.push(Action::Set(to, PieceType::VPentomino, shape_index));
                }
            }
        }

        if self.pieces_left[PieceType::YPentomino as usize][self.current_player as usize] {
            for shape_index in 83..91 {
                let mut destinations = match shape_index {
                    83 => {
                        (four_down & legal_fields >> 22)
                            & (placement_fields | placement_fields >> 22 | placement_fields >> 63)
                    }
                    84 => {
                        (four_down & legal_fields >> 43)
                            & (placement_fields | placement_fields >> 43 | placement_fields >> 63)
                    }
                    85 => {
                        ((four_down & legal_fields >> 41)
                            & (placement_fields
                                | placement_fields >> 41
                                | placement_fields >> 63)) >> 1
                    }
                    86 => {
                        ((four_down & legal_fields >> 20)
                            & (placement_fields | placement_fields >> 20 | placement_fields >> 63))
                            >> 1
                    }
                    87 => {
                        (four_right & legal_fields >> 23)
                            & (placement_fields | placement_fields >> 3 | placement_fields >> 23)
                    }
                    88 => {
                        (four_right & legal_fields >> 22)
                            & (placement_fields | placement_fields >> 3 | placement_fields >> 22)
                    }
                    89 => {
                        (two_up & two_right & three_left) >> 23
                            & (placement_fields >> 2
                                | placement_fields >> 21
                                | placement_fields >> 24)
                    }
                    _ => {
                        ((legal_fields & four_right >> 20)
                            & (placement_fields | placement_fields >> 20 | placement_fields >> 23))
                            >> 1
                    }
                };
                while destinations.not_zero() {
                    let to = destinations.trailing_zeros();
                    destinations.flip_bit(to);
                    action_list.push(Action::Set(to, PieceType::YPentomino, shape_index));
                }
            }
        }

        if self.pieces_left[PieceType::TTetromino as usize][self.current_player as usize] {
            for shape_index in 35..39 {
                let mut destinations = match shape_index {
                    35 => {
                        (three_right & legal_fields >> 22)
                            & (placement_fields | placement_fields >> 2 | placement_fields >> 22)
                    }
                    36 => {
                        (two_up & two_right & two_left) >> 22
                            & (placement_fields >> 1
                                | placement_fields >> 21
                                | placement_fields >> 23)
                    }
                    37 => {
                        (three_down & legal_fields >> 22)
                            & (placement_fields | placement_fields >> 22 | placement_fields >> 42)
                    }
                    _ => {
                        (two_up & two_down & two_left) >> 22
                            & (placement_fields >> 1
                                | placement_fields >> 21
                                | placement_fields >> 43)
                    }
                };
                while destinations.not_zero() {
                    let to = destinations.trailing_zeros();
                    destinations.flip_bit(to);
                    action_list.push(Action::Set(to, PieceType::TTetromino, shape_index));
                }
            }
        }

        if self.pieces_left[PieceType::OTetromino as usize][self.current_player as usize] {
            let mut destinations = square
                & (placement_fields
                    | placement_fields >> 1
                    | placement_fields >> 21
                    | placement_fields >> 22);
            while destinations.not_zero() {
                let to = destinations.trailing_zeros();
                destinations.flip_bit(to);
                action_list.push(Action::Set(to, PieceType::OTetromino, 9));
            }
        }

        if self.pieces_left[PieceType::PPentomino as usize][self.current_player as usize] {
            for shape_index in 75..83 {
                let mut destinations = match shape_index {
                    75 => {
                        (square & legal_fields >> 42)
                            & (placement_fields
                                | placement_fields >> 1
                                | placement_fields >> 22
                                | placement_fields >> 42)
                    }
                    76 => {
                        (square & legal_fields >> 43)
                            & (placement_fields
                                | placement_fields >> 1
                                | placement_fields >> 21
                                | placement_fields >> 43)
                    }
                    77 => {
                        (square & legal_fields >> 23)
                            & (placement_fields
                                | placement_fields >> 1
                                | placement_fields >> 21
                                | placement_fields >> 23)
                    }
                    78 => {
                        (square & legal_fields >> 2)
                            & (placement_fields
                                | placement_fields >> 2
                                | placement_fields >> 21
                                | placement_fields >> 22)
                    }
                    79 => {
                        (square >> 1 & legal_fields)
                            & (placement_fields
                                | placement_fields >> 2
                                | placement_fields >> 22
                                | placement_fields >> 23)
                    }
                    80 => {
                        ((square & legal_fields >> 20)
                            & (placement_fields
                                | placement_fields >> 1
                                | placement_fields >> 20
                                | placement_fields >> 22))
                            >> 1
                    }
                    81 => {
                        ((legal_fields & square >> 20)
                            & (placement_fields
                                | placement_fields >> 20
                                | placement_fields >> 41
                                | placement_fields >> 42))
                            >> 1
                    }
                    _ => {
                        (legal_fields & square >> 21)
                            & (placement_fields
                                | placement_fields >> 22
                                | placement_fields >> 42
                                | placement_fields >> 43)
                    }
                };
                while destinations.not_zero() {
                    let to = destinations.trailing_zeros();
                    destinations.flip_bit(to);
                    action_list.push(Action::Set(to, PieceType::PPentomino, shape_index));
                }
            }
        }

        if self.pieces_left[PieceType::ZTetromino as usize][self.current_player as usize] {
            for shape_index in 39..43 {
                let mut destinations = match shape_index {
                    39 => {
                        ((two_right & two_right >> 20)
                            & (placement_fields
                                | placement_fields >> 1
                                | placement_fields >> 20
                                | placement_fields >> 21))
                            >> 1
                    }
                    40 => {
                        (two_right & two_right >> 22)
                            & (placement_fields
                                | placement_fields >> 1
                                | placement_fields >> 22
                                | placement_fields >> 23)
                    }
                    41 => {
                        ((two_down & two_down >> 20)
                            & (placement_fields
                                | placement_fields >> 20
                                | placement_fields >> 21
                                | placement_fields >> 41))
                            >> 1
                    }
                    _ => {
                        (two_down & two_down >> 22)
                            & (placement_fields
                                | placement_fields >> 21
                                | placement_fields >> 22
                                | placement_fields >> 43)
                    }
                };
                while destinations.not_zero() {
                    let to = destinations.trailing_zeros();
                    destinations.flip_bit(to);
                    action_list.push(Action::Set(to, PieceType::ZTetromino, shape_index));
                }
            }
        }

        if self.pieces_left[PieceType::LTetromino as usize][self.current_player as usize] {
            for shape_index in 15..23 {
                let mut destinations = match shape_index {
                    15 => {
                        (three_down & two_right)
                            & (placement_fields | placement_fields >> 1 | placement_fields >> 42)
                    }
                    16 => {
                        (two_right & three_down >> 1)
                            & (placement_fields | placement_fields >> 1 | placement_fields >> 43)
                    }
                    17 => {
                        ((three_down & two_right >> 41)
                            & (placement_fields | placement_fields >> 41 | placement_fields >> 42))
                            >> 1
                    }
                    18 => {
                        (three_down & two_right >> 42)
                            & (placement_fields | placement_fields >> 42 | placement_fields >> 43)
                    }
                    19 => {
                        (legal_fields & three_right >> 21)
                            & (placement_fields | placement_fields >> 21 | placement_fields >> 23)
                    }
                    20 => {
                        (three_right & legal_fields >> 21)
                            & (placement_fields | placement_fields >> 2 | placement_fields >> 21)
                    }
                    21 => {
                        (three_right & legal_fields >> 23)
                            & (placement_fields | placement_fields >> 2 | placement_fields >> 23)
                    }
                    _ => {
                        (two_up & three_left) >> 23
                            & (placement_fields >> 2
                                | placement_fields >> 21
                                | placement_fields >> 23)
                    }
                };
                while destinations.not_zero() {
                    let to = destinations.trailing_zeros();
                    destinations.flip_bit(to);
                    action_list.push(Action::Set(to, PieceType::LTetromino, shape_index));
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
                if let Action::Set(_, piece_type, _) = action_list[i] {
                    if piece_type == self.start_piece_type {
                        action_list.swap(idx, i);
                        idx += 1;
                    }
                }
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
            if self.monomino_placed_last[Color::YELLOW as usize] {
                yellow_score += 20;
            } else {
                yellow_score += 15;
            }
        }
        if red_score == 89 {
            if self.monomino_placed_last[Color::RED as usize] {
                red_score += 20;
            } else {
                red_score += 15;
            }
        }
        if green_score == 89 {
            if self.monomino_placed_last[Color::GREEN as usize] {
                green_score += 20;
            } else {
                green_score += 15;
            }
        }

        blue_score + red_score - yellow_score - green_score
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
        for (start_piece_index, piece) in PIECE_TYPES.iter().enumerate() {
            if *piece == self.start_piece_type {
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
        let mut entries: Vec<&str> = string.split(' ').collect();
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
        //state.hash = calculate_hash(&state);
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
            string.push(' ');
        }
        string.push_str("║\n");

        string.push_str("╠");
        for _ in 0..40 {
            string.push_str("═");
        }
        string.push_str("╣");

        for y in 0..20 {
            string.push_str("\n║");
            for x in 0..20 {
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

impl Default for GameState {
    fn default() -> GameState {
        Self::new()
    }
}
