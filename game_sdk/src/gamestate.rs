use super::{
    Action, ActionList, Bitboard, Color, PieceType, FIELD_HASH, PIECE_HASH, PIECE_TYPES, PLY_HASH,
    START_FIELDS, VALID_FIELDS,
};
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
        self.hash ^= PLY_HASH[self.ply as usize];
        match action {
            Action::Skip => {
                self.skipped |= 1 << self.current_player as usize;
            }
            Action::Set(to, piece_type, shape_index) => {
                self.hash ^= PIECE_HASH[shape_index][self.current_player as usize];
                self.hash ^= FIELD_HASH[to as usize][self.current_player as usize];
                let piece = Bitboard::with_piece(to, shape_index);
                self.skipped &= !1 << self.current_player as usize;

                debug_assert!(
                    !((self.board[0] | self.board[1] | self.board[2] | self.board[3]) & piece)
                        .not_zero(),
                    "Piece can't be placed on other pieces. Move was {}\n{}",
                    action.to_string(),
                    Bitboard::with_piece(to, shape_index).to_string(),
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
        self.hash ^= PLY_HASH[self.ply as usize];
        match action {
            Action::Skip => {
                self.skipped &= !1 << self.current_player as usize;
            }
            Action::Set(to, piece_type, shape_index) => {
                self.hash ^= PIECE_HASH[shape_index][self.current_player as usize];
                self.hash ^= FIELD_HASH[to as usize][self.current_player as usize];
                let piece = Bitboard::with_piece(to, shape_index);
                debug_assert!(
                    !self.pieces_left[piece_type as usize][self.current_player as usize],
                    "Cannot remove piece that has not been placed."
                );
                self.pieces_left[piece_type as usize][self.current_player as usize] = true;
                self.board[self.current_player as usize] ^= piece;
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
                    println!("Piece shifted to invalid position");
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
        action_list.size = 0;
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

        let three_right = two_right & (legal_fields >> 2 & VALID_FIELDS);
        let three_left = two_left & (legal_fields << 2 & VALID_FIELDS);
        let three_down = two_down & (legal_fields >> 42 & VALID_FIELDS);
        let three_up = two_up & (legal_fields << 42 & VALID_FIELDS);

        let four_right = three_right & (legal_fields >> 3 & VALID_FIELDS);
        let four_left = three_left & (legal_fields << 3 & VALID_FIELDS);
        let four_down = three_down & (legal_fields >> 63 & VALID_FIELDS);
        let four_up = three_up & (legal_fields << 63 & VALID_FIELDS);

        if self.pieces_left[PieceType::Domino as usize][self.current_player as usize] {
            action_list.append_actions(
                &mut ((two_right & placement_fields) | (two_left & placement_fields) >> 1),
                PieceType::Domino,
                1,
            );
            action_list.append_actions(
                &mut ((two_down & placement_fields) | (two_up & placement_fields) >> 21),
                PieceType::Domino,
                2,
            );
        }

        if self.pieces_left[PieceType::ITromino as usize][self.current_player as usize] {
            action_list.append_actions(
                &mut ((three_right & placement_fields) | (three_left & placement_fields) >> 2),
                PieceType::ITromino,
                3,
            );
            action_list.append_actions(
                &mut ((three_up & placement_fields) >> 42 | (three_down & placement_fields)),
                PieceType::ITromino,
                4,
            );
        }

        if self.pieces_left[PieceType::ITetromino as usize][self.current_player as usize] {
            action_list.append_actions(
                &mut ((four_right & placement_fields) | (four_left & placement_fields) >> 3),
                PieceType::ITetromino,
                5,
            );
            action_list.append_actions(
                &mut ((four_down & placement_fields) | (four_up & placement_fields) >> 63),
                PieceType::ITetromino,
                6,
            );
        }

        if self.pieces_left[PieceType::IPentomino as usize][self.current_player as usize] {
            action_list.append_actions(
                &mut ((four_right & legal_fields >> 4 & placement_fields)
                    | (four_left & legal_fields << 4 & placement_fields) >> 4),
                PieceType::IPentomino,
                7,
            );
            action_list.append_actions(
                &mut ((four_down & legal_fields >> 84 & placement_fields)
                    | (four_up & legal_fields << 84 & placement_fields) >> 84),
                PieceType::IPentomino,
                8,
            );
        }

        if self.pieces_left[PieceType::XPentomino as usize][self.current_player as usize] {
            action_list.append_actions(
                &mut ((three_right >> 20 & three_down)
                    & (placement_fields
                        | placement_fields >> 20
                        | placement_fields >> 22
                        | placement_fields >> 42)),
                PieceType::XPentomino,
                10,
            )
        }

        if self.pieces_left[PieceType::LTromino as usize][self.current_player as usize] {
            action_list.append_actions(
                &mut ((two_up & two_right) >> 21
                    & (placement_fields | placement_fields >> 21 | placement_fields >> 22)),
                PieceType::LTromino,
                11,
            );
            action_list.append_actions(
                &mut ((two_down & two_right)
                    & (placement_fields | placement_fields >> 1 | placement_fields >> 21)),
                PieceType::LTromino,
                12,
            );
            action_list.append_actions(
                &mut ((two_down >> 1 & two_right)
                    & (placement_fields | placement_fields >> 1 | placement_fields >> 22)),
                PieceType::LTromino,
                13,
            );
            action_list.append_actions(
                &mut ((two_down >> 1 & two_right >> 21)
                    & (placement_fields >> 1 | placement_fields >> 21 | placement_fields >> 22)),
                PieceType::LTromino,
                14,
            );
        }

        if self.pieces_left[PieceType::LPentomino as usize][self.current_player as usize] {
            action_list.append_actions(
                &mut ((four_right & legal_fields >> 24)
                    & (placement_fields | placement_fields >> 3 | placement_fields >> 24)),
                PieceType::LPentomino,
                23,
            );
            action_list.append_actions(
                &mut ((four_right & two_down)
                    & (placement_fields | placement_fields >> 3 | placement_fields >> 21)),
                PieceType::LPentomino,
                24,
            );
            action_list.append_actions(
                &mut ((legal_fields & four_right >> 21)
                    & (placement_fields | placement_fields >> 21 | placement_fields >> 24)),
                PieceType::LPentomino,
                25,
            );
            action_list.append_actions(
                &mut ((four_left & two_up) >> 24
                    & (placement_fields >> 3 | placement_fields >> 21 | placement_fields >> 24)),
                PieceType::LPentomino,
                26,
            );
            action_list.append_actions(
                &mut ((two_right & four_down)
                    & (placement_fields | placement_fields >> 1 | placement_fields >> 63)),
                PieceType::LPentomino,
                27,
            );
            action_list.append_actions(
                &mut ((four_down & legal_fields >> 64)
                    & (placement_fields | placement_fields >> 63 | placement_fields >> 64)),
                PieceType::LPentomino,
                28,
            );
            action_list.append_actions(
                &mut ((two_right & four_down >> 1)
                    & (placement_fields | placement_fields >> 1 | placement_fields >> 64)),
                PieceType::LPentomino,
                29,
            );
            action_list.append_actions(
                &mut ((four_up & two_left) >> 64
                    & (placement_fields >> 1 | placement_fields >> 63 | placement_fields >> 64)),
                PieceType::LPentomino,
                30,
            );
        }

        if self.pieces_left[PieceType::TPentomino as usize][self.current_player as usize] {
            action_list.append_actions(
                &mut ((three_right & three_down >> 1)
                    & (placement_fields | placement_fields >> 2 | placement_fields >> 43)),
                PieceType::TPentomino,
                31,
            );
            action_list.append_actions(
                &mut ((two_left & two_right & three_up) >> 43
                    & (placement_fields >> 1 | placement_fields >> 42 | placement_fields >> 44)),
                PieceType::TPentomino,
                32,
            );
            action_list.append_actions(
                &mut ((three_down & three_right >> 21)
                    & (placement_fields | placement_fields >> 23 | placement_fields >> 42)),
                PieceType::TPentomino,
                33,
            );
            action_list.append_actions(
                &mut ((three_left & two_up & two_down) >> 23
                    & (placement_fields >> 2 | placement_fields >> 21 | placement_fields >> 44)),
                PieceType::TPentomino,
                34,
            );
        }

        if self.pieces_left[PieceType::ZPentomino as usize][self.current_player as usize] {
            action_list.append_actions(
                &mut ((legal_fields & (three_left & two_down) >> 23)
                    & (placement_fields
                        | placement_fields >> 21
                        | placement_fields >> 23
                        | placement_fields >> 44)),
                PieceType::ZPentomino,
                43,
            );
            action_list.append_actions(
                &mut (((legal_fields & (three_right & two_down) >> 19)
                    & (placement_fields
                        | placement_fields >> 19
                        | placement_fields >> 21
                        | placement_fields >> 40))
                    >> 2),
                PieceType::ZPentomino,
                44,
            );
            action_list.append_actions(
                &mut ((legal_fields >> 2 & (three_up & two_left) >> 43)
                    & (placement_fields >> 1
                        | placement_fields >> 2
                        | placement_fields >> 42
                        | placement_fields >> 43)),
                PieceType::ZPentomino,
                45,
            );
            action_list.append_actions(
                &mut ((two_right & (two_right & three_up) >> 43)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 43
                        | placement_fields >> 44)),
                PieceType::ZPentomino,
                46,
            );
        }

        if self.pieces_left[PieceType::UPentomino as usize][self.current_player as usize] {
            action_list.append_actions(
                &mut ((three_right & two_down & legal_fields >> 23)
                    & (placement_fields
                        | placement_fields >> 2
                        | placement_fields >> 21
                        | placement_fields >> 23)),
                PieceType::UPentomino,
                47,
            );
            action_list.append_actions(
                &mut ((legal_fields & (three_left & two_up) >> 23)
                    & (placement_fields
                        | placement_fields >> 2
                        | placement_fields >> 21
                        | placement_fields >> 23)),
                PieceType::UPentomino,
                48,
            );
            action_list.append_actions(
                &mut ((three_down & two_right & legal_fields >> 43)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 42
                        | placement_fields >> 43)),
                PieceType::UPentomino,
                49,
            );
            action_list.append_actions(
                &mut ((two_right & (two_left & three_up) >> 43)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 42
                        | placement_fields >> 43)),
                PieceType::UPentomino,
                50,
            );
        }

        if self.pieces_left[PieceType::FPentomino as usize][self.current_player as usize] {
            action_list.append_actions(
                &mut (((three_up & two_left) >> 43 & legal_fields >> 23)
                    & (placement_fields >> 1
                        | placement_fields >> 23
                        | placement_fields >> 42
                        | placement_fields >> 43)),
                PieceType::FPentomino,
                51,
            );
            action_list.append_actions(
                &mut ((legal_fields >> 21 & (three_up & two_right) >> 43)
                    & (placement_fields >> 1
                        | placement_fields >> 21
                        | placement_fields >> 43
                        | placement_fields >> 44)),
                PieceType::FPentomino,
                52,
            );
            action_list.append_actions(
                &mut ((((three_down & two_right) & legal_fields >> 20)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 20
                        | placement_fields >> 42))
                    >> 1),
                PieceType::FPentomino,
                53,
            );
            action_list.append_actions(
                &mut (((three_down & two_left) >> 1 & legal_fields >> 23)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 23
                        | placement_fields >> 43)),
                PieceType::FPentomino,
                54,
            );
            action_list.append_actions(
                &mut (((three_left & two_up) >> 23 & legal_fields >> 43)
                    & (placement_fields >> 2
                        | placement_fields >> 21
                        | placement_fields >> 23
                        | placement_fields >> 43)),
                PieceType::FPentomino,
                55,
            );
            action_list.append_actions(
                &mut (((three_right & two_up) >> 21 & legal_fields >> 43)
                    & (placement_fields
                        | placement_fields >> 21
                        | placement_fields >> 23
                        | placement_fields >> 43)),
                PieceType::FPentomino,
                56,
            );
            action_list.append_actions(
                &mut (((legal_fields & (three_left & two_down) >> 22)
                    & (placement_fields
                        | placement_fields >> 20
                        | placement_fields >> 22
                        | placement_fields >> 43))
                    >> 1),
                PieceType::FPentomino,
                57,
            );
            action_list.append_actions(
                &mut (((legal_fields & (three_right & two_down) >> 20)
                    & (placement_fields
                        | placement_fields >> 20
                        | placement_fields >> 22
                        | placement_fields >> 41))
                    >> 1),
                PieceType::FPentomino,
                58,
            );
        }

        if self.pieces_left[PieceType::WPentomino as usize][self.current_player as usize] {
            action_list.append_actions(
                &mut ((two_down & (two_up & two_right) >> 43)
                    & (placement_fields
                        | placement_fields >> 21
                        | placement_fields >> 22
                        | placement_fields >> 43
                        | placement_fields >> 44)),
                PieceType::WPentomino,
                59,
            );
            action_list.append_actions(
                &mut (((two_up & two_left) >> 23 & two_right >> 42)
                    & (placement_fields >> 2
                        | placement_fields >> 22
                        | placement_fields >> 23
                        | placement_fields >> 42
                        | placement_fields >> 43)),
                PieceType::WPentomino,
                60,
            );
            action_list.append_actions(
                &mut ((two_right & (two_down & two_left) >> 23)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 22
                        | placement_fields >> 23
                        | placement_fields >> 44)),
                PieceType::WPentomino,
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
                PieceType::WPentomino,
                62,
            );
        }

        if self.pieces_left[PieceType::NPentomino as usize][self.current_player as usize] {
            action_list.append_actions(
                &mut (((three_down & two_down >> 41)
                    & (placement_fields
                        | placement_fields >> 41
                        | placement_fields >> 42
                        | placement_fields >> 62))
                    >> 1),
                PieceType::NPentomino,
                63,
            );
            action_list.append_actions(
                &mut ((three_down & two_down >> 43)
                    & (placement_fields
                        | placement_fields >> 42
                        | placement_fields >> 43
                        | placement_fields >> 64)),
                PieceType::NPentomino,
                64,
            );
            action_list.append_actions(
                &mut (((two_down & three_down >> 20)
                    & (placement_fields
                        | placement_fields >> 20
                        | placement_fields >> 21
                        | placement_fields >> 62))
                    >> 1),
                PieceType::NPentomino,
                65,
            );
            action_list.append_actions(
                &mut ((two_down & three_down >> 22)
                    & (placement_fields
                        | placement_fields >> 21
                        | placement_fields >> 22
                        | placement_fields >> 64)),
                PieceType::NPentomino,
                66,
            );
            action_list.append_actions(
                &mut (((two_right & three_right >> 19)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 19
                        | placement_fields >> 21))
                    >> 2),
                PieceType::NPentomino,
                67,
            );
            action_list.append_actions(
                &mut ((three_right & two_right >> 23)
                    & (placement_fields
                        | placement_fields >> 2
                        | placement_fields >> 23
                        | placement_fields >> 24)),
                PieceType::NPentomino,
                68,
            );
            action_list.append_actions(
                &mut ((two_right & three_right >> 22)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 22
                        | placement_fields >> 24)),
                PieceType::NPentomino,
                69,
            );
            action_list.append_actions(
                &mut (((three_right & two_right >> 20)
                    & (placement_fields
                        | placement_fields >> 2
                        | placement_fields >> 20
                        | placement_fields >> 21))
                    >> 1),
                PieceType::NPentomino,
                70,
            );
        }

        if self.pieces_left[PieceType::VPentomino as usize][self.current_player as usize] {
            action_list.append_actions(
                &mut ((three_right & three_down)
                    & (placement_fields | placement_fields >> 2 | placement_fields >> 42)),
                PieceType::VPentomino,
                71,
            );
            action_list.append_actions(
                &mut ((three_up & three_left) >> 44
                    & (placement_fields >> 2 | placement_fields >> 42 | placement_fields >> 44)),
                PieceType::VPentomino,
                72,
            );
            action_list.append_actions(
                &mut ((three_right & three_down >> 2)
                    & (placement_fields | placement_fields >> 2 | placement_fields >> 44)),
                PieceType::VPentomino,
                73,
            );
            action_list.append_actions(
                &mut ((three_down & three_right >> 42)
                    & (placement_fields | placement_fields >> 42 | placement_fields >> 44)),
                PieceType::VPentomino,
                74,
            );
        }

        if self.pieces_left[PieceType::YPentomino as usize][self.current_player as usize] {
            action_list.append_actions(
                &mut ((four_down & legal_fields >> 22)
                    & (placement_fields | placement_fields >> 22 | placement_fields >> 63)),
                PieceType::YPentomino,
                83,
            );
            action_list.append_actions(
                &mut ((four_down & legal_fields >> 43)
                    & (placement_fields | placement_fields >> 43 | placement_fields >> 63)),
                PieceType::YPentomino,
                84,
            );
            action_list.append_actions(
                &mut (((four_down & legal_fields >> 41)
                    & (placement_fields | placement_fields >> 41 | placement_fields >> 63))
                    >> 1),
                PieceType::YPentomino,
                85,
            );
            action_list.append_actions(
                &mut (((four_down & legal_fields >> 20)
                    & (placement_fields | placement_fields >> 20 | placement_fields >> 63))
                    >> 1),
                PieceType::YPentomino,
                86,
            );
            action_list.append_actions(
                &mut ((four_right & legal_fields >> 23)
                    & (placement_fields | placement_fields >> 3 | placement_fields >> 23)),
                PieceType::YPentomino,
                87,
            );
            action_list.append_actions(
                &mut ((four_right & legal_fields >> 22)
                    & (placement_fields | placement_fields >> 3 | placement_fields >> 22)),
                PieceType::YPentomino,
                88,
            );
            action_list.append_actions(
                &mut ((two_up & two_right & three_left) >> 23
                    & (placement_fields >> 2 | placement_fields >> 21 | placement_fields >> 24)),
                PieceType::YPentomino,
                89,
            );
            action_list.append_actions(
                &mut (((legal_fields & four_right >> 20)
                    & (placement_fields | placement_fields >> 20 | placement_fields >> 23))
                    >> 1),
                PieceType::YPentomino,
                90,
            );
        }

        if self.pieces_left[PieceType::TTetromino as usize][self.current_player as usize] {
            action_list.append_actions(
                &mut ((three_right & legal_fields >> 22)
                    & (placement_fields | placement_fields >> 2 | placement_fields >> 22)),
                PieceType::TTetromino,
                35,
            );
            action_list.append_actions(
                &mut ((two_up & two_right & two_left) >> 22
                    & (placement_fields >> 1 | placement_fields >> 21 | placement_fields >> 23)),
                PieceType::TTetromino,
                36,
            );
            action_list.append_actions(
                &mut ((three_down & legal_fields >> 22)
                    & (placement_fields | placement_fields >> 22 | placement_fields >> 42)),
                PieceType::TTetromino,
                37,
            );
            action_list.append_actions(
                &mut ((two_up & two_down & two_left) >> 22
                    & (placement_fields >> 1 | placement_fields >> 21 | placement_fields >> 43)),
                PieceType::TTetromino,
                38,
            );
        }

        {
            let square = two_right & two_right >> 21;
            if self.pieces_left[PieceType::OTetromino as usize][self.current_player as usize] {
                action_list.append_actions(
                    &mut (square
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 21
                            | placement_fields >> 22)),
                    PieceType::OTetromino,
                    9,
                )
            }

            if self.pieces_left[PieceType::PPentomino as usize][self.current_player as usize] {
                action_list.append_actions(
                    &mut ((square & legal_fields >> 42)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 22
                            | placement_fields >> 42)),
                    PieceType::PPentomino,
                    75,
                );
                action_list.append_actions(
                    &mut ((square & legal_fields >> 43)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 21
                            | placement_fields >> 43)),
                    PieceType::PPentomino,
                    76,
                );
                action_list.append_actions(
                    &mut ((square & legal_fields >> 23)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 21
                            | placement_fields >> 23)),
                    PieceType::PPentomino,
                    77,
                );
                action_list.append_actions(
                    &mut ((square & legal_fields >> 2)
                        & (placement_fields
                            | placement_fields >> 2
                            | placement_fields >> 21
                            | placement_fields >> 22)),
                    PieceType::PPentomino,
                    78,
                );
                action_list.append_actions(
                    &mut ((square >> 1 & legal_fields)
                        & (placement_fields
                            | placement_fields >> 2
                            | placement_fields >> 22
                            | placement_fields >> 23)),
                    PieceType::PPentomino,
                    79,
                );
                action_list.append_actions(
                    &mut (((square & legal_fields >> 20)
                        & (placement_fields
                            | placement_fields >> 1
                            | placement_fields >> 20
                            | placement_fields >> 22))
                        >> 1),
                    PieceType::PPentomino,
                    80,
                );
                action_list.append_actions(
                    &mut (((legal_fields & square >> 20)
                        & (placement_fields
                            | placement_fields >> 20
                            | placement_fields >> 41
                            | placement_fields >> 42))
                        >> 1),
                    PieceType::PPentomino,
                    81,
                );
                action_list.append_actions(
                    &mut ((legal_fields & square >> 21)
                        & (placement_fields
                            | placement_fields >> 22
                            | placement_fields >> 42
                            | placement_fields >> 43)),
                    PieceType::PPentomino,
                    82,
                );
            }
        }

        if self.pieces_left[PieceType::ZTetromino as usize][self.current_player as usize] {
            action_list.append_actions(
                &mut (((two_right & two_right >> 20)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 20
                        | placement_fields >> 21))
                    >> 1),
                PieceType::ZTetromino,
                39,
            );
            action_list.append_actions(
                &mut ((two_right & two_right >> 22)
                    & (placement_fields
                        | placement_fields >> 1
                        | placement_fields >> 22
                        | placement_fields >> 23)),
                PieceType::ZTetromino,
                40,
            );
            action_list.append_actions(
                &mut (((two_down & two_down >> 20)
                    & (placement_fields
                        | placement_fields >> 20
                        | placement_fields >> 21
                        | placement_fields >> 41))
                    >> 1),
                PieceType::ZTetromino,
                41,
            );
            action_list.append_actions(
                &mut ((two_down & two_down >> 22)
                    & (placement_fields
                        | placement_fields >> 21
                        | placement_fields >> 22
                        | placement_fields >> 43)),
                PieceType::ZTetromino,
                42,
            );
        }

        if self.pieces_left[PieceType::LTetromino as usize][self.current_player as usize] {
            action_list.append_actions(
                &mut ((three_down & two_right)
                    & (placement_fields | placement_fields >> 1 | placement_fields >> 42)),
                PieceType::LTetromino,
                15,
            );
            action_list.append_actions(
                &mut ((two_right & three_down >> 1)
                    & (placement_fields | placement_fields >> 1 | placement_fields >> 43)),
                PieceType::LTetromino,
                16,
            );
            action_list.append_actions(
                &mut (((three_down & two_right >> 41)
                    & (placement_fields | placement_fields >> 41 | placement_fields >> 42))
                    >> 1),
                PieceType::LTetromino,
                17,
            );
            action_list.append_actions(
                &mut ((three_down & two_right >> 42)
                    & (placement_fields | placement_fields >> 42 | placement_fields >> 43)),
                PieceType::LTetromino,
                18,
            );
            action_list.append_actions(
                &mut ((legal_fields & three_right >> 21)
                    & (placement_fields | placement_fields >> 21 | placement_fields >> 23)),
                PieceType::LTetromino,
                19,
            );
            action_list.append_actions(
                &mut ((three_right & legal_fields >> 21)
                    & (placement_fields | placement_fields >> 2 | placement_fields >> 21)),
                PieceType::LTetromino,
                20,
            );
            action_list.append_actions(
                &mut ((three_right & legal_fields >> 23)
                    & (placement_fields | placement_fields >> 2 | placement_fields >> 23)),
                PieceType::LTetromino,
                21,
            );
            action_list.append_actions(
                &mut ((two_up & three_left) >> 23
                    & (placement_fields >> 2 | placement_fields >> 21 | placement_fields >> 23)),
                PieceType::LTetromino,
                22,
            );
        }

        if self.pieces_left[PieceType::Monomino as usize][self.current_player as usize] {
            action_list.append_actions(&mut placement_fields, PieceType::Monomino, 0);
        }

        if self.ply < 4 {
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

    #[inline(always)]
    pub fn is_game_over(&self) -> bool {
        self.skipped == 0b1111 || self.ply > 100 // the game is over when all players skipped or after round 25 / ply 100
    }

    pub fn game_result(&self) -> i16 {
        let mut scores: [i16; 4] = [
            self.board[Color::BLUE as usize].count_ones() as i16,
            self.board[Color::YELLOW as usize].count_ones() as i16,
            self.board[Color::RED as usize].count_ones() as i16,
            self.board[Color::GREEN as usize].count_ones() as i16,
        ];

        for (i, score) in scores.iter_mut().enumerate() {
            if *score == 89 {
                *score += 5;
            }
            if self.monomino_placed_last[i] {
                *score += 15;
            }
        }
        scores[0] + scores[2] - scores[1] - scores[3]
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
