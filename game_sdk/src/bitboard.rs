use super::constants::{COLUMN_MASK, PIECE_SHAPES, ROW_MASK, VALID_FIELDS};
use super::Action;
use rand::{rngs::SmallRng, RngCore};
use std::fmt::{Display, Formatter, Result};
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
    ShrAssign,
};

#[derive(Debug, Copy, Clone, Eq)]
pub struct Bitboard(pub u128, pub u128, pub u128, pub u128);

impl Bitboard {
    pub fn empty() -> Self {
        Bitboard(0, 0, 0, 0)
    }

    pub fn with_piece(to: u16, shape: usize) -> Bitboard {
        let shape = PIECE_SHAPES[shape];
        let shift = to as u8 & 0b1111111;
        let board = match to >> 7 {
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
    pub fn is_zero(&self) -> bool {
        self.0 == 0 && self.1 == 0 && self.2 == 0 && self.3 == 0
    }

    #[inline(always)]
    pub fn not_zero(&self) -> bool {
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
        while board.not_zero() {
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
        self.0 = (self.0 << n) | (self.1 >> (128 - n));
        self.1 = (self.1 << n) | (self.2 >> (128 - n));
        self.2 = (self.2 << n) | (self.3 >> (128 - n));
        self.3 = self.3 << n;
    }
}

impl Shr<u8> for Bitboard {
    type Output = Self;

    fn shr(self, n: u8) -> Self::Output {
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
