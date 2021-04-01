use super::hashing::{FIELD_HASH, PIECE_HASH, PLY_HASH};
use super::{Action, ActionList, Bitboard, PieceType};
use super::{PIECE_TYPES, START_FIELDS, VALID_FIELDS};
use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Eq, PartialEq)]
pub struct GameState {
    pub ply: u8,
    pub board: [Bitboard; 4],
    pub pieces_left: [[bool; 4]; 21],
    pub monomino_placed_last: u8,
    pub skipped: u64,
    pub start_piece_type: PieceType,
    pub hash: u64,
}

impl GameState {
    pub fn random() -> GameState {
        GameState {
            ply: 0,
            board: [Bitboard::empty(); 4],
            pieces_left: [[true; 4]; 21],
            monomino_placed_last: 0,
            skipped: 0,
            start_piece_type: PieceType::random_pentomino(),
            hash: 0,
        }
    }

    #[inline(always)]
    pub fn get_current_color(&self) -> usize {
        (self.ply & 0b11) as usize
    }

    #[inline(always)]
    pub fn get_team(&self) -> i16 {
        ((self.ply as i16 & 0b1) << 1) - 1
    }

    #[inline(always)]
    pub fn get_occupied_fields(&self) -> Bitboard {
        self.board[0] | self.board[1] | self.board[2] | self.board[3]
    }

    #[inline(always)]
    pub fn has_color_skipped(&self, color: usize) -> bool {
        self.skipped & 1 << color != 0
    }

    #[inline(always)]
    pub fn has_team_skipped(&self, team: u8) -> bool {
        let team_mask = 0b101 << (team & 0b1);
        self.skipped & team_mask == team_mask
    }

    pub fn do_action(&mut self, action: Action) {
        debug_assert!(self.validate_action(&action));
        self.hash ^= PLY_HASH[self.ply as usize];
        let color = self.get_current_color();
        if action.is_skip() {
            self.skipped = ((self.skipped & 0b1111) | self.skipped << 4) | (1 << color);
        } else {
            let destination = action.get_destination();
            let shape = action.get_shape() as usize;
            let piece_type = PieceType::from_shape(shape);
            self.pieces_left[piece_type as usize][color] = false;
            self.board[color] ^= Bitboard::with_piece(destination, shape);
            self.hash ^= PIECE_HASH[shape][color] ^ FIELD_HASH[destination as usize][color];
            if piece_type == PieceType::Monomino {
                self.monomino_placed_last |= 1 << color;
            } else {
                self.monomino_placed_last &= !(1 << color);
            }
        };
        self.ply += 1;
        debug_assert!(self.check_integrity());
    }

    pub fn undo_action(&mut self, action: Action) {
        self.ply -= 1;
        self.hash ^= PLY_HASH[self.ply as usize];
        let color = self.get_current_color();
        if action.is_skip() {
            self.skipped >>= 4;
        } else {
            let destination = action.get_destination();
            let shape = action.get_shape() as usize;
            let piece_type = PieceType::from_shape(shape);
            debug_assert!(
                !self.pieces_left[piece_type as usize][color],
                "Can't remove piece that has not been placed."
            );
            self.pieces_left[piece_type as usize][color] = true;
            self.board[color] ^= Bitboard::with_piece(destination, shape);
            self.hash ^= PIECE_HASH[shape][color] ^ FIELD_HASH[destination as usize][color];
        }
        debug_assert!(self.check_integrity());
    }

