use super::hashing::{DESTINATION_HASH, PLY_HASH, SHAPE_HASH};
use super::{Action, ActionList, Bitboard, PieceType};
use super::{PIECE_TYPES, START_FIELDS, VALID_FIELDS};
use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Eq, PartialEq)]
pub struct GameState {
    pub ply: u8,                         // Current turn of the GameState
    pub board: [Bitboard; 4],            // 512-bit bitboards (indexed by color)
    pub pieces_left: [[bool; 4]; 21], // Array that stores which player has which pieces left (indexed by piece_type, color)
    pub monomino_placed_last: [bool; 4], // Saves whether a player's last action was a Monomino (indexed by color)
    pub skipped: u64,                    // Keeps track of which player skipped
    pub start_piece_type: PieceType, // The piece type that each player has to place in the first round
    pub hash: u64,                   // Hash of the current state. Only used in Minimax
}

impl GameState {
    pub fn random() -> GameState {
        // Returns an empty GameState with a random start_piece_type
        GameState {
            ply: 0,
            board: [Bitboard::empty(); 4],
            pieces_left: [[true; 4]; 21],
            monomino_placed_last: [false; 4],
            skipped: 0,
            start_piece_type: PieceType::random_pentomino(),
            hash: 0,
        }
    }

    #[inline(always)]
    pub fn get_current_color(&self) -> usize {
        // Blue = 0
        // Yellow = 1
        // Red = 2
        // Green = 3
        (self.ply & 0b11) as usize
    }

    #[inline(always)]
    pub fn get_team(&self) -> i16 {
        // Returns -1 for team Blue/Red and +1 for team Yellow/Green
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
            self.skipped = self.skipped << 4 | self.skipped & 0b1111 | 1 << color;
        } else {
            let destination = action.get_destination();
            let shape = action.get_shape() as usize;
            let piece_type = PieceType::from_shape(shape);
            self.pieces_left[piece_type as usize][color] = false;
            self.board[color] ^= Bitboard::with_piece(destination, shape);
            self.hash ^= SHAPE_HASH[shape][color] ^ DESTINATION_HASH[destination as usize][color];
            self.monomino_placed_last[color] = piece_type == PieceType::Monomino;
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
            self.pieces_left[piece_type as usize][color] = true;
            self.board[color] ^= Bitboard::with_piece(destination, shape);
            self.hash ^= SHAPE_HASH[shape][color] ^ DESTINATION_HASH[destination as usize][color];
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
        let legal_fields = !(own_fields | other_fields | own_fields.neighbors()) & VALID_FIELDS;
        let p = if self.ply > 3 {
            own_fields.diagonal_neighbors() & legal_fields
        } else {
            START_FIELDS & !other_fields
        };
        if (piece & p).is_empty() {
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
            // The color has no possible actions if it had to skip in a previous round
            al.push(Action::SKIP);
            return;
        }
        // Fields that are occupied by the current color
        let own_fields = self.board[color];
        // All fields that are occupied by the other colors
        let other_fields = self.get_occupied_fields() & !own_fields;
        // Fields that newly placed pieces can occupy
        let legal_fields = !(own_fields | other_fields | own_fields.neighbors()) & VALID_FIELDS;
        // Calculate the corners of existing pieces at which new pieces can be placed
        let p = if self.ply > 3 {
            own_fields.diagonal_neighbors() & legal_fields
        } else {
            START_FIELDS & !other_fields
        };
        // Create a lot of shortcuts to speed up the action generation
        let mut shortcuts: [Bitboard; 13] = [Bitboard::empty(); 13];
        shortcuts[0] = legal_fields & (legal_fields >> 1 & VALID_FIELDS);
        shortcuts[1] = legal_fields & (legal_fields << 1 & VALID_FIELDS);
        shortcuts[2] = legal_fields & (legal_fields >> 21 & VALID_FIELDS);
        shortcuts[3] = legal_fields & (legal_fields << 21 & VALID_FIELDS);
        shortcuts[4] = shortcuts[0] & (legal_fields >> 2 & VALID_FIELDS);
        shortcuts[5] = shortcuts[1] & (legal_fields << 2 & VALID_FIELDS);
        shortcuts[6] = shortcuts[2] & (legal_fields >> 42 & VALID_FIELDS);
        shortcuts[7] = shortcuts[3] & (legal_fields << 42 & VALID_FIELDS);
        shortcuts[8] = shortcuts[4] & (legal_fields >> 3 & VALID_FIELDS);
        shortcuts[9] = legal_fields;
        shortcuts[10] = shortcuts[6] & (legal_fields >> 63 & VALID_FIELDS);
        shortcuts[11] = shortcuts[0] & shortcuts[0] >> 21;
        shortcuts[12] = p;

        // Add all legal actions for each piece type to the ActionList
        for (piece_type, generator) in ACTION_GENERATORS.iter().enumerate() {
            if self.pieces_left[piece_type + 1][color] {
                generator(shortcuts, al);
            }
        }
        if self.pieces_left[PieceType::Monomino as usize][color] {
            al.append(p, 0);
        }
        if self.ply < 4 {
            // Remove all piece types that are not the start piece type
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
            al.push(Action::SKIP);
        }
    }

