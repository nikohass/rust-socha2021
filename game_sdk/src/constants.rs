use super::Bitboard;

pub const VALID_FIELDS: Bitboard = Bitboard(
    34359721983,
    337623909661717427026139553986326233087,
    329648537884054317714434393650000297983,
    297747050773401880467613752304696557567,
);

pub const COLUMN_MASK: Bitboard = Bitboard(
    32768,
    5316914518442072874470106890883956736,
    21267658073768291497880427563535826944,
    85070632295073165991521710254143307777,
);

pub const ROW_MASK: Bitboard = Bitboard(0, 0, 0, 1048575);

pub const START_FIELDS: Bitboard = Bitboard(1 << 34 | 1 << 15, 0, 0, 1 | 1 << 19);

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

pub const PENTOMINO_SHAPES: [usize; 63] = [
    7, 8, 10, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 43, 44, 45, 46, 47, 48, 49, 50, 51,
    52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75,
    76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90,
];

// rotation, flipped
pub const PIECE_ORIENTATIONS: [(u8, bool); 91] = [
    (0, false),
    (0, false),
    (1, false),
    (1, false),
    (0, false),
    (1, false),
    (0, false),
    (1, false),
    (0, false),
    (0, false), // O-Tetromino
    (0, false), // X-Pentomino
    (0, false), // L-Tromino
    (1, false),
    (2, false),
    (3, false),
    (2, true), // L-Tetromino
    (2, false),
    (0, true),
    (0, false),
    (3, true),
    (1, false),
    (1, true),
    (3, false),
    (1, true), // L-Pentomino
    (1, false),
    (3, true),
    (3, false),
    (2, true),
    (0, false),
    (2, false),
    (0, true),
    (0, false), // T-Pentomino
    (2, false),
    (3, false),
    (1, false),
    (0, false), // T-Tetromino
    (2, false),
    (3, false),
    (1, false),
    (0, true), // Z-Tetromino
    (0, false),
    (3, false),
    (1, true),
    (3, true), // Z-Pentomino
    (3, false),
    (0, true),
    (0, false),
    (2, false), // U-Pentomino
    (0, false),
    (1, false),
    (3, false),
    (1, true), // F-Pentomino
    (1, false),
    (3, true),
    (3, false),
    (0, false),
    (0, true),
    (2, true),
    (2, false),
    (0, false), // W-Pentomino
    (3, false),
    (2, false),
    (1, false),
    (3, true), // N-Pentomino
    (3, false),
    (1, true),
    (1, false),
    (2, false),
    (0, true),
    (2, true),
    (0, false),
    (1, false), // V-Pentomino
    (3, false),
    (2, false),
    (0, false),
    (0, false), // P-Pentomino
    (0, true),
    (3, false),
    (1, true),
    (1, false),
    (3, true),
    (2, false),
    (2, true),
    (0, true), // Y-Pentomino
    (2, false),
    (2, true),
    (0, false),
    (3, true),
    (3, false),
    (1, false),
    (1, true),
];
