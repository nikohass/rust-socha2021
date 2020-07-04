use std::ops::{
    BitXor, BitXorAssign, BitAnd, BitAndAssign,
    BitOr, BitOrAssign, Not, Shl, ShlAssign, Shr, ShrAssign
};

pub const VALID_FIELDS: Bitboard = Bitboard::from(
    34359721983,
    337623909661717427026139553986326233087,
    329648537884054317714434393650000297983,
    297747050773401880467613752304696557567
);

#[derive(Debug, Copy, Clone)]
pub struct Bitboard {
    pub one: u128,
    pub two: u128,
    pub three: u128,
    pub four: u128
}

impl Bitboard {
    pub fn new() -> Bitboard {
        Bitboard {
            one: 0,
            two: 0,
            three: 0,
            four: 0
        }
    }

    pub const fn from(one: u128, two: u128, three: u128, four: u128) -> Bitboard {
        Bitboard {
            one: one,
            two: two,
            three: three,
            four: four
        }
    }

    pub fn bit(n: u16) -> Bitboard {
        if n < 128 {
            return Bitboard::from(0, 0, 0, 1 << n);
        }
        if n < 256 {
            return Bitboard::from(0, 0, 1 << (n - 128), 0);
        }
        if n < 384 {
            return Bitboard::from(0, 1 << (n - 256), 0, 0);
        }
        return Bitboard::from(1 << (n - 384), 0, 0, 0);
    }

    pub fn count_ones(&self) -> u32 {
        return self.one.count_ones() + self.two.count_ones()
            + self.three.count_ones() + self.four.count_ones();
    }

    pub fn trailing_zeros(&self) -> u32 {
        if self.one != 0 {
            return self.one.trailing_zeros() + 384;
        }
        if self.two != 0 {
            return self.two.trailing_zeros() + 256;
        }
        if self.three != 0 {
            return self.three.trailing_zeros() + 128;
        }
        if self.four != 0 {
            return self.four.trailing_zeros();
        }
        512
    }

    pub fn not_zero(&self) -> bool {
        self.one != 0 || self.two != 0 || self.three != 0 || self.four != 0
    }

    pub fn neighbours(&self) -> Bitboard {
        ((*self << 1) | (*self >> 1) | (*self >> 21) | (*self << 21)) & VALID_FIELDS
    }

    pub fn diagonal_neighbours(&self) -> Bitboard {
        ((*self << 22) | (*self >> 22) | (*self >> 20) | (*self << 20)) & VALID_FIELDS
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self::Output {
        Bitboard {
            one: self.one ^ other.one,
            two: self.two ^ other.two,
            three: self.three ^ other.three,
            four: self.four ^ other.four
        }
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, other: Self) {
        self.one ^= other.one;
        self.two ^= other.two;
        self.three ^= other.three;
        self.four ^= other.four;
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, other: Self) -> Self::Output {
        Bitboard {
            one: self.one & other.one,
            two: self.two & other.two,
            three: self.three & other.three,
            four: self.four & other.four
        }
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, other: Self) {
        self.one &= other.one;
        self.two &= other.two;
        self.three &= other.three;
        self.four &= other.four;
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, other: Self) -> Self::Output {
        Bitboard {
            one: self.one | other.one,
            two: self.two | other.two,
            three: self.three | other.three,
            four: self.four | other.four
        }
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, other: Self) {
        self.one |= other.one;
        self.two |= other.two;
        self.three |= other.three;
        self.four |= other.four;
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Bitboard {
            one: !self.one,
            two: !self.two,
            three: !self.three,
            four: !self.four
        }
    }
}

impl Shl<u8> for Bitboard {
    type Output = Self;

    fn shl(self, n: u8) -> Self::Output {
        Bitboard {
            one: (self.one << n) | (self.two >> (128 - n)),
            two: (self.two << n) | (self.three >> (128 - n)),
            three: (self.three << n) | (self.four >> (128 - n)),
            four: self.four << n
        }
    }
}

impl ShlAssign<u8> for Bitboard {
    fn shl_assign(&mut self, n: u8) {
        self.one = (self.one << n) | (self.two >> (128 - n));
        self.two = (self.two << n) | (self.three >> (128 - n));
        self.three = (self.three << n) | (self.four >> (128 - n));
        self.four = self.four << n;
    }
}

impl Shr<u8> for Bitboard {
    type Output = Self;

    fn shr(self, n: u8) -> Self::Output {
        Bitboard {
            one: self.one >> n,
            two: (self.two >> n) | (self.one << (128 - n)),
            three: (self.three >> n) | (self.two << (128 - n)),
            four: (self.four >> n) | self.three << (128 - n)
        }
    }
}

impl ShrAssign<u8> for Bitboard {
    fn shr_assign(&mut self, n: u8) {
        self.four = (self.four >> n) | self.three << (128 - n);
        self.three = (self.three >> n) | (self.two << (128 - n));
        self.two = (self.two >> n) | (self.one << (128 - n));
        self.one >>= n;
    }
}

impl PartialEq for Bitboard {
    fn eq(&self, other: &Self) -> bool {
        return self.one == other.one && self.two == other.two
            && self.three == other.three && self.four == other.four
    }
}
