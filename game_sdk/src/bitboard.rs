use super::Action;
use rand::{rngs::SmallRng, RngCore};
use std::fmt::{Display, Formatter, Result};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};
use std::ops::{Shl, ShlAssign, Shr, ShrAssign};

/* VALID_FIELDS
       X   0     1     2     3     4     5     6     7     8     9    10    11    12    13    14    15    16    17    18    19
    Y
        --------------------------------------------------------------------------------------------------------------------------
    0      0 |   1 |   2 |   3 |   4 |   5 |   6 |   7 |   8 |   9 |  10 |  11 |  12 |  13 |  14 |  15 |  16 |  17 |  18 |  19 | .
         --------------------------------------------------------------------------------------------------------------------------
    1     21 |  22 |  23 |  24 |  25 |  26 |  27 |  28 |  29 |  30 |  31 |  32 |  33 |  34 |  35 |  36 |  37 |  38 |  39 |  40 | .
         --------------------------------------------------------------------------------------------------------------------------
    2     42 |  43 |  44 |  45 |  46 |  47 |  48 |  49 |  50 |  51 |  52 |  53 |  54 |  55 |  56 |  57 |  58 |  59 |  60 |  61 | .
         --------------------------------------------------------------------------------------------------------------------------
    3     63 |  64 |  65 |  66 |  67 |  68 |  69 |  70 |  71 |  72 |  73 |  74 |  75 |  76 |  77 |  78 |  79 |  80 |  81 |  82 | .
         --------------------------------------------------------------------------------------------------------------------------
    4     84 |  85 |  86 |  87 |  88 |  89 |  90 |  91 |  92 |  93 |  94 |  95 |  96 |  97 |  98 |  99 | 100 | 101 | 102 | 103 | .
         --------------------------------------------------------------------------------------------------------------------------
    5    105 | 106 | 107 | 108 | 109 | 110 | 111 | 112 | 113 | 114 | 115 | 116 | 117 | 118 | 119 | 120 | 121 | 122 | 123 | 124 | .
         --------------------------------------------------------------------------------------------------------------------------
    6    126 | 127 | 128 | 129 | 130 | 131 | 132 | 133 | 134 | 135 | 136 | 137 | 138 | 139 | 140 | 141 | 142 | 143 | 144 | 145 | .
         --------------------------------------------------------------------------------------------------------------------------
    7    147 | 148 | 149 | 150 | 151 | 152 | 153 | 154 | 155 | 156 | 157 | 158 | 159 | 160 | 161 | 162 | 163 | 164 | 165 | 166 | .
         --------------------------------------------------------------------------------------------------------------------------
    8    168 | 169 | 170 | 171 | 172 | 173 | 174 | 175 | 176 | 177 | 178 | 179 | 180 | 181 | 182 | 183 | 184 | 185 | 186 | 187 | .
         --------------------------------------------------------------------------------------------------------------------------
    9    189 | 190 | 191 | 192 | 193 | 194 | 195 | 196 | 197 | 198 | 199 | 200 | 201 | 202 | 203 | 204 | 205 | 206 | 207 | 208 | .
         --------------------------------------------------------------------------------------------------------------------------
    10   210 | 211 | 212 | 213 | 214 | 215 | 216 | 217 | 218 | 219 | 220 | 221 | 222 | 223 | 224 | 225 | 226 | 227 | 228 | 229 | .
         --------------------------------------------------------------------------------------------------------------------------
    11   231 | 232 | 233 | 234 | 235 | 236 | 237 | 238 | 239 | 240 | 241 | 242 | 243 | 244 | 245 | 246 | 247 | 248 | 249 | 250 | .
         --------------------------------------------------------------------------------------------------------------------------
    12   252 | 253 | 254 | 255 | 256 | 257 | 258 | 259 | 260 | 261 | 262 | 263 | 264 | 265 | 266 | 267 | 268 | 269 | 270 | 271 | .
         --------------------------------------------------------------------------------------------------------------------------
    13   273 | 274 | 275 | 276 | 277 | 278 | 279 | 280 | 281 | 282 | 283 | 284 | 285 | 286 | 287 | 288 | 289 | 290 | 291 | 292 | .
         --------------------------------------------------------------------------------------------------------------------------
    14   294 | 295 | 296 | 297 | 298 | 299 | 300 | 301 | 302 | 303 | 304 | 305 | 306 | 307 | 308 | 309 | 310 | 311 | 312 | 313 | .
         --------------------------------------------------------------------------------------------------------------------------
    15   315 | 316 | 317 | 318 | 319 | 320 | 321 | 322 | 323 | 324 | 325 | 326 | 327 | 328 | 329 | 330 | 331 | 332 | 333 | 334 | .
         --------------------------------------------------------------------------------------------------------------------------
    16   336 | 337 | 338 | 339 | 340 | 341 | 342 | 343 | 344 | 345 | 346 | 347 | 348 | 349 | 350 | 351 | 352 | 353 | 354 | 355 | .
         --------------------------------------------------------------------------------------------------------------------------
    17   357 | 358 | 359 | 360 | 361 | 362 | 363 | 364 | 365 | 366 | 367 | 368 | 369 | 370 | 371 | 372 | 373 | 374 | 375 | 376 | .
         --------------------------------------------------------------------------------------------------------------------------
    18   378 | 379 | 380 | 381 | 382 | 383 | 384 | 385 | 386 | 387 | 388 | 389 | 390 | 391 | 392 | 393 | 394 | 395 | 396 | 397 | .
         --------------------------------------------------------------------------------------------------------------------------
    19   399 | 400 | 401 | 402 | 403 | 404 | 405 | 406 | 407 | 408 | 409 | 410 | 411 | 412 | 413 | 414 | 415 | 416 | 417 | 418 | .
*/
pub const VALID_FIELDS: Bitboard = Bitboard(
    34359721983,
    337623909661717427026139553986326233087,
    329648537884054317714434393650000297983,
    297747050773401880467613752304696557567,
);

