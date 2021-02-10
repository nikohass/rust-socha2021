use super::{
    Action, ActionList, Bitboard, Color, PieceType, FIELD_HASH, PENTOMINO_SHAPES, PIECE_HASH,
    PIECE_TYPES, PLY_HASH, START_FIELDS, VALID_FIELDS,
};
use rand::{rngs::SmallRng, RngCore};
use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Eq, PartialEq)]
pub struct GameState {
    pub ply: u8,
    pub board: [Bitboard; 4],
    pub current_color: Color,
    pub pieces_left: [[bool; 4]; 21],
    pub monomino_placed_last: u8,
    pub skipped: u64,
    pub start_piece_type: PieceType,
    pub hash: u64,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            ply: 0,
            board: [Bitboard::empty(); 4],
            current_color: Color::BLUE,
            pieces_left: [[true; 4]; 21],
            monomino_placed_last: 0,
            skipped: 0,
            start_piece_type: PieceType::random_pentomino(),
            hash: 0,
        }
    }

    pub fn do_action(&mut self, action: Action) {
        debug_assert!(self.validate_action(&action), "Action is invalid");
        self.hash ^= PLY_HASH[self.ply as usize];
        match action {
            Action::Skip => {
                self.skipped =
                    ((self.skipped & 0b1111) | self.skipped << 4) | (1 << self.current_color as u8);
            }
            Action::Set(to, shape_index) => {
                let piece_type = PieceType::from_shape_index(shape_index);
                self.hash ^= PIECE_HASH[shape_index][self.current_color as usize];
                self.hash ^= FIELD_HASH[to as usize][self.current_color as usize];
                let piece = Bitboard::with_piece(to, shape_index);
                debug_assert!(
                    !((self.board[0] | self.board[1] | self.board[2] | self.board[3]) & piece)
                        .not_zero(),
                    "Piece can't be placed on other pieces. Action was {}",
                    action.visualize(),
                );
                self.pieces_left[piece_type as usize][self.current_color as usize] = false;
                self.board[self.current_color as usize] ^= piece;
                if piece_type == PieceType::Monomino {
                    self.monomino_placed_last |= 1 << self.current_color as usize;
                } else {
                    self.monomino_placed_last &= !(1 << self.current_color as usize);
                }
            }
        };
        self.current_color = self.current_color.next();
        self.ply += 1;
        debug_assert!(self.check_integrity());
    }

    pub fn undo_action(&mut self, action: Action) {
        self.current_color = self.current_color.previous();
        self.ply -= 1;
        self.hash ^= PLY_HASH[self.ply as usize];
        self.skipped >>= 4;
        if let Action::Set(to, shape_index) = action {
            let piece_type = PieceType::from_shape_index(shape_index);
            self.hash ^= PIECE_HASH[shape_index][self.current_color as usize];
            self.hash ^= FIELD_HASH[to as usize][self.current_color as usize];
            let piece = Bitboard::with_piece(to, shape_index);
            debug_assert!(
                !self.pieces_left[piece_type as usize][self.current_color as usize],
                "Can't remove piece that has not been placed."
            );
            self.pieces_left[piece_type as usize][self.current_color as usize] = true;
            self.board[self.current_color as usize] ^= piece;
        }
        debug_assert!(self.check_integrity());
    }

    pub fn validate_action(&self, action: &Action) -> bool {
        match action {
            Action::Skip => true,
            Action::Set(to, shape_index) => {
                let piece_type = PieceType::from_shape_index(*shape_index);
                let mut is_valid = true;
                if !self.pieces_left[piece_type as usize][self.current_color as usize] {
                    println!("Can't place piece that has already been placed.");
                    return false;
                }
                let piece = Bitboard::with_piece(*to, *shape_index);
                let own_fields = self.board[self.current_color as usize];
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
                    println!("Piece shifted to invalid position");
                    is_valid = false;
                }
                if !is_valid {
                    println!("{}", action.to_string());
                    println!("{}", piece.to_string());
                    println!("{}", action.visualize());
                    println!("{}", self);
                }
                is_valid
            }
        }
    }

    pub fn check_integrity(&self) -> bool {
        for color in 0..4 {
            if self.ply % 4 == color && self.current_color as u8 != color {
                return false;
            }
        }

        for color in 0..4 {
            let pieces = self.board[color].get_pieces();
            let mut pieces_left: [bool; 21] = [true; 21];
            for piece in pieces.iter() {
                if let Action::Set(_, shape_index) = piece {
                    let piece_type = PieceType::from_shape_index(*shape_index);
                    pieces_left[piece_type as usize] = false;
                }
            }
            for piece_type in PIECE_TYPES.iter() {
                if self.pieces_left[*piece_type as usize][color]
                    != pieces_left[*piece_type as usize]
                {
                    return false;
                }
            }
        }

        for color in 0..4 {
            let mut should_have: u32 = 0;
            for piece_type in PIECE_TYPES.iter() {
                if !self.pieces_left[*piece_type as usize][color] {
                    should_have += piece_type.piece_size() as u32;
                }
            }
            if should_have != self.board[color].count_ones() {
                return false;
            }
        }
        true
    }

    pub fn get_possible_actions(&self, action_list: &mut ActionList) {
        action_list.clear();
        if self.skipped & 1 << self.current_color as u8 != 0 {
            action_list.push(Action::Skip);
            return;
        }
        // fields of the current color
        let own_fields = self.board[self.current_color as usize];
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

        let three_right = two_right & (legal_fields >> 2 & VALID_FIELDS);
        let three_left = two_left & (legal_fields << 2 & VALID_FIELDS);
        let three_down = two_down & (legal_fields >> 42 & VALID_FIELDS);
        let three_up = two_up & (legal_fields << 42 & VALID_FIELDS);

        let four_right = three_right & (legal_fields >> 3 & VALID_FIELDS);
        let four_left = three_left & (legal_fields << 3 & VALID_FIELDS);
        let four_down = three_down & (legal_fields >> 63 & VALID_FIELDS);
        let four_up = three_up & (legal_fields << 63 & VALID_FIELDS);

        if self.pieces_left[PieceType::Domino as usize][self.current_color as usize] {
            action_list.append_actions(
                &mut ((two_right & placement_fields) | (two_left & placement_fields) >> 1),
                1,
            );
            action_list.append_actions(
                &mut ((two_down & placement_fields) | (two_up & placement_fields) >> 21),
                2,
            );
        }

        if self.pieces_left[PieceType::ITromino as usize][self.current_color as usize] {
            action_list.append_actions(
                &mut ((three_right & placement_fields) | (three_left & placement_fields) >> 2),
                3,
            );
            action_list.append_actions(
                &mut ((three_up & placement_fields) >> 42 | (three_down & placement_fields)),
                4,
            );
        }

        if self.pieces_left[PieceType::ITetromino as usize][self.current_color as usize] {
            action_list.append_actions(
                &mut ((four_right & placement_fields) | (four_left & placement_fields) >> 3),
                5,
            );
            action_list.append_actions(
                &mut ((four_down & placement_fields) | (four_up & placement_fields) >> 63),
                6,
            );
        }

        if self.pieces_left[PieceType::IPentomino as usize][self.current_color as usize] {
            action_list.append_actions(
                &mut ((four_right & legal_fields >> 4 & placement_fields)
                    | (four_left & legal_fields << 4 & placement_fields) >> 4),
                7,
            );
            action_list.append_actions(
                &mut ((four_down & legal_fields >> 84 & placement_fields)
                    | (four_up & legal_fields << 84 & placement_fields) >> 84),
                8,
            );
        }

        if self.pieces_left[PieceType::XPentomino as usize][self.current_color as usize] {
            action_list.append_actions(
                &mut (((three_right >> 20 & three_down)
                    & (placement_fields
                        | placement_fields >> 20
                        | placement_fields >> 22
                        | placement_fields >> 42))
                    >> 1),
                10,
            )
        }

        if self.pieces_left[PieceType::LTromino as usize][self.current_color as usize] {
            action_list.append_actions(
                &mut ((two_up & two_right) >> 21
                    & (placement_fields | placement_fields >> 21 | placement_fields >> 22)),
                11,
            );
            action_list.append_actions(
                &mut ((two_down & two_right)
                    & (placement_fields | placement_fields >> 1 | placement_fields >> 21)),
                12,
            );
            action_list.append_actions(
                &mut ((two_down >> 1 & two_right)
                    & (placement_fields | placement_fields >> 1 | placement_fields >> 22)),
                13,
            );
            action_list.append_actions(
                &mut ((two_down >> 1 & two_right >> 21)
                    & (placement_fields >> 1 | placement_fields >> 21 | placement_fields >> 22)),
                14,
            );
        }

        if self.pieces_left[PieceType::LPentomino as usize][self.current_color as usize] {
            action_list.append_actions(
                &mut ((four_right & legal_fields >> 24)
                    & (placement_fields | placement_fields >> 3 | placement_fields >> 24)),
                23,
            );
            action_list.append_actions(
                &mut ((four_right & two_down)
                    & (placement_fields | placement_fields >> 3 | placement_fields >> 21)),
                24,
            );
            action_list.append_actions(
                &mut ((legal_fields & four_right >> 21)
                    & (placement_fields | placement_fields >> 21 | placement_fields >> 24)),
                25,
            );
            action_list.append_actions(
                &mut ((four_left & two_up) >> 24
                    & (placement_fields >> 3 | placement_fields >> 21 | placement_fields >> 24)),
                26,
            );
            action_list.append_actions(
                &mut ((two_right & four_down)
                    & (placement_fields | placement_fields >> 1 | placement_fields >> 63)),
                27,
            );
            action_list.append_actions(
                &mut ((four_down & legal_fields >> 64)
                    & (placement_fields | placement_fields >> 63 | placement_fields >> 64)),
                28,
            );
            action_list.append_actions(
                &mut ((two_right & four_down >> 1)
                    & (placement_fields | placement_fields >> 1 | placement_fields >> 64)),
                29,
            );
            action_list.append_actions(
                &mut ((four_up & two_left) >> 64
                    & (placement_fields >> 1 | placement_fields >> 63 | placement_fields >> 64)),
                30,
            );
        }

        if self.pieces_left[PieceType::TPentomino as usize][self.current_color as usize] {
            action_list.append_actions(
                &mut ((three_right & three_down >> 1)
                    & (placement_fields | placement_fields >> 2 | placement_fields >> 43)),
                31,
            );
            action_list.append_actions(
                &mut ((two_left & two_right & three_up) >> 43
                    & (placement_fields >> 1 | placement_fields >> 42 | placement_fields >> 44)),
                32,
            );
            action_list.append_actions(
                &mut ((three_down & three_right >> 21)
                    & (placement_fields | placement_fields >> 23 | placement_fields >> 42)),
                33,
            );
            action_list.append_actions(
                &mut ((three_left & two_up & two_down) >> 23
                    & (placement_fields >> 2 | placement_fields >> 21 | placement_fields >> 44)),
                34,
            );
        }

        if self.pieces_left[PieceType::ZPentomino as usize][self.current_color as usize] {
            action_list.append_actions(
                &mut ((legal_fields & (three_left & two_down) >> 23)
                    & (placement_fields
                        | placement_fields >> 21
                        | placement_fields >> 23
                        | placement_fields >> 44)),
                43,
            );
            action_list.append_actions(
                &mut (((legal_fields & (three_right & two_down) >> 19)
                    & (placement_fields
                        | placement_fields >> 19
                        | placement_fields >> 21
                        | placement_fields >> 40))
                    >> 2),
                44,
            );
            action_list.append_actions(
                &mut ((legal_fields >> 2 & (three_up & two_left) >> 43)
                    & (placement_fields >> 1
                        | placement_fields >> 2
                        | placement_fields >> 42
                        | placement_fields >> 43)),
                45,
            );
            action_list.append_actions(
                &mut ((two_right & (two_right & three_up) >> 43)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 43
                        | placement_fields >> 44)),
                46,
            );
        }

        if self.pieces_left[PieceType::UPentomino as usize][self.current_color as usize] {
            action_list.append_actions(
                &mut ((three_right & two_down & legal_fields >> 23)
                    & (placement_fields
                        | placement_fields >> 2
                        | placement_fields >> 21
                        | placement_fields >> 23)),
                47,
            );
            action_list.append_actions(
                &mut ((legal_fields & (three_left & two_up) >> 23)
                    & (placement_fields
                        | placement_fields >> 2
                        | placement_fields >> 21
                        | placement_fields >> 23)),
                48,
            );
            action_list.append_actions(
                &mut ((three_down & two_right & legal_fields >> 43)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 42
                        | placement_fields >> 43)),
                49,
            );
            action_list.append_actions(
                &mut ((two_right & (two_left & three_up) >> 43)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 42
                        | placement_fields >> 43)),
                50,
            );
        }

        if self.pieces_left[PieceType::FPentomino as usize][self.current_color as usize] {
            action_list.append_actions(
                &mut (((three_up & two_left) >> 43 & legal_fields >> 23)
                    & (placement_fields >> 1
                        | placement_fields >> 23
                        | placement_fields >> 42
                        | placement_fields >> 43)),
                51,
            );
            action_list.append_actions(
                &mut ((legal_fields >> 21 & (three_up & two_right) >> 43)
                    & (placement_fields >> 1
                        | placement_fields >> 21
                        | placement_fields >> 43
                        | placement_fields >> 44)),
                52,
            );
            action_list.append_actions(
                &mut ((((three_down & two_right) & legal_fields >> 20)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 20
                        | placement_fields >> 42))
                    >> 1),
                53,
            );
            action_list.append_actions(
                &mut (((three_down & two_left) >> 1 & legal_fields >> 23)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 23
                        | placement_fields >> 43)),
                54,
            );
            action_list.append_actions(
                &mut (((three_left & two_up) >> 23 & legal_fields >> 43)
                    & (placement_fields >> 2
                        | placement_fields >> 21
                        | placement_fields >> 23
                        | placement_fields >> 43)),
                55,
            );
            action_list.append_actions(
                &mut (((three_right & two_up) >> 21 & legal_fields >> 43)
                    & (placement_fields
                        | placement_fields >> 21
                        | placement_fields >> 23
                        | placement_fields >> 43)),
                56,
            );
            action_list.append_actions(
                &mut (((legal_fields & (three_left & two_down) >> 22)
                    & (placement_fields
                        | placement_fields >> 20
                        | placement_fields >> 22
                        | placement_fields >> 43))
                    >> 1),
                57,
            );
            action_list.append_actions(
                &mut (((legal_fields & (three_right & two_down) >> 20)
                    & (placement_fields
                        | placement_fields >> 20
                        | placement_fields >> 22
                        | placement_fields >> 41))
                    >> 1),
                58,
            );
        }

        if self.pieces_left[PieceType::WPentomino as usize][self.current_color as usize] {
            action_list.append_actions(
                &mut ((two_down & (two_up & two_right) >> 43)
                    & (placement_fields
                        | placement_fields >> 21
                        | placement_fields >> 22
                        | placement_fields >> 43
                        | placement_fields >> 44)),
                59,
            );
            action_list.append_actions(
                &mut (((two_up & two_left) >> 23 & two_right >> 42)
                    & (placement_fields >> 2
                        | placement_fields >> 22
                        | placement_fields >> 23
                        | placement_fields >> 42
                        | placement_fields >> 43)),
                60,
            );
            action_list.append_actions(
                &mut ((two_right & (two_down & two_left) >> 23)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 22
                        | placement_fields >> 23
                        | placement_fields >> 44)),
                61,
            );
            action_list.append_actions(
                &mut (((two_right & (two_right & two_down) >> 20)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 20
                        | placement_fields >> 21
                        | placement_fields >> 41))
                    >> 1),
                62,
            );
        }

        if self.pieces_left[PieceType::NPentomino as usize][self.current_color as usize] {
            action_list.append_actions(
                &mut (((three_down & two_down >> 41)
                    & (placement_fields
                        | placement_fields >> 41
                        | placement_fields >> 42
                        | placement_fields >> 62))
                    >> 1),
                63,
            );
            action_list.append_actions(
                &mut ((three_down & two_down >> 43)
                    & (placement_fields
                        | placement_fields >> 42
                        | placement_fields >> 43
                        | placement_fields >> 64)),
                64,
            );
            action_list.append_actions(
                &mut (((two_down & three_down >> 20)
                    & (placement_fields
                        | placement_fields >> 20
                        | placement_fields >> 21
                        | placement_fields >> 62))
                    >> 1),
                65,
            );
            action_list.append_actions(
                &mut ((two_down & three_down >> 22)
                    & (placement_fields
                        | placement_fields >> 21
                        | placement_fields >> 22
                        | placement_fields >> 64)),
                66,
            );
            action_list.append_actions(
                &mut (((two_right & three_right >> 19)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 19
                        | placement_fields >> 21))
                    >> 2),
                67,
            );
            action_list.append_actions(
                &mut ((three_right & two_right >> 23)
                    & (placement_fields
                        | placement_fields >> 2
                        | placement_fields >> 23
                        | placement_fields >> 24)),
                68,
            );
            action_list.append_actions(
                &mut ((two_right & three_right >> 22)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 22
                        | placement_fields >> 24)),
                69,
            );
            action_list.append_actions(
                &mut (((three_right & two_right >> 20)
                    & (placement_fields
                        | placement_fields >> 2
                        | placement_fields >> 20
                        | placement_fields >> 21))
                    >> 1),
                70,
            );
        }

        if self.pieces_left[PieceType::VPentomino as usize][self.current_color as usize] {
            action_list.append_actions(
                &mut ((three_right & three_down)
                    & (placement_fields | placement_fields >> 2 | placement_fields >> 42)),
                71,
            );
            action_list.append_actions(
                &mut ((three_up & three_left) >> 44
                    & (placement_fields >> 2 | placement_fields >> 42 | placement_fields >> 44)),
                72,
            );
            action_list.append_actions(
                &mut ((three_right & three_down >> 2)
                    & (placement_fields | placement_fields >> 2 | placement_fields >> 44)),
                73,
            );
            action_list.append_actions(
                &mut ((three_down & three_right >> 42)
                    & (placement_fields | placement_fields >> 42 | placement_fields >> 44)),
                74,
            );
        }

        if self.pieces_left[PieceType::YPentomino as usize][self.current_color as usize] {
            action_list.append_actions(
                &mut ((four_down & legal_fields >> 22)
                    & (placement_fields | placement_fields >> 22 | placement_fields >> 63)),
                83,
            );
            action_list.append_actions(
                &mut ((four_down & legal_fields >> 43)
                    & (placement_fields | placement_fields >> 43 | placement_fields >> 63)),
                84,
            );
            action_list.append_actions(
                &mut (((four_down & legal_fields >> 41)
                    & (placement_fields | placement_fields >> 41 | placement_fields >> 63))
                    >> 1),
                85,
            );
            action_list.append_actions(
                &mut (((four_down & legal_fields >> 20)
                    & (placement_fields | placement_fields >> 20 | placement_fields >> 63))
                    >> 1),
                86,
            );
            action_list.append_actions(
                &mut ((four_right & legal_fields >> 23)
                    & (placement_fields | placement_fields >> 3 | placement_fields >> 23)),
                87,
            );
            action_list.append_actions(
                &mut ((four_right & legal_fields >> 22)
                    & (placement_fields | placement_fields >> 3 | placement_fields >> 22)),
                88,
            );
            action_list.append_actions(
                &mut ((two_up & two_right & three_left) >> 23
                    & (placement_fields >> 2 | placement_fields >> 21 | placement_fields >> 24)),
                89,
            );
            action_list.append_actions(
                &mut (((legal_fields & four_right >> 20)
                    & (placement_fields | placement_fields >> 20 | placement_fields >> 23))
                    >> 1),
                90,
            );
        }

        if self.pieces_left[PieceType::TTetromino as usize][self.current_color as usize] {
            action_list.append_actions(
                &mut ((three_right & legal_fields >> 22)
                    & (placement_fields | placement_fields >> 2 | placement_fields >> 22)),
                35,
            );
            action_list.append_actions(
                &mut ((two_up & two_right & two_left) >> 22
                    & (placement_fields >> 1 | placement_fields >> 21 | placement_fields >> 23)),
                36,
            );
            action_list.append_actions(
                &mut ((three_down & legal_fields >> 22)
                    & (placement_fields | placement_fields >> 22 | placement_fields >> 42)),
                37,
            );
            action_list.append_actions(
                &mut ((two_up & two_down & two_left) >> 22
                    & (placement_fields >> 1 | placement_fields >> 21 | placement_fields >> 43)),
                38,
            );
        }

        {
            let square = two_right & two_right >> 21;
            if self.pieces_left[PieceType::OTetromino as usize][self.current_color as usize] {
                action_list.append_actions(
                    &mut (square
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 21
                            | placement_fields >> 22)),
                    9,
                )
            }

            if self.pieces_left[PieceType::PPentomino as usize][self.current_color as usize] {
                action_list.append_actions(
                    &mut ((square & legal_fields >> 42)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 22
                            | placement_fields >> 42)),
                    75,
                );
                action_list.append_actions(
                    &mut ((square & legal_fields >> 43)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 21
                            | placement_fields >> 43)),
                    76,
                );
                action_list.append_actions(
                    &mut ((square & legal_fields >> 23)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 21
                            | placement_fields >> 23)),
                    77,
                );
                action_list.append_actions(
                    &mut ((square & legal_fields >> 2)
                        & (placement_fields
                            | placement_fields >> 2
                            | placement_fields >> 21
                            | placement_fields >> 22)),
                    78,
                );
                action_list.append_actions(
                    &mut ((square >> 1 & legal_fields)
                        & (placement_fields
                            | placement_fields >> 2
                            | placement_fields >> 22
                            | placement_fields >> 23)),
                    79,
                );
                action_list.append_actions(
                    &mut (((square & legal_fields >> 20)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 20
                            | placement_fields >> 22))
                        >> 1),
                    80,
                );
                action_list.append_actions(
                    &mut (((legal_fields & square >> 20)
                        & (placement_fields
                            | placement_fields >> 20
                            | placement_fields >> 41
                            | placement_fields >> 42))
                        >> 1),
                    81,
                );
                action_list.append_actions(
                    &mut ((legal_fields & square >> 21)
                        & (placement_fields
                            | placement_fields >> 22
                            | placement_fields >> 42
                            | placement_fields >> 43)),
                    82,
                );
            }
        }

        if self.pieces_left[PieceType::ZTetromino as usize][self.current_color as usize] {
            action_list.append_actions(
                &mut (((two_right & two_right >> 20)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 20
                        | placement_fields >> 21))
                    >> 1),
                39,
            );
            action_list.append_actions(
                &mut ((two_right & two_right >> 22)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 22
                        | placement_fields >> 23)),
                40,
            );
            action_list.append_actions(
                &mut (((two_down & two_down >> 20)
                    & (placement_fields
                        | placement_fields >> 20
                        | placement_fields >> 21
                        | placement_fields >> 41))
                    >> 1),
                41,
            );
            action_list.append_actions(
                &mut ((two_down & two_down >> 22)
                    & (placement_fields
                        | placement_fields >> 21
                        | placement_fields >> 22
                        | placement_fields >> 43)),
                42,
            );
        }

        if self.pieces_left[PieceType::LTetromino as usize][self.current_color as usize] {
            action_list.append_actions(
                &mut ((three_down & two_right)
                    & (placement_fields | placement_fields >> 1 | placement_fields >> 42)),
                15,
            );
            action_list.append_actions(
                &mut ((two_right & three_down >> 1)
                    & (placement_fields | placement_fields >> 1 | placement_fields >> 43)),
                16,
            );
            action_list.append_actions(
                &mut (((three_down & two_right >> 41)
                    & (placement_fields | placement_fields >> 41 | placement_fields >> 42))
                    >> 1),
                17,
            );
            action_list.append_actions(
                &mut ((three_down & two_right >> 42)
                    & (placement_fields | placement_fields >> 42 | placement_fields >> 43)),
                18,
            );
            action_list.append_actions(
                &mut ((legal_fields & three_right >> 21)
                    & (placement_fields | placement_fields >> 21 | placement_fields >> 23)),
                19,
            );
            action_list.append_actions(
                &mut ((three_right & legal_fields >> 21)
                    & (placement_fields | placement_fields >> 2 | placement_fields >> 21)),
                20,
            );
            action_list.append_actions(
                &mut ((three_right & legal_fields >> 23)
                    & (placement_fields | placement_fields >> 2 | placement_fields >> 23)),
                21,
            );
            action_list.append_actions(
                &mut ((two_up & three_left) >> 23
                    & (placement_fields >> 2 | placement_fields >> 21 | placement_fields >> 23)),
                22,
            );
        }

        if self.pieces_left[PieceType::Monomino as usize][self.current_color as usize] {
            action_list.append_actions(&mut placement_fields, 0);
        }

        if self.ply < 4 {
            let mut idx = 0;
            for i in 0..action_list.size {
                if let Action::Set(_, shape_index) = action_list[i] {
                    let piece_type = PieceType::from_shape_index(shape_index);
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

    pub fn get_random_possible_action(
        &self,
        rng: &mut SmallRng,
        pentomino_only: bool,
        tries: usize,
    ) -> Action {
        let own_fields = self.board[self.current_color as usize];
        let other_fields =
            (self.board[0] | self.board[1] | self.board[2] | self.board[3]) & !own_fields;
        // all fields that are empty and aren't next to own fields
        let legal_fields = !(own_fields | other_fields | own_fields.neighbours()) & VALID_FIELDS;
        // every placed piece has to touch at least one of these fields
        let placement_fields = if self.ply > 3 {
            own_fields.diagonal_neighbours() & legal_fields
        } else {
            START_FIELDS & !other_fields
        };

        if placement_fields.is_zero() {
            return Action::Skip;
        }

        let two_right = legal_fields & (legal_fields >> 1 & VALID_FIELDS);
        let two_left = legal_fields & (legal_fields << 1 & VALID_FIELDS);
        let two_down = legal_fields & (legal_fields >> 21 & VALID_FIELDS);
        let two_up = legal_fields & (legal_fields << 21 & VALID_FIELDS);

        let three_right = two_right & (legal_fields >> 2 & VALID_FIELDS);
        let three_left = two_left & (legal_fields << 2 & VALID_FIELDS);
        let three_down = two_down & (legal_fields >> 42 & VALID_FIELDS);
        let three_up = two_up & (legal_fields << 42 & VALID_FIELDS);

        let four_right = three_right & (legal_fields >> 3 & VALID_FIELDS);
        let four_left = three_left & (legal_fields << 3 & VALID_FIELDS);
        let four_down = three_down & (legal_fields >> 63 & VALID_FIELDS);
        let four_up = three_up & (legal_fields << 63 & VALID_FIELDS);

        let square = two_right & two_right >> 21;

        for _ in 0..tries {
            let to = random_field(&mut placement_fields.clone(), rng); // select a random placement field
            let mut shape_index = if pentomino_only {
                PENTOMINO_SHAPES[(rng.next_u64() % 63) as usize]
            } else {
                (rng.next_u32() % 91) as usize
            };
            if self.ply < 4 {
                while PieceType::from_shape_index(shape_index as usize) != self.start_piece_type {
                    shape_index = PENTOMINO_SHAPES[(rng.next_u64() % 63) as usize]
                }
            }
            if !self.pieces_left[PieceType::from_shape_index(shape_index as usize) as usize]
                [self.current_color as usize]
            {
                continue;
            }
            match shape_index {
                0 => {
                    return Action::Set(to, 0);
                }
                1 => {
                    let mut destinations =
                        (two_right & placement_fields) | (two_left & placement_fields) >> 1;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 1);
                    }
                }
                2 => {
                    let mut destinations =
                        (two_down & placement_fields) | (two_up & placement_fields) >> 21;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 2);
                    }
                }
                3 => {
                    let mut destinations =
                        (three_right & placement_fields) | (three_left & placement_fields) >> 2;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 3);
                    }
                }
                4 => {
                    let mut destinations =
                        (three_up & placement_fields) >> 42 | (three_down & placement_fields);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 4);
                    }
                }
                5 => {
                    let mut destinations =
                        (four_right & placement_fields) | (four_left & placement_fields) >> 3;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 5);
                    }
                }
                6 => {
                    let mut destinations =
                        (four_down & placement_fields) | (four_up & placement_fields) >> 63;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 6);
                    }
                }
                7 => {
                    let mut destinations = (four_right & legal_fields >> 4 & placement_fields)
                        | (four_left & legal_fields << 4 & placement_fields) >> 4;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 7);
                    }
                }
                8 => {
                    let mut destinations = (four_down & legal_fields >> 84 & placement_fields)
                        | (four_up & legal_fields << 84 & placement_fields) >> 84;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 8);
                    }
                }
                9 => {
                    let mut destinations = (two_right & two_right >> 21)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 21
                            | placement_fields >> 22);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 9);
                    }
                }
                10 => {
                    let mut destinations = ((three_right >> 20 & three_down)
                        & (placement_fields
                            | placement_fields >> 20
                            | placement_fields >> 22
                            | placement_fields >> 42))
                        >> 1;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 10);
                    }
                }
                11 => {
                    let mut destinations = (two_up & two_right) >> 21
                        & (placement_fields | placement_fields >> 21 | placement_fields >> 22);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 11);
                    }
                }
                12 => {
                    let mut destinations = (two_down & two_right)
                        & (placement_fields | placement_fields >> 1 | placement_fields >> 21);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 12);
                    }
                }
                13 => {
                    let mut destinations = (two_down >> 1 & two_right)
                        & (placement_fields | placement_fields >> 1 | placement_fields >> 22);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 13);
                    }
                }
                14 => {
                    let mut destinations = (two_down >> 1 & two_right >> 21)
                        & (placement_fields >> 1 | placement_fields >> 21 | placement_fields >> 22);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 14);
                    }
                }
                15 => {
                    let mut destinations = (three_down & two_right)
                        & (placement_fields | placement_fields >> 1 | placement_fields >> 42);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 15);
                    }
                }
                16 => {
                    let mut destinations = (two_right & three_down >> 1)
                        & (placement_fields | placement_fields >> 1 | placement_fields >> 43);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 16);
                    }
                }
                17 => {
                    let mut destinations = ((three_down & two_right >> 41)
                        & (placement_fields | placement_fields >> 41 | placement_fields >> 42))
                        >> 1;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 17);
                    }
                }
                18 => {
                    let mut destinations = (three_down & two_right >> 42)
                        & (placement_fields | placement_fields >> 42 | placement_fields >> 43);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 18);
                    }
                }
                19 => {
                    let mut destinations = (legal_fields & three_right >> 21)
                        & (placement_fields | placement_fields >> 21 | placement_fields >> 23);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 19);
                    }
                }
                20 => {
                    let mut destinations = (three_right & legal_fields >> 21)
                        & (placement_fields | placement_fields >> 2 | placement_fields >> 21);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 20);
                    }
                }
                21 => {
                    let mut destinations = (three_right & legal_fields >> 23)
                        & (placement_fields | placement_fields >> 2 | placement_fields >> 23);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 21);
                    }
                }
                22 => {
                    let mut destinations = (two_up & three_left) >> 23
                        & (placement_fields >> 2 | placement_fields >> 21 | placement_fields >> 23);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 22);
                    }
                }
                23 => {
                    let mut destinations = (four_right & legal_fields >> 24)
                        & (placement_fields | placement_fields >> 3 | placement_fields >> 24);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 23);
                    }
                }
                24 => {
                    let mut destinations = (four_right & two_down)
                        & (placement_fields | placement_fields >> 3 | placement_fields >> 21);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 24);
                    }
                }
                25 => {
                    let mut destinations = (legal_fields & four_right >> 21)
                        & (placement_fields | placement_fields >> 21 | placement_fields >> 24);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 25);
                    }
                }
                26 => {
                    let mut destinations = (four_left & two_up) >> 24
                        & (placement_fields >> 3 | placement_fields >> 21 | placement_fields >> 24);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 26);
                    }
                }
                27 => {
                    let mut destinations = (two_right & four_down)
                        & (placement_fields | placement_fields >> 1 | placement_fields >> 63);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 27);
                    }
                }
                28 => {
                    let mut destinations = (four_down & legal_fields >> 64)
                        & (placement_fields | placement_fields >> 63 | placement_fields >> 64);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 28);
                    }
                }
                29 => {
                    let mut destinations = (two_right & four_down >> 1)
                        & (placement_fields | placement_fields >> 1 | placement_fields >> 64);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 29);
                    }
                }
                30 => {
                    let mut destinations = (four_up & two_left) >> 64
                        & (placement_fields >> 1 | placement_fields >> 63 | placement_fields >> 64);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 30);
                    }
                }
                31 => {
                    let mut destinations = (three_right & three_down >> 1)
                        & (placement_fields | placement_fields >> 2 | placement_fields >> 43);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 31);
                    }
                }
                32 => {
                    let mut destinations = (two_left & two_right & three_up) >> 43
                        & (placement_fields >> 1 | placement_fields >> 42 | placement_fields >> 44);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 32);
                    }
                }
                33 => {
                    let mut destinations = (three_down & three_right >> 21)
                        & (placement_fields | placement_fields >> 23 | placement_fields >> 42);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 33);
                    }
                }
                34 => {
                    let mut destinations = (three_left & two_up & two_down) >> 23
                        & (placement_fields >> 2 | placement_fields >> 21 | placement_fields >> 44);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 34);
                    }
                }
                35 => {
                    let mut destinations = (three_right & legal_fields >> 22)
                        & (placement_fields | placement_fields >> 2 | placement_fields >> 22);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 35);
                    }
                }
                36 => {
                    let mut destinations = (two_up & two_right & two_left) >> 22
                        & (placement_fields >> 1 | placement_fields >> 21 | placement_fields >> 23);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 36);
                    }
                }
                37 => {
                    let mut destinations = (three_down & legal_fields >> 22)
                        & (placement_fields | placement_fields >> 22 | placement_fields >> 42);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 37);
                    }
                }
                38 => {
                    let mut destinations = (two_up & two_down & two_left) >> 22
                        & (placement_fields >> 1 | placement_fields >> 21 | placement_fields >> 43);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 38);
                    }
                }
                39 => {
                    let mut destinations = ((two_right & two_right >> 20)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 20
                            | placement_fields >> 21))
                        >> 1;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 39);
                    }
                }
                40 => {
                    let mut destinations = (two_right & two_right >> 22)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 22
                            | placement_fields >> 23);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 40);
                    }
                }
                41 => {
                    let mut destinations = ((two_down & two_down >> 20)
                        & (placement_fields
                            | placement_fields >> 20
                            | placement_fields >> 21
                            | placement_fields >> 41))
                        >> 1;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 41);
                    }
                }
                42 => {
                    let mut destinations = (two_down & two_down >> 22)
                        & (placement_fields
                            | placement_fields >> 21
                            | placement_fields >> 22
                            | placement_fields >> 43);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 42);
                    }
                }
                43 => {
                    let mut destinations = (legal_fields & (three_left & two_down) >> 23)
                        & (placement_fields
                            | placement_fields >> 21
                            | placement_fields >> 23
                            | placement_fields >> 44);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 43);
                    }
                }
                44 => {
                    let mut destinations = ((legal_fields & (three_right & two_down) >> 19)
                        & (placement_fields
                            | placement_fields >> 19
                            | placement_fields >> 21
                            | placement_fields >> 40))
                        >> 2;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 44);
                    }
                }
                45 => {
                    let mut destinations = (legal_fields >> 2 & (three_up & two_left) >> 43)
                        & (placement_fields >> 1
                            | placement_fields >> 2
                            | placement_fields >> 42
                            | placement_fields >> 43);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 45);
                    }
                }
                46 => {
                    let mut destinations = (two_right & (two_right & three_up) >> 43)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 43
                            | placement_fields >> 44);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 46);
                    }
                }
                47 => {
                    let mut destinations = (three_right & two_down & legal_fields >> 23)
                        & (placement_fields
                            | placement_fields >> 2
                            | placement_fields >> 21
                            | placement_fields >> 23);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 47);
                    }
                }
                48 => {
                    let mut destinations = (legal_fields & (three_left & two_up) >> 23)
                        & (placement_fields
                            | placement_fields >> 2
                            | placement_fields >> 21
                            | placement_fields >> 23);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 48);
                    }
                }
                49 => {
                    let mut destinations = (three_down & two_right & legal_fields >> 43)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 42
                            | placement_fields >> 43);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 49);
                    }
                }
                50 => {
                    let mut destinations = (two_right & (two_left & three_up) >> 43)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 42
                            | placement_fields >> 43);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 50);
                    }
                }
                51 => {
                    let mut destinations = ((three_up & two_left) >> 43 & legal_fields >> 23)
                        & (placement_fields >> 1
                            | placement_fields >> 23
                            | placement_fields >> 42
                            | placement_fields >> 43);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 51);
                    }
                }
                52 => {
                    let mut destinations = (legal_fields >> 21 & (three_up & two_right) >> 43)
                        & (placement_fields >> 1
                            | placement_fields >> 21
                            | placement_fields >> 43
                            | placement_fields >> 44);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 52);
                    }
                }
                53 => {
                    let mut destinations = (((three_down & two_right) & legal_fields >> 20)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 20
                            | placement_fields >> 42))
                        >> 1;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 53);
                    }
                }
                54 => {
                    let mut destinations = ((three_down & two_left) >> 1 & legal_fields >> 23)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 23
                            | placement_fields >> 43);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 54);
                    }
                }
                55 => {
                    let mut destinations = ((three_left & two_up) >> 23 & legal_fields >> 43)
                        & (placement_fields >> 2
                            | placement_fields >> 21
                            | placement_fields >> 23
                            | placement_fields >> 43);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 55);
                    }
                }
                56 => {
                    let mut destinations = ((three_right & two_up) >> 21 & legal_fields >> 43)
                        & (placement_fields
                            | placement_fields >> 21
                            | placement_fields >> 23
                            | placement_fields >> 43);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 56);
                    }
                }
                57 => {
                    let mut destinations = ((legal_fields & (three_left & two_down) >> 22)
                        & (placement_fields
                            | placement_fields >> 20
                            | placement_fields >> 22
                            | placement_fields >> 43))
                        >> 1;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 57);
                    }
                }
                58 => {
                    let mut destinations = ((legal_fields & (three_right & two_down) >> 20)
                        & (placement_fields
                            | placement_fields >> 20
                            | placement_fields >> 22
                            | placement_fields >> 41))
                        >> 1;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 58);
                    }
                }
                59 => {
                    let mut destinations = (two_down & (two_up & two_right) >> 43)
                        & (placement_fields
                            | placement_fields >> 21
                            | placement_fields >> 22
                            | placement_fields >> 43
                            | placement_fields >> 44);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 59);
                    }
                }
                60 => {
                    let mut destinations = ((two_up & two_left) >> 23 & two_right >> 42)
                        & (placement_fields >> 2
                            | placement_fields >> 22
                            | placement_fields >> 23
                            | placement_fields >> 42
                            | placement_fields >> 43);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 60);
                    }
                }
                61 => {
                    let mut destinations = (two_right & (two_down & two_left) >> 23)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 22
                            | placement_fields >> 23
                            | placement_fields >> 44);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 61);
                    }
                }
                62 => {
                    let mut destinations = ((two_right & (two_right & two_down) >> 20)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 20
                            | placement_fields >> 21
                            | placement_fields >> 41))
                        >> 1;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 62);
                    }
                }
                63 => {
                    let mut destinations = ((three_down & two_down >> 41)
                        & (placement_fields
                            | placement_fields >> 41
                            | placement_fields >> 42
                            | placement_fields >> 62))
                        >> 1;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 63);
                    }
                }
                64 => {
                    let mut destinations = (three_down & two_down >> 43)
                        & (placement_fields
                            | placement_fields >> 42
                            | placement_fields >> 43
                            | placement_fields >> 64);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 64);
                    }
                }
                65 => {
                    let mut destinations = ((two_down & three_down >> 20)
                        & (placement_fields
                            | placement_fields >> 20
                            | placement_fields >> 21
                            | placement_fields >> 62))
                        >> 1;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 65);
                    }
                }
                66 => {
                    let mut destinations = (two_down & three_down >> 22)
                        & (placement_fields
                            | placement_fields >> 21
                            | placement_fields >> 22
                            | placement_fields >> 64);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 66);
                    }
                }
                67 => {
                    let mut destinations = ((two_right & three_right >> 19)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 19
                            | placement_fields >> 21))
                        >> 2;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 67);
                    }
                }
                68 => {
                    let mut destinations = (three_right & two_right >> 23)
                        & (placement_fields
                            | placement_fields >> 2
                            | placement_fields >> 23
                            | placement_fields >> 24);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 68);
                    }
                }
                69 => {
                    let mut destinations = (two_right & three_right >> 22)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 22
                            | placement_fields >> 24);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 69);
                    }
                }
                70 => {
                    let mut destinations = ((three_right & two_right >> 20)
                        & (placement_fields
                            | placement_fields >> 2
                            | placement_fields >> 20
                            | placement_fields >> 21))
                        >> 1;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 70);
                    }
                }
                71 => {
                    let mut destinations = (three_right & three_down)
                        & (placement_fields | placement_fields >> 2 | placement_fields >> 42);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 71);
                    }
                }
                72 => {
                    let mut destinations = (three_up & three_left) >> 44
                        & (placement_fields >> 2 | placement_fields >> 42 | placement_fields >> 44);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 72);
                    }
                }
                73 => {
                    let mut destinations = (three_right & three_down >> 2)
                        & (placement_fields | placement_fields >> 2 | placement_fields >> 44);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 73);
                    }
                }
                74 => {
                    let mut destinations = (three_down & three_right >> 42)
                        & (placement_fields | placement_fields >> 42 | placement_fields >> 44);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 74);
                    }
                }
                75 => {
                    let mut destinations = (square & legal_fields >> 42)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 22
                            | placement_fields >> 42);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 75);
                    }
                }
                76 => {
                    let mut destinations = (square & legal_fields >> 43)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 21
                            | placement_fields >> 43);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 76);
                    }
                }
                77 => {
                    let mut destinations = (square & legal_fields >> 23)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 21
                            | placement_fields >> 23);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 77);
                    }
                }
                78 => {
                    let mut destinations = (square & legal_fields >> 2)
                        & (placement_fields
                            | placement_fields >> 2
                            | placement_fields >> 21
                            | placement_fields >> 22);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 78);
                    }
                }
                79 => {
                    let mut destinations = (square >> 1 & legal_fields)
                        & (placement_fields
                            | placement_fields >> 2
                            | placement_fields >> 22
                            | placement_fields >> 23);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 79);
                    }
                }
                80 => {
                    let mut destinations = ((square & legal_fields >> 20)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 20
                            | placement_fields >> 22))
                        >> 1;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 80);
                    }
                }
                81 => {
                    let mut destinations = ((legal_fields & square >> 20)
                        & (placement_fields
                            | placement_fields >> 20
                            | placement_fields >> 41
                            | placement_fields >> 42))
                        >> 1;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 81);
                    }
                }
                82 => {
                    let mut destinations = (legal_fields & square >> 21)
                        & (placement_fields
                            | placement_fields >> 22
                            | placement_fields >> 42
                            | placement_fields >> 43);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 82);
                    }
                }
                83 => {
                    let mut destinations = (four_down & legal_fields >> 22)
                        & (placement_fields | placement_fields >> 22 | placement_fields >> 63);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 83);
                    }
                }
                84 => {
                    let mut destinations = (four_down & legal_fields >> 43)
                        & (placement_fields | placement_fields >> 43 | placement_fields >> 63);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 84);
                    }
                }
                85 => {
                    let mut destinations = ((four_down & legal_fields >> 41)
                        & (placement_fields | placement_fields >> 41 | placement_fields >> 63))
                        >> 1;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 85);
                    }
                }
                86 => {
                    let mut destinations = ((four_down & legal_fields >> 20)
                        & (placement_fields | placement_fields >> 20 | placement_fields >> 63))
                        >> 1;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 86);
                    }
                }
                87 => {
                    let mut destinations = (four_right & legal_fields >> 23)
                        & (placement_fields | placement_fields >> 3 | placement_fields >> 23);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 87);
                    }
                }
                88 => {
                    let mut destinations = (four_right & legal_fields >> 22)
                        & (placement_fields | placement_fields >> 3 | placement_fields >> 22);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 88);
                    }
                }
                89 => {
                    let mut destinations = (two_up & two_right & three_left) >> 23
                        & (placement_fields >> 2 | placement_fields >> 21 | placement_fields >> 24);
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 89);
                    }
                }
                90 => {
                    let mut destinations = ((legal_fields & four_right >> 20)
                        & (placement_fields | placement_fields >> 20 | placement_fields >> 23))
                        >> 1;
                    if destinations.not_zero() {
                        return Action::Set(random_field(&mut destinations, rng), 90);
                    }
                }
                _ => {}
            }
        }
        Action::Skip
    }

    #[inline(always)]
    pub fn is_game_over(&self) -> bool {
        self.skipped & 0b1111 == 0b1111 || self.ply > 100 // the game is over when all colors skipped or after round 25 / ply 100
    }

    pub fn game_result(&self) -> i16 {
        let mut scores: [i16; 4] = [
            self.board[Color::BLUE as usize].count_ones() as i16,
            self.board[Color::YELLOW as usize].count_ones() as i16,
            self.board[Color::RED as usize].count_ones() as i16,
            self.board[Color::GREEN as usize].count_ones() as i16,
        ];

        for (color, score) in scores.iter_mut().enumerate() {
            if *score == 89 {
                *score += 15;
            }
            if self.monomino_placed_last & (1 << color) != 0 {
                *score += 5;
            }
        }
        scores[0] + scores[2] - scores[1] - scores[3]
    }

    pub fn to_fen(&self) -> String {
        let mut data = self.monomino_placed_last as u128;
        for color in 0..4 {
            for piece_type in 0..21 {
                if self.pieces_left[piece_type as usize][color as usize] {
                    data |= 1 << (piece_type + 21 * color + 4);
                }
            }
        }
        for (start_piece_index, piece) in PIECE_TYPES.iter().enumerate() {
            if *piece == self.start_piece_type {
                data |= (start_piece_index as u128) << 110;
                break;
            }
        }
        format!(
            "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            (self.ply as u128) | (self.skipped as u128) << 8,
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
            data
        )
    }

    pub fn from_fen(string: String) -> GameState {
        let mut entries: Vec<&str> = string.split(' ').collect();
        let mut state = GameState::new();
        let first_entry = entries.remove(0).parse::<u128>().unwrap();
        state.ply = (first_entry & 0b11111111) as u8;
        state.skipped = (first_entry >> 8) as u64;
        state.current_color = match state.ply % 4 {
            0 => Color::BLUE,
            1 => Color::YELLOW,
            2 => Color::RED,
            _ => Color::GREEN,
        };
        for color in 0..4 {
            state.board[color].one = entries.remove(0).parse::<u128>().unwrap();
            state.board[color].two = entries.remove(0).parse::<u128>().unwrap();
            state.board[color].three = entries.remove(0).parse::<u128>().unwrap();
            state.board[color].four = entries.remove(0).parse::<u128>().unwrap();
        }
        let data = entries.remove(0).parse::<u128>().unwrap();
        state.monomino_placed_last = data as u8 & 0b1111;
        for color in 0..4 {
            for piece_type in 0..21 {
                state.pieces_left[piece_type][color] =
                    data & 1 << (piece_type + 21 * color + 4) != 0;
            }
        }
        let start_piece_index = data >> 110 & 31;
        state.start_piece_type = PIECE_TYPES[start_piece_index as usize];
        state
    }
}