    #[inline(always)]
    pub fn is_game_over(&self) -> bool {
        self.skipped & 0b1111 == 0b1111 || self.ply > 100 // The game is over after all colors have skipped
    }

    pub fn game_result(&self) -> i16 {
        // Only works when the game is over
        // Returns a positive value if team Blue/Red won, a negative value if team Yellow/Green won, and 0 if the game ended in a draw
        let mut result: i16 = 0;
        for (color, board) in self.board.iter().enumerate() {
            let fields = board.count_ones() as i16;
            result -= (fields
                + (fields == 89) as i16 * (15 + 5 * (self.monomino_placed_last[color]) as i16))
                * (((color as i16 & 0b1) << 1) - 1);
        }
        result
    }

    pub fn to_fen(&self) -> String {
        let mut data = (self.start_piece_type as u128) << 4;
        data |= (self.ply as u128) << 9;
        data |= (self.skipped as u128) << 17;
        let mut pieces: u128 = 0;
        for color in 0..4 {
            for piece_type in 0..21 {
                if !self.pieces_left[piece_type][color] {
                    pieces |= 1 << (piece_type + color * 21);
                }
            }
            data |= (self.monomino_placed_last[color] as u128) << color
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
            state.monomino_placed_last[color] = data & (1 << color) != 0;
        }
        for color in 0..4 {
            state.board[color].0 = entries.remove(0).parse::<u128>().unwrap();
            state.board[color].1 = entries.remove(0).parse::<u128>().unwrap();
            state.board[color].2 = entries.remove(0).parse::<u128>().unwrap();
            state.board[color].3 = entries.remove(0).parse::<u128>().unwrap();
        }
        state
    }

    pub fn display_board(&self, board: Bitboard) -> String {
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
                } else if board.check_bit(field) {
                    string.push('🟧');
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
        string
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.display_board(Bitboard::empty()))
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            ply: 0,
            board: [Bitboard::empty(); 4],
            pieces_left: [[true; 4]; 21],
            monomino_placed_last: [false; 4],
            skipped: 0,
            start_piece_type: PieceType::LPentomino,
            hash: 0,
        }
    }
}

fn domino(s: [Bitboard; 13], al: &mut ActionList) {
    let p = s[12];
    al.append(s[0] & (p | p >> 1), 1);
    al.append(s[2] & (p | p >> 21), 2);
}

fn i_tromino(s: [Bitboard; 13], al: &mut ActionList) {
    let p = s[12];
    al.append(s[4] & (p | p >> 2), 3);
    al.append(s[6] & (p | p >> 42), 4);
}

fn l_tromino(s: [Bitboard; 13], al: &mut ActionList) {
    let p = s[12];
    al.append((s[3] & s[0]) >> 21 & (p | p >> 21 | p >> 22), 11);
    al.append((s[2] & s[0]) & (p | p >> 1 | p >> 21), 12);
    al.append((s[2] >> 1 & s[0]) & (p | p >> 1 | p >> 22), 13);
    al.append((s[2] >> 1 & s[0] >> 21) & (p >> 1 | p >> 21 | p >> 22), 14);
}

fn i_tetromino(s: [Bitboard; 13], al: &mut ActionList) {
    let p = s[12];
    al.append(s[8] & (p | p >> 3), 5);
    al.append(s[10] & (p | p >> 63), 6);
}

