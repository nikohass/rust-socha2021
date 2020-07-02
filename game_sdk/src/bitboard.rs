use std::ops::{
    BitXor, BitXorAssign, BitAnd, BitAndAssign,
    BitOr, BitOrAssign, Not, Shl, ShlAssign, Shr, ShrAssign
};

#[derive(Debug, Copy, Clone)]
pub struct Bitboard {
    pub left: u128,
    pub right: u128
}

impl Bitboard {
    pub fn new() -> Bitboard {
        return Bitboard {
            left: 0,
            right: 0
        }
    }

    pub const fn from(left: u128, right: u128) -> Bitboard {
        return Bitboard {
            left: left,
            right: right
        }
    }

    pub fn bit(n: u8) -> Bitboard {
        if n < 128 {
            return Bitboard::from(0, 1 << n);
        }
        return Bitboard::from(1 << (n - 128), 0);
    }

    pub fn count_ones(&self) -> u32 {
        return self.left.count_ones() + self.right.count_ones();
    }

    pub fn trailing_zeros(&self) -> u32 {
        if self.left == 0 {
            return self.right.trailing_zeros() + 128;
        }
        return self.left.trailing_zeros();
    }

    pub fn not_zero(&self) -> bool {
        self.left != 0 || self.right != 0
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self::Output {
        Bitboard {
            left: self.left ^ other.left,
            right: self.right ^ other.right
        }
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, other: Self) {
        self.left ^= other.left;
        self.right ^= other.right;
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, other: Self) -> Self::Output {
        Bitboard {
            left: self.left & other.left,
            right: self.right & other.right
        }
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, other: Self) {
        self.left &= other.left;
        self.right &= other.right;
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, other: Self) -> Self::Output {
        Bitboard {
            left: self.left | other.left,
            right: self.right | other.right
        }
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, other: Self) {
        self.left |= other.left;
        self.right |= other.right;
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Bitboard {
            left: !self.left,
            right: !self.right
        }
    }
}

impl Shl<u8> for Bitboard {
    type Output = Self;

    fn shl(self, n: u8) -> Self::Output {
        Bitboard {
            left: (self.left << n) | (self.right >> (128 - n)),
            right: self.right << n
        }
    }
}

impl ShlAssign<u8> for Bitboard {
    fn shl_assign(&mut self, n: u8) {
        self.left = (self.left << n) | (self.right >> (128 - n));
        self.right <<= n;
    }
}

impl Shr<u8> for Bitboard {
    type Output = Self;

    fn shr(self, n: u8) -> Self::Output {
        Bitboard {
            left: self.left >> n,
            right: (self.right >> n) | (self.left << (128 - n))
        }
    }
}

impl ShrAssign<u8> for Bitboard {
    fn shr_assign(&mut self, n: u8) {
        self.right = (self.right >> n) | (self.left << (128 - n));
        self.left >>= n;
    }
}

impl PartialEq for Bitboard {
    fn eq(&self, other: &Self) -> bool {
        return self.right == other.right && self.left == other.left;
    }
}