    pub fn validate_action(&self, action: &Action) -> bool {
        let color = self.get_current_color();
        if action.is_skip() {
            return true;
        }
        let mut is_valid = true;
        let destination = action.get_destination();
        let shape = action.get_shape() as usize;
        let piece_type = PieceType::from_shape(shape);
        if self.ply < 4 && piece_type != self.start_piece_type {
            println!(
                "Invalid piece type. Start piece type is {}",
                self.start_piece_type
            );
            is_valid = false;
        }
        if !self.pieces_left[piece_type as usize][color] {
            println!("Can't place piece that has already been placed.");
            return false;
        }
        let piece = Bitboard::with_piece(destination, shape);
        let own_fields = self.board[color];
        let other_fields = self.get_occupied_fields() & !own_fields;
        let legal_fields = !(own_fields | other_fields | own_fields.neighbours()) & VALID_FIELDS;
        let p = if self.ply > 3 {
            own_fields.diagonal_neighbours() & legal_fields
        } else {
            START_FIELDS & !other_fields
        };
        if (piece & p).is_zero() {
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

    pub fn check_integrity(&self) -> bool {
        for color in 0..4 {
            let pieces = self.board[color].get_pieces();
            let mut pieces_left: [bool; 21] = [true; 21];
            for piece in pieces.iter() {
                let piece_type = PieceType::from_shape(piece.get_shape() as usize);
                pieces_left[piece_type as usize] = false;
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

    pub fn get_possible_actions(&self, al: &mut ActionList) {
        let color = self.get_current_color();
        al.clear();

        if self.has_color_skipped(color) {
            al.push(Action::skip());
            return;
        }
        let own_fields = self.board[color];
        let other_fields = self.get_occupied_fields() & !own_fields;
        let legal_fields = !(own_fields | other_fields | own_fields.neighbours()) & VALID_FIELDS;
        let p = if self.ply > 3 {
            own_fields.diagonal_neighbours() & legal_fields
        } else {
            START_FIELDS & !other_fields
        };

        let r2 = legal_fields & (legal_fields >> 1 & VALID_FIELDS);
        let l2 = legal_fields & (legal_fields << 1 & VALID_FIELDS);
        let d2 = legal_fields & (legal_fields >> 21 & VALID_FIELDS);
        let u2 = legal_fields & (legal_fields << 21 & VALID_FIELDS);

        let r3 = r2 & (legal_fields >> 2 & VALID_FIELDS);
        let l3 = l2 & (legal_fields << 2 & VALID_FIELDS);
        let d3 = d2 & (legal_fields >> 42 & VALID_FIELDS);
        let u3 = u2 & (legal_fields << 42 & VALID_FIELDS);

        let r4 = r3 & (legal_fields >> 3 & VALID_FIELDS);
        let l4 = l3 & (legal_fields << 3 & VALID_FIELDS);
        let d4 = d3 & (legal_fields >> 63 & VALID_FIELDS);
        let u4 = u3 & (legal_fields << 63 & VALID_FIELDS);

        if self.pieces_left[PieceType::FPentomino as usize][color] {
            al.append(
                ((u3 & l2) >> 43 & legal_fields >> 23) & (p >> 1 | p >> 23 | p >> 42 | p >> 43),
                51,
            );
            al.append(
                (legal_fields >> 21 & (u3 & r2) >> 43) & (p >> 1 | p >> 21 | p >> 43 | p >> 44),
                52,
            );
            al.append(
                (((d3 & r2) & legal_fields >> 20) & (p | p >> 1 | p >> 20 | p >> 42)) >> 1,
                53,
            );
            al.append(
                ((d3 & l2) >> 1 & legal_fields >> 23) & (p | p >> 1 | p >> 23 | p >> 43),
                54,
            );
            al.append(
                ((l3 & u2) >> 23 & legal_fields >> 43) & (p >> 2 | p >> 21 | p >> 23 | p >> 43),
                55,
            );
            al.append(
                ((r3 & u2) >> 21 & legal_fields >> 43) & (p | p >> 21 | p >> 23 | p >> 43),
                56,
            );
            al.append(
                ((legal_fields & (l3 & d2) >> 22) & (p | p >> 20 | p >> 22 | p >> 43)) >> 1,
                57,
            );
            al.append(
                ((legal_fields & (r3 & d2) >> 20) & (p | p >> 20 | p >> 22 | p >> 41)) >> 1,
                58,
            );
        }

        if self.pieces_left[PieceType::WPentomino as usize][color] {
            al.append(
                (d2 & (u2 & r2) >> 43) & (p | p >> 21 | p >> 22 | p >> 43 | p >> 44),
                59,
            );
            al.append(
                ((u2 & l2) >> 23 & r2 >> 42) & (p >> 2 | p >> 22 | p >> 23 | p >> 42 | p >> 43),
                60,
            );
            al.append(
                (r2 & (d2 & l2) >> 23) & (p | p >> 1 | p >> 22 | p >> 23 | p >> 44),
                61,
            );
            al.append(
                ((r2 & (r2 & d2) >> 20) & (p | p >> 1 | p >> 20 | p >> 21 | p >> 41)) >> 1,
                62,
            );
        }

        if self.pieces_left[PieceType::LPentomino as usize][color] {
            al.append((r4 & legal_fields >> 24) & (p | p >> 3 | p >> 24), 23);
            al.append((r4 & d2) & (p | p >> 3 | p >> 21), 24);
            al.append((legal_fields & r4 >> 21) & (p | p >> 21 | p >> 24), 25);
            al.append((l4 & u2) >> 24 & (p >> 3 | p >> 21 | p >> 24), 26);
            al.append((r2 & d4) & (p | p >> 1 | p >> 63), 27);
            al.append((d4 & legal_fields >> 64) & (p | p >> 63 | p >> 64), 28);
            al.append((r2 & d4 >> 1) & (p | p >> 1 | p >> 64), 29);
            al.append((u4 & l2) >> 64 & (p >> 1 | p >> 63 | p >> 64), 30);
        }

        if self.pieces_left[PieceType::TPentomino as usize][color] {
            al.append((r3 & d3 >> 1) & (p | p >> 2 | p >> 43), 31);
            al.append((l2 & r2 & u3) >> 43 & (p >> 1 | p >> 42 | p >> 44), 32);
            al.append((d3 & r3 >> 21) & (p | p >> 23 | p >> 42), 33);
            al.append((l3 & u2 & d2) >> 23 & (p >> 2 | p >> 21 | p >> 44), 34);
        }

        if self.pieces_left[PieceType::ZPentomino as usize][color] {
            al.append(
                (legal_fields & (l3 & d2) >> 23) & (p | p >> 21 | p >> 23 | p >> 44),
                43,
            );
            al.append(
                ((legal_fields & (r3 & d2) >> 19) & (p | p >> 19 | p >> 21 | p >> 40)) >> 2,
                44,
            );
            al.append(
                (legal_fields >> 2 & (u3 & l2) >> 43) & (p >> 1 | p >> 2 | p >> 42 | p >> 43),
                45,
            );
            al.append(
                (r2 & (r2 & u3) >> 43) & (p | p >> 1 | p >> 43 | p >> 44),
                46,
            );
        }

        if self.pieces_left[PieceType::Domino as usize][color] {
            al.append((r2 & p) | (l2 & p) >> 1, 1);
            al.append((d2 & p) | (u2 & p) >> 21, 2);
        }

        if self.pieces_left[PieceType::ITromino as usize][color] {
            al.append((r3 & p) | (l3 & p) >> 2, 3);
            al.append((u3 & p) >> 42 | (d3 & p), 4);
        }

        if self.pieces_left[PieceType::ITetromino as usize][color] {
            al.append((r4 & p) | (l4 & p) >> 3, 5);
            al.append((d4 & p) | (u4 & p) >> 63, 6);
        }

        if self.pieces_left[PieceType::IPentomino as usize][color] {
            al.append(
                (r4 & legal_fields >> 4 & p) | (l4 & legal_fields << 4 & p) >> 4,
                7,
            );
            al.append(
                (d4 & legal_fields >> 84 & p) | (u4 & legal_fields << 84 & p) >> 84,
                8,
            );
        }

        if self.pieces_left[PieceType::XPentomino as usize][color] {
            al.append(
                ((r3 >> 20 & d3) & (p | p >> 20 | p >> 22 | p >> 42)) >> 1,
                10,
            )
        }

        if self.pieces_left[PieceType::LTromino as usize][color] {
            al.append((u2 & r2) >> 21 & (p | p >> 21 | p >> 22), 11);
            al.append((d2 & r2) & (p | p >> 1 | p >> 21), 12);
            al.append((d2 >> 1 & r2) & (p | p >> 1 | p >> 22), 13);
            al.append((d2 >> 1 & r2 >> 21) & (p >> 1 | p >> 21 | p >> 22), 14);
        }

        if self.pieces_left[PieceType::UPentomino as usize][color] {
            al.append(
                (r3 & d2 & legal_fields >> 23) & (p | p >> 2 | p >> 21 | p >> 23),
                47,
            );
            al.append(
                (legal_fields & (l3 & u2) >> 23) & (p | p >> 2 | p >> 21 | p >> 23),
                48,
            );
            al.append(
                (d3 & r2 & legal_fields >> 43) & (p | p >> 1 | p >> 42 | p >> 43),
                49,
            );
            al.append(
                (r2 & (l2 & u3) >> 43) & (p | p >> 1 | p >> 42 | p >> 43),
                50,
            );
        }

        if self.pieces_left[PieceType::NPentomino as usize][color] {
            al.append(
                ((d3 & d2 >> 41) & (p | p >> 41 | p >> 42 | p >> 62)) >> 1,
                63,
            );
            al.append((d3 & d2 >> 43) & (p | p >> 42 | p >> 43 | p >> 64), 64);
            al.append(
                ((d2 & d3 >> 20) & (p | p >> 20 | p >> 21 | p >> 62)) >> 1,
                65,
            );
            al.append((d2 & d3 >> 22) & (p | p >> 21 | p >> 22 | p >> 64), 66);
            al.append(
                ((r2 & r3 >> 19) & (p | p >> 1 | p >> 19 | p >> 21)) >> 2,
                67,
            );
            al.append((r3 & r2 >> 23) & (p | p >> 2 | p >> 23 | p >> 24), 68);
            al.append((r2 & r3 >> 22) & (p | p >> 1 | p >> 22 | p >> 24), 69);
            al.append(
                ((r3 & r2 >> 20) & (p | p >> 2 | p >> 20 | p >> 21)) >> 1,
                70,
            );
        }

        if self.pieces_left[PieceType::VPentomino as usize][color] {
            al.append((r3 & d3) & (p | p >> 2 | p >> 42), 71);
            al.append((u3 & l3) >> 44 & (p >> 2 | p >> 42 | p >> 44), 72);
            al.append((r3 & d3 >> 2) & (p | p >> 2 | p >> 44), 73);
            al.append((d3 & r3 >> 42) & (p | p >> 42 | p >> 44), 74);
        }

        if self.pieces_left[PieceType::YPentomino as usize][color] {
            al.append((d4 & legal_fields >> 22) & (p | p >> 22 | p >> 63), 83);
            al.append((d4 & legal_fields >> 43) & (p | p >> 43 | p >> 63), 84);
            al.append(
                ((d4 & legal_fields >> 41) & (p | p >> 41 | p >> 63)) >> 1,
                85,
            );
            al.append(
                ((d4 & legal_fields >> 20) & (p | p >> 20 | p >> 63)) >> 1,
                86,
            );
            al.append((r4 & legal_fields >> 23) & (p | p >> 3 | p >> 23), 87);
            al.append((r4 & legal_fields >> 22) & (p | p >> 3 | p >> 22), 88);
            al.append((u2 & r2 & l3) >> 23 & (p >> 2 | p >> 21 | p >> 24), 89);
            al.append(
                ((legal_fields & r4 >> 20) & (p | p >> 20 | p >> 23)) >> 1,
                90,
            );
        }

        if self.pieces_left[PieceType::TTetromino as usize][color] {
            al.append((r3 & legal_fields >> 22) & (p | p >> 2 | p >> 22), 35);
            al.append((u2 & r2 & l2) >> 22 & (p >> 1 | p >> 21 | p >> 23), 36);
            al.append((d3 & legal_fields >> 22) & (p | p >> 22 | p >> 42), 37);
            al.append((u2 & d2 & l2) >> 22 & (p >> 1 | p >> 21 | p >> 43), 38);
        }

        let sq = r2 & r2 >> 21;
        if self.pieces_left[PieceType::OTetromino as usize][color] {
            al.append(sq & (p | p >> 1 | p >> 21 | p >> 22), 9)
        }

        if self.pieces_left[PieceType::PPentomino as usize][color] {
            al.append(
                (sq & legal_fields >> 42) & (p | p >> 1 | p >> 22 | p >> 42),
                75,
            );
            al.append(
                (sq & legal_fields >> 43) & (p | p >> 1 | p >> 21 | p >> 43),
                76,
            );
            al.append(
                (sq & legal_fields >> 23) & (p | p >> 1 | p >> 21 | p >> 23),
                77,
            );
            al.append(
                (sq & legal_fields >> 2) & (p | p >> 2 | p >> 21 | p >> 22),
                78,
            );
            al.append(
                (sq >> 1 & legal_fields) & (p | p >> 2 | p >> 22 | p >> 23),
                79,
            );
            al.append(
                ((sq & legal_fields >> 20) & (p | p >> 1 | p >> 20 | p >> 22)) >> 1,
                80,
            );
            al.append(
                ((legal_fields & sq >> 20) & (p | p >> 20 | p >> 41 | p >> 42)) >> 1,
                81,
            );
            al.append(
                (legal_fields & sq >> 21) & (p | p >> 22 | p >> 42 | p >> 43),
                82,
            );
        }

        if self.pieces_left[PieceType::ZTetromino as usize][color] {
            al.append(
                ((r2 & r2 >> 20) & (p | p >> 1 | p >> 20 | p >> 21)) >> 1,
                39,
            );
            al.append((r2 & r2 >> 22) & (p | p >> 1 | p >> 22 | p >> 23), 40);
            al.append(
                ((d2 & d2 >> 20) & (p | p >> 20 | p >> 21 | p >> 41)) >> 1,
                41,
            );
            al.append((d2 & d2 >> 22) & (p | p >> 21 | p >> 22 | p >> 43), 42);
        }

        if self.pieces_left[PieceType::LTetromino as usize][color] {
            al.append((d3 & r2) & (p | p >> 1 | p >> 42), 15);
            al.append((r2 & d3 >> 1) & (p | p >> 1 | p >> 43), 16);
            al.append(((d3 & r2 >> 41) & (p | p >> 41 | p >> 42)) >> 1, 17);
            al.append((d3 & r2 >> 42) & (p | p >> 42 | p >> 43), 18);
            al.append((legal_fields & r3 >> 21) & (p | p >> 21 | p >> 23), 19);
            al.append((r3 & legal_fields >> 21) & (p | p >> 2 | p >> 21), 20);
            al.append((r3 & legal_fields >> 23) & (p | p >> 2 | p >> 23), 21);
            al.append((u2 & l3) >> 23 & (p >> 2 | p >> 21 | p >> 23), 22);
        }

        if self.pieces_left[PieceType::Monomino as usize][color] {
            al.append(p, 0);
        }

        if self.ply < 4 {
            let mut idx = 0;
            for i in 0..al.size {
                let shape = al[i].get_shape() as usize;
                let piece_type = PieceType::from_shape(shape);
                if piece_type == self.start_piece_type {
                    al.swap(idx, i);
                    idx += 1;
                }
            }
            al.size = idx;
        }

        if al.size == 0 {
            al.push(Action::skip());
        }
    }

    #[inline(always)]
    pub fn is_game_over(&self) -> bool {
        self.skipped & 0b1111 == 0b1111 || self.ply > 100 // the game is over when all colors skipped or after round 25 / ply 100
    }

    pub fn game_result(&self) -> i16 {
        let mut scores: [i16; 4] = [
            self.board[0].count_ones() as i16,
            self.board[1].count_ones() as i16,
            self.board[2].count_ones() as i16,
            self.board[3].count_ones() as i16,
        ];

        for (color, score) in scores.iter_mut().enumerate() {
            if *score == 89 {
                *score += 15;
                if self.monomino_placed_last & (1 << color) != 0 {
                    *score += 5;
                }
            }
        }
        scores[0] - scores[1] + scores[2] - scores[3]
    }

    pub fn to_fen(&self) -> String {
        let mut data = self.monomino_placed_last as u128;
        data |= (self.start_piece_type as u128) << 4;
        data |= (self.ply as u128) << 9;
        data |= (self.skipped as u128) << 17;
        let mut pieces: u128 = 0;
        for color in 0..4 {
            for piece_type in 0..21 {
                if !self.pieces_left[piece_type][color] {
                    pieces |= 1 << (piece_type + color * 21);
                }
            }
        }
        format!(
            "{} {} {} {} {} {}",
            data,
            pieces,
            self.board[0].to_fen(),
            self.board[1].to_fen(),
            self.board[2].to_fen(),
            self.board[3].to_fen(),
        )
    }

    pub fn from_fen(string: String) -> GameState {
        let mut entries: Vec<&str> = string.split(' ').collect();
        let mut state = GameState::default();
        let data = entries.remove(0).parse::<u128>().unwrap();
        state.monomino_placed_last = (data & 0b1111) as u8;
        state.start_piece_type = PIECE_TYPES[(data >> 4 & 0b11111) as usize];
        state.ply = (data >> 9 & 0b11111111) as u8;
        state.skipped = (data >> 17) as u64;
        let pieces = entries.remove(0).parse::<u128>().unwrap();
        for color in 0..4 {
            for piece_type in 0..21 {
                if pieces & 1 << (piece_type + color * 21) != 0 {
                    state.pieces_left[piece_type][color] = false;
                }
            }
        }
        for color in 0..4 {
            state.board[color].0 = entries.remove(0).parse::<u128>().unwrap();
            state.board[color].1 = entries.remove(0).parse::<u128>().unwrap();
            state.board[color].2 = entries.remove(0).parse::<u128>().unwrap();
            state.board[color].3 = entries.remove(0).parse::<u128>().unwrap();
        }
        state
    }
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
            match self.get_current_color() {
                0 => "🟦",
                1 => "🟨",
                2 => "🟥",
                _ => "🟩",
            },
            self.ply,
            self.game_result(),
        );
        string.push_str(info);
        for _ in info.len()..45 {
            string.push(' ');
        }
        string.push_str("║\n╠");
        for _ in 0..40 {
            string.push('═');
        }
        string.push('╣');
        for y in 0..20 {
            string.push_str("\n║");
            for x in 0..20 {
                let field = x + y * 21;
                if self.board[0].check_bit(field) {
                    string.push('🟦');
                } else if self.board[1].check_bit(field) {
                    string.push('🟨');
                } else if self.board[2].check_bit(field) {
                    string.push('🟥');
                } else if self.board[3].check_bit(field) {
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
    fn default() -> Self {
        Self {
            ply: 0,
            board: [Bitboard::empty(); 4],
            pieces_left: [[true; 4]; 21],
            monomino_placed_last: 0,
            skipped: 0,
            start_piece_type: PieceType::LPentomino,
            hash: 0,
        }
    }
}