fn l_tetromino(s: [Bitboard; 13], al: &mut ActionList) {
    let p = s[12];
    al.append((s[6] & s[0]) & (p | p >> 1 | p >> 42), 15);
    al.append((s[0] & s[6] >> 1) & (p | p >> 1 | p >> 43), 16);
    al.append(((s[6] & s[0] >> 41) & (p | p >> 41 | p >> 42)) >> 1, 17);
    al.append((s[6] & s[0] >> 42) & (p | p >> 42 | p >> 43), 18);
    al.append((s[9] & s[4] >> 21) & (p | p >> 21 | p >> 23), 19);
    al.append((s[4] & s[9] >> 21) & (p | p >> 2 | p >> 21), 20);
    al.append((s[4] & s[9] >> 23) & (p | p >> 2 | p >> 23), 21);
    al.append((s[3] & s[5]) >> 23 & (p >> 2 | p >> 21 | p >> 23), 22);
}

fn t_tetromino(s: [Bitboard; 13], al: &mut ActionList) {
    let p = s[12];
    al.append((s[4] & s[9] >> 22) & (p | p >> 2 | p >> 22), 35);
    al.append(
        (s[3] & s[0] & s[1]) >> 22 & (p >> 1 | p >> 21 | p >> 23),
        36,
    );
    al.append((s[6] & s[9] >> 22) & (p | p >> 22 | p >> 42), 37);
    al.append(
        (s[3] & s[2] & s[1]) >> 22 & (p >> 1 | p >> 21 | p >> 43),
        38,
    );
}

fn o_tetromino(s: [Bitboard; 13], al: &mut ActionList) {
    let p = s[12];
    al.append(s[11] & (p | p >> 1 | p >> 21 | p >> 22), 9)
}

fn z_tetromino(s: [Bitboard; 13], al: &mut ActionList) {
    let p = s[12];
    al.append(
        ((s[0] & s[0] >> 20) & (p | p >> 1 | p >> 20 | p >> 21)) >> 1,
        39,
    );
    al.append((s[0] & s[0] >> 22) & (p | p >> 1 | p >> 22 | p >> 23), 40);
    al.append(
        ((s[2] & s[2] >> 20) & (p | p >> 20 | p >> 21 | p >> 41)) >> 1,
        41,
    );
    al.append((s[2] & s[2] >> 22) & (p | p >> 21 | p >> 22 | p >> 43), 42);
}

fn f_pentomino(s: [Bitboard; 13], al: &mut ActionList) {
    let p = s[12];
    al.append(
        ((s[7] & s[1]) >> 43 & s[9] >> 23) & (p >> 1 | p >> 23 | p >> 42 | p >> 43),
        51,
    );
    al.append(
        (s[9] >> 21 & (s[7] & s[0]) >> 43) & (p >> 1 | p >> 21 | p >> 43 | p >> 44),
        52,
    );
    al.append(
        (((s[6] & s[0]) & s[9] >> 20) & (p | p >> 1 | p >> 20 | p >> 42)) >> 1,
        53,
    );
    al.append(
        ((s[6] & s[1]) >> 1 & s[9] >> 23) & (p | p >> 1 | p >> 23 | p >> 43),
        54,
    );
    al.append(
        ((s[5] & s[3]) >> 23 & s[9] >> 43) & (p >> 2 | p >> 21 | p >> 23 | p >> 43),
        55,
    );
    al.append(
        ((s[4] & s[3]) >> 21 & s[9] >> 43) & (p | p >> 21 | p >> 23 | p >> 43),
        56,
    );
    al.append(
        ((s[9] & (s[5] & s[2]) >> 22) & (p | p >> 20 | p >> 22 | p >> 43)) >> 1,
        57,
    );
    al.append(
        ((s[9] & (s[4] & s[2]) >> 20) & (p | p >> 20 | p >> 22 | p >> 41)) >> 1,
        58,
    );
}

fn i_pentomino(s: [Bitboard; 13], al: &mut ActionList) {
    let p = s[12];
    al.append((s[8] & s[9] >> 4) & (p | p >> 4), 7);
    al.append((s[10] & s[9] >> 84) & (p | p >> 84), 8);
}