pub fn random_field(board: &mut Bitboard, rng: &mut SmallRng) -> u16 {
    let mut n = board.count_ones();
    for _ in 0..n - 1 {
        let bit_index = board.trailing_zeros();
        if rng.next_u32() < std::u32::MAX / n {
            return bit_index;
        }
        board.flip_bit(bit_index);
        n -= 1;
    }
    board.trailing_zeros()
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut string = String::new();

        string.push('╔');
        for _ in 0..40 {
            string.push('═');
        }
        string.push_str("╗\n");

        let info = &format!(
            "║ {} Turn: {} Score: {}",
            self.current_color.to_string(),
            self.ply,
            self.game_result(),
        );
        string.push_str(info);

        for _ in info.len()..43 {
            string.push(' ');
        }
        string.push_str("║\n");

        string.push('╠');
        for _ in 0..40 {
            string.push('═');
        }
        string.push('╣');

        for y in 0..20 {
            string.push_str("\n║");
            for x in 0..20 {
                let field = x + y * 21;
                let bit = Bitboard::bit(field);
                if self.board[0] & bit == bit {
                    string.push('🟦');
                } else if self.board[1] & bit == bit {
                    string.push('🟨');
                } else if self.board[2] & bit == bit {
                    string.push('🟥');
                } else if self.board[3] & bit == bit {
                    string.push('🟩');
                } else {
                    string.push_str("▪️");
                }
            }
            string.push('║');
        }
        string.push_str("\n╚");
        for _ in 0..40 {
            string.push('═');
        }
        string.push('╝');

        write!(f, "{}", string)
    }
}

impl Default for GameState {
    fn default() -> GameState {
        Self::new()
    }
}