/* COLUMN_MASK
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
*/
pub const COLUMN_MASK: Bitboard = Bitboard(
    32768,
    5316914518442072874470106890883956736,
    21267658073768291497880427563535826944,
    85070632295073165991521710254143307777,
);

/* ROW_MASK
    1  1  1  1  1  1  1  1  1  1  1  1  1  1  1  1  1  1  1  1
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
*/
pub const ROW_MASK: Bitboard = Bitboard(0, 0, 0, 1048575);

/* START_FIELDS
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  1
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .
    1  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  1
*/
pub const START_FIELDS: Bitboard = Bitboard(1 << 34 | 1 << 15, 0, 0, 1 | 1 << 19);

#[derive(Debug, Copy, Clone, Eq)]
pub struct Bitboard(pub u128, pub u128, pub u128, pub u128);

impl Bitboard {
    pub fn empty() -> Self {
        Bitboard(0, 0, 0, 0)
    }

    pub fn with_piece(destination: u16, shape: usize) -> Bitboard {
        let shape = PIECE_SHAPES[shape];
        let shift = destination as u8 & 0b1111111;
        let board = match destination >> 7 {
            0 => Bitboard(0, 0, 0, shape),
            1 => Bitboard(0, 0, shape, 0),
            2 => Bitboard(0, shape, 0, 0),
            _ => Bitboard(shape, 0, 0, 0),
        };
        if shift != 0 {
            board << shift
        } else {
            board
        }
    }

    pub fn bit(bit_idx: u16) -> Bitboard {
        if bit_idx < 128 {
            Bitboard(0, 0, 0, 1 << bit_idx)
        } else if bit_idx < 256 {
            Bitboard(0, 0, 1 << (bit_idx - 128), 0)
        } else if bit_idx < 384 {
            Bitboard(0, 1 << (bit_idx - 256), 0, 0)
        } else {
            Bitboard(1 << (bit_idx - 384), 0, 0, 0)
        }
    }

    pub fn check_bit(&self, bit_idx: u16) -> bool {
        if bit_idx < 128 {
            self.3 & 1 << bit_idx != 0
        } else if bit_idx < 256 {
            self.2 & 1 << (bit_idx - 128) != 0
        } else if bit_idx < 384 {
            self.1 & 1 << (bit_idx - 256) != 0
        } else {
            self.0 & 1 << (bit_idx - 384) != 0
        }
    }

    pub fn flip_bit(&mut self, bit_idx: u16) {
        if bit_idx < 128 {
            self.3 ^= 1 << bit_idx;
        } else if bit_idx < 256 {
            self.2 ^= 1 << (bit_idx - 128);
        } else if bit_idx < 384 {
            self.1 ^= 1 << (bit_idx - 256);
        } else {
            self.0 ^= 1 << (bit_idx - 384);
        }
    }

    pub fn trailing_zeros(&self) -> u16 {
        if self.3 != 0 {
            self.3.trailing_zeros() as u16
        } else if self.2 != 0 {
            self.2.trailing_zeros() as u16 + 128
        } else if self.1 != 0 {
            self.1.trailing_zeros() as u16 + 256
        } else if self.0 != 0 {
            self.0.trailing_zeros() as u16 + 384
        } else {
            512
        }
    }