fn l_pentomino(s: [Bitboard; 13], al: &mut ActionList) {
    let p = s[12];
    al.append((s[8] & s[9] >> 24) & (p | p >> 3 | p >> 24), 23);
    al.append((s[8] & s[2]) & (p | p >> 3 | p >> 21), 24);
    al.append((s[9] & s[8] >> 21) & (p | p >> 21 | p >> 24), 25);
    al.append((s[8] << 3 & s[3]) >> 24 & (p >> 3 | p >> 21 | p >> 24), 26);
    al.append((s[0] & s[10]) & (p | p >> 1 | p >> 63), 27);
    al.append((s[10] & s[9] >> 64) & (p | p >> 63 | p >> 64), 28);
    al.append((s[0] & s[10] >> 1) & (p | p >> 1 | p >> 64), 29);
    al.append((s[10] >> 1 & s[1] >> 64) & (p >> 1 | p >> 63 | p >> 64), 30);
}

fn n_pentomino(s: [Bitboard; 13], al: &mut ActionList) {
    let p = s[12];
    al.append(
        ((s[6] & s[2] >> 41) & (p | p >> 41 | p >> 42 | p >> 62)) >> 1,
        63,
    );
    al.append((s[6] & s[2] >> 43) & (p | p >> 42 | p >> 43 | p >> 64), 64);
    al.append(
        ((s[2] & s[6] >> 20) & (p | p >> 20 | p >> 21 | p >> 62)) >> 1,
        65,
    );
    al.append((s[2] & s[6] >> 22) & (p | p >> 21 | p >> 22 | p >> 64), 66);
    al.append(
        ((s[0] & s[4] >> 19) & (p | p >> 1 | p >> 19 | p >> 21)) >> 2,
        67,
    );
    al.append((s[4] & s[0] >> 23) & (p | p >> 2 | p >> 23 | p >> 24), 68);
    al.append((s[0] & s[4] >> 22) & (p | p >> 1 | p >> 22 | p >> 24), 69);
    al.append(
        ((s[4] & s[0] >> 20) & (p | p >> 2 | p >> 20 | p >> 21)) >> 1,
        70,
    );
}

fn p_pentomino(s: [Bitboard; 13], al: &mut ActionList) {
    let p = s[12];
    al.append((s[11] & s[9] >> 42) & (p | p >> 1 | p >> 22 | p >> 42), 75);
    al.append((s[11] & s[9] >> 43) & (p | p >> 1 | p >> 21 | p >> 43), 76);
    al.append((s[11] & s[9] >> 23) & (p | p >> 1 | p >> 21 | p >> 23), 77);
    al.append((s[11] & s[9] >> 2) & (p | p >> 2 | p >> 21 | p >> 22), 78);
    al.append((s[11] >> 1 & s[9]) & (p | p >> 2 | p >> 22 | p >> 23), 79);
    al.append(
        ((s[11] & s[9] >> 20) & (p | p >> 1 | p >> 20 | p >> 22)) >> 1,
        80,
    );
    al.append(
        ((s[9] & s[11] >> 20) & (p | p >> 20 | p >> 41 | p >> 42)) >> 1,
        81,
    );
    al.append((s[9] & s[11] >> 21) & (p | p >> 22 | p >> 42 | p >> 43), 82);
}

fn t_pentomino(s: [Bitboard; 13], al: &mut ActionList) {
    let p = s[12];
    al.append((s[4] & s[6] >> 1) & (p | p >> 2 | p >> 43), 31);
    al.append(
        (s[1] & s[0] & s[7]) >> 43 & (p >> 1 | p >> 42 | p >> 44),
        32,
    );
    al.append((s[6] & s[4] >> 21) & (p | p >> 23 | p >> 42), 33);
    al.append(
        (s[5] & s[3] & s[2]) >> 23 & (p >> 2 | p >> 21 | p >> 44),
        34,
    );
}

fn u_pentomino(s: [Bitboard; 13], al: &mut ActionList) {
    let p = s[12];
    al.append(
        (s[4] & s[2] & s[9] >> 23) & (p | p >> 2 | p >> 21 | p >> 23),
        47,
    );
    al.append(
        (s[9] & (s[5] & s[3]) >> 23) & (p | p >> 2 | p >> 21 | p >> 23),
        48,
    );
    al.append(
        (s[6] & s[0] & s[9] >> 43) & (p | p >> 1 | p >> 42 | p >> 43),
        49,
    );
    al.append(
        (s[0] & (s[1] & s[7]) >> 43) & (p | p >> 1 | p >> 42 | p >> 43),
        50,
    );
}