    pub fn r_shift_save(&self, mut n: usize) -> Bitboard {
        let mut ret = *self;
        while n > 127 {
            n -= 127;
            ret >>= 127;
        }
        if n != 0 {
            ret >> n as u8
        } else {
            ret
        }
    }

    pub fn l_shift_save(&self, mut n: usize) -> Bitboard {
        let mut ret = *self;
        while n > 127 {
            n -= 127;
            ret <<= 127;
        }
        if n != 0 {
            ret << n as u8
        } else {
            ret
        }
    }

    pub fn flip(&self) -> Bitboard {
        let mut board = Bitboard::empty();
        for row in 0..20 {
            board |= (self.r_shift_save(21 * row) & ROW_MASK).l_shift_save((19 - row) * 21);
        }
        board
    }

    pub fn mirror(&self) -> Bitboard {
        let mut board = Bitboard::empty();
        for col in 0..20 {
            board |= (self.r_shift_save(col) & COLUMN_MASK).l_shift_save(19 - col);
        }
        board
    }

    pub fn mirror_diagonal(&self) -> Bitboard {
        let mut board = Bitboard::empty();
        for x in 0..20 {
            for y in 0..20 {
                if self.check_bit((x + y * 21) as u16) {
                    board.flip_bit((y + x * 21) as u16);
                }
            }
        }
        board
    }

    pub fn rotate_left(&self) -> Bitboard {
        self.mirror_diagonal().flip()
    }

    pub fn rotate_right(&self) -> Bitboard {
        self.mirror_diagonal().mirror()
    }

    #[inline(always)]
    pub fn count_ones(&self) -> u32 {
        self.0.count_ones() + self.1.count_ones() + self.2.count_ones() + self.3.count_ones()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0 == 0 && self.1 == 0 && self.2 == 0 && self.3 == 0
    }

    #[inline(always)]
    pub fn not_empty(&self) -> bool {
        self.0 != 0 || self.1 != 0 || self.2 != 0 || self.3 != 0
    }

    #[inline(always)]
    pub fn neighbours(&self) -> Bitboard {
        ((*self << 1) | (*self >> 1) | (*self >> 21) | (*self << 21)) & VALID_FIELDS
    }

    #[inline(always)]
    pub fn diagonal_neighbours(&self) -> Bitboard {
        ((*self << 22) | (*self >> 22) | (*self >> 20) | (*self << 20)) & VALID_FIELDS
    }

    pub fn get_pieces(&self) -> Vec<Action> {
        let mut board = *self;
        let mut actions: Vec<Action> = Vec::with_capacity(21);
        while board.not_empty() {
            let mut piece_board = Bitboard::bit(board.trailing_zeros());
            for _ in 0..5 {
                piece_board |= piece_board.neighbours() & board;
            }
            board ^= piece_board;
            actions.push(Action::from_bitboard(piece_board));
        }
        actions
    }

    pub fn random_field(&mut self, rng: &mut SmallRng) -> u16 {
        let n = self.count_ones() as usize;
        if n < 2 {
            return self.trailing_zeros();
        }
        for _ in 0..rng.next_u32() as usize % n {
            self.flip_bit(self.trailing_zeros());
        }
        self.trailing_zeros()
    }

    pub fn to_fen(&self) -> String {
        format!("{} {} {} {}", self.0, self.1, self.2, self.3)
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self::Output {
        Bitboard(
            self.0 ^ other.0,
            self.1 ^ other.1,
            self.2 ^ other.2,
            self.3 ^ other.3,
        )
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, other: Self) {
        self.0 ^= other.0;
        self.1 ^= other.1;
        self.2 ^= other.2;
        self.3 ^= other.3;
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, other: Self) -> Self::Output {
        Bitboard(
            self.0 & other.0,
            self.1 & other.1,
            self.2 & other.2,
            self.3 & other.3,
        )
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, other: Self) {
        self.0 &= other.0;
        self.1 &= other.1;
        self.2 &= other.2;
        self.3 &= other.3;
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, other: Self) -> Self::Output {
        Bitboard(
            self.0 | other.0,
            self.1 | other.1,
            self.2 | other.2,
            self.3 | other.3,
        )
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
        self.1 |= other.1;
        self.2 |= other.2;
        self.3 |= other.3;
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Bitboard(!self.0, !self.1, !self.2, !self.3)
    }
}

impl Shl<u8> for Bitboard {
    type Output = Self;

    fn shl(self, n: u8) -> Self::Output {
        // 0 < n < 128. Use l_shift_save if n might be 0 or larger than 127
        Bitboard(
            (self.0 << n) | (self.1 >> (128 - n)),
            (self.1 << n) | (self.2 >> (128 - n)),
            (self.2 << n) | (self.3 >> (128 - n)),
            self.3 << n,
        )
    }
}

impl ShlAssign<u8> for Bitboard {
    fn shl_assign(&mut self, n: u8) {
        // 0 < n < 128. Use l_shift_save if n might be 0 or larger than 127
        self.0 = (self.0 << n) | (self.1 >> (128 - n));
        self.1 = (self.1 << n) | (self.2 >> (128 - n));
        self.2 = (self.2 << n) | (self.3 >> (128 - n));
        self.3 = self.3 << n;
    }
}

impl Shr<u8> for Bitboard {
    type Output = Self;

    fn shr(self, n: u8) -> Self::Output {
        // 0 < n < 128. Use r_shift_save if n might be 0 or larger than 127
        Bitboard(
            self.0 >> n,
            (self.1 >> n) | (self.0 << (128 - n)),
            (self.2 >> n) | (self.1 << (128 - n)),
            (self.3 >> n) | self.2 << (128 - n),
        )
    }
}

impl ShrAssign<u8> for Bitboard {
    fn shr_assign(&mut self, n: u8) {
        // 0 < n < 128. Use r_shift_save if n might be 0 or larger than 127
        self.3 = (self.3 >> n) | self.2 << (128 - n);
        self.2 = (self.2 >> n) | (self.1 << (128 - n));
        self.1 = (self.1 >> n) | (self.0 << (128 - n));
        self.0 >>= n;
    }
}

impl PartialEq for Bitboard {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2 && self.3 == other.3
    }
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut string = "0 1 2 3 4 5 6 7 8 9 10        15    19\n".to_string();
        for y in 0..21 {
            for x in 0..21 {
                if self.check_bit(x + y * 21) {
                    if x < 20 && y < 20 {
                        string.push('🟧');
                    } else {
                        string.push('🟥');
                    }
                } else {
                    string.push_str(". ");
                }
            }
            string.push_str(&format!("{}\n", y));
        }
        write!(f, "{}", string)
    }
}

// All shapes on a 128-bit bitboard
pub const PIECE_SHAPES: [u128; 91] = [
    1,                          // Monomino
    3,                          // Domino horizontal
    2097153,                    // Domion vertical
    7,                          // I-Tromino horizontal
    4398048608257,              // I-Tromino vertical
    15,                         // I-Tetromino horizontal
    9223376434903384065,        // I-Tetromino vertical
    31,                         // I-Pentomino horizontal
    19342822337210501698682881, // I-Pentomino vertical
    6291459,                    // O-Tetromino
    8796107702274,              // X-Pentomino
    6291457,                    // L-Tromino
    2097155,
    4194307,
    6291458,
    4398048608259, // L-Tetromino
    8796097216515,
    13194143727618,
    13194141630465,
    14680065,
    2097159,
    8388615,
    14680068,
    16777231, // L-Pentomino
    2097167,
    31457281,
    31457288,
    9223376434903384067,
    27670120508612935681,
    18446752869806768131,
    27670124906661543938,
    8796097216519, // T-Pentomino
    30786329772034,
    4398061191169,
    17592200724484,
    4194311, // T-Tetromino
    14680066,
    4398052802561,
    8796099313666,
    6291462, // Z-Tetromino
    12582915,
    4398052802562,
    8796099313665,
    17592200724481, // Z-Pentomino
    4398061191172,
    13194143727622,
    26388283260931,
    10485767, // U-Pentomino
    14680069,
    13194141630467,
    13194143727619,
    13194152116226, // F-Pentomino
    26388285358082,
    8796099313670,
    8796105605123,
    8796107702276,
    8796107702273,
    17592200724482,
    4398061191170,
    26388285358081, // W-Pentomino
    13194152116228,
    17592198627331,
    4398052802566,
    9223385230998503426, // N-Pentomino
    18446757267851182081,
    9223376434907578370,
    18446752869808865281,
    14680076,
    25165831,
    29360131,
    6291470,
    4398048608263, // V-Pentomino
    30786333966340,
    17592194433031,
    30786327674881,
    4398052802563, // P-Pentomino
    8796099313667,
    14680067,
    6291463,
    12582919,
    14680070,
    13194145824770,
    13194145824769,
    9223376434907578369, // Y-Pentomino
    9223385230996406273,
    18446757267853279234,
    18446752869808865282,
    8388623,
    4194319,
    31457284,
    31457282,
];