fn v_pentomino(s: [Bitboard; 13], al: &mut ActionList) {
    let p = s[12];
    al.append((s[4] & s[6]) & (p | p >> 2 | p >> 42), 71);
    al.append((s[7] & s[5]) >> 44 & (p >> 2 | p >> 42 | p >> 44), 72);
    al.append((s[4] & s[6] >> 2) & (p | p >> 2 | p >> 44), 73);
    al.append((s[6] & s[4] >> 42) & (p | p >> 42 | p >> 44), 74);
}

fn w_pentomino(s: [Bitboard; 13], al: &mut ActionList) {
    let p = s[12];
    al.append(
        (s[2] & (s[3] & s[0]) >> 43) & (p | p >> 21 | p >> 22 | p >> 43 | p >> 44),
        59,
    );
    al.append(
        ((s[3] & s[1]) >> 23 & s[0] >> 42) & (p >> 2 | p >> 22 | p >> 23 | p >> 42 | p >> 43),
        60,
    );
    al.append(
        (s[0] & (s[2] & s[1]) >> 23) & (p | p >> 1 | p >> 22 | p >> 23 | p >> 44),
        61,
    );
    al.append(
        ((s[0] & (s[0] & s[2]) >> 20) & (p | p >> 1 | p >> 20 | p >> 21 | p >> 41)) >> 1,
        62,
    );
}

fn x_pentomino(s: [Bitboard; 13], al: &mut ActionList) {
    let p = s[12];
    al.append(
        ((s[4] >> 20 & s[6]) & (p | p >> 20 | p >> 22 | p >> 42)) >> 1,
        10,
    )
}

fn y_pentomino(s: [Bitboard; 13], al: &mut ActionList) {
    let p = s[12];
    al.append((s[10] & s[9] >> 22) & (p | p >> 22 | p >> 63), 83);
    al.append((s[10] & s[9] >> 43) & (p | p >> 43 | p >> 63), 84);
    al.append(((s[10] & s[9] >> 41) & (p | p >> 41 | p >> 63)) >> 1, 85);
    al.append(((s[10] & s[9] >> 20) & (p | p >> 20 | p >> 63)) >> 1, 86);
    al.append((s[8] & s[9] >> 23) & (p | p >> 3 | p >> 23), 87);
    al.append((s[8] & s[9] >> 22) & (p | p >> 3 | p >> 22), 88);
    al.append(
        (s[3] & s[0] & s[5]) >> 23 & (p >> 2 | p >> 21 | p >> 24),
        89,
    );
    al.append(((s[9] & s[8] >> 20) & (p | p >> 20 | p >> 23)) >> 1, 90);
}

fn z_pentomino(s: [Bitboard; 13], al: &mut ActionList) {
    let p = s[12];
    al.append(
        (s[9] & (s[5] & s[2]) >> 23) & (p | p >> 21 | p >> 23 | p >> 44),
        43,
    );
    al.append(
        ((s[9] & (s[4] & s[2]) >> 19) & (p | p >> 19 | p >> 21 | p >> 40)) >> 2,
        44,
    );
    al.append(
        (s[9] >> 2 & (s[7] & s[1]) >> 43) & (p >> 1 | p >> 2 | p >> 42 | p >> 43),
        45,
    );
    al.append(
        (s[0] & (s[0] & s[7]) >> 43) & (p | p >> 1 | p >> 43 | p >> 44),
        46,
    );
}

const ACTION_GENERATORS: [fn([Bitboard; 13], &mut ActionList); 20] = [
    domino,
    i_tromino,
    l_tromino,
    i_tetromino,
    l_tetromino,
    t_tetromino,
    o_tetromino,
    z_tetromino,
    f_pentomino,
    i_pentomino,
    l_pentomino,
    n_pentomino,
    p_pentomino,
    t_pentomino,
    u_pentomino,
    v_pentomino,
    w_pentomino,
    x_pentomino,
    y_pentomino,
    z_pentomino,
];
