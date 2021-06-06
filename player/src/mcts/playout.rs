use super::rave::RaveTable;
use game_sdk::{Action, Bitboard, GameState, PieceType};
use game_sdk::{START_FIELDS, VALID_FIELDS};
use rand::{rngs::SmallRng, RngCore};

type ShapeFunction = fn(Bitboard, Bitboard) -> Bitboard;
const MOVEGEN_RETRIES: usize = 40;

pub fn result_to_value(result: i16) -> f32 {
    // Returns 1 if team Blue/Red won, 0 if team Yellow/Green won, and 0.5 if the game ended in a draw
    let abs = result.abs() as f32 / 100_000.; // Encourages the player to win with a large score difference
    match result {
        r if r > 0 => 0.999 + abs,
        r if r < 0 => 0.001 - abs,
        _ => 0.5,
    }
}

pub fn playout(state: &mut GameState, rng: &mut SmallRng, rave_table: &mut RaveTable) -> f32 {
    // Plays a game recursively to the end, returns the results and adds the values to the RaveTable
    if state.is_game_over() {
        let result = state.game_result();
        result_to_value(result)
    } else {
        let color = state.get_current_color();
        let action = random_action(&state, rng, state.ply < 12);
        state.do_action(action);
        let result = playout(state, rng, rave_table);
        rave_table.add_value(action, color, result);
        result
    }
}

pub fn random_action(state: &GameState, rng: &mut SmallRng, pentomino_only: bool) -> Action {
    let color = state.get_current_color();
    if state.has_color_skipped(color) {
        return Action::SKIP;
    }
    // Fields that are occupied by the current color
    let own_fields = state.board[color];
    // All fields that are occupied by the other colors
    let other_fields = state.get_occupied_fields() & !own_fields;
    // Fields that newly placed pieces can occupy
    let legal_fields = !(own_fields | other_fields | own_fields.neighbours()) & VALID_FIELDS;
    // Calculate the corners of existing pieces at which new pieces can be placed
    let p = if state.ply > 3 {
        own_fields.diagonal_neighbours() & legal_fields
    } else {
        START_FIELDS & !other_fields
    };
    if p.is_empty() {
        return Action::SKIP;
    }
    for _ in 0..MOVEGEN_RETRIES {
        // Select a random shape
        let shape = if pentomino_only {
            PENTOMINO_SHAPES[(rng.next_u64() % 63) as usize]
        } else {
            (rng.next_u32() % 91) as usize
        };
        if state.pieces_left[PieceType::from_shape(shape) as usize][color] {
            // Generate all possible destination for this shape
            let mut destinations = SHAPE_FUNCTIONS[shape](legal_fields, p);
            if destinations.not_empty() {
                // Return an action with one of the possible destinations
                return Action::set(destinations.random_field(rng), shape as u16);
            }
        }
    }
    Action::SKIP
}

// All these functions take legal_fields and placement_fields as an argument and return a bitboard with all possible destinations for the shape
// generated with rust-socha2021/helper_scripts/shapes.py

fn shape_0(_l: Bitboard, p: Bitboard) -> Bitboard {
    p
}

fn shape_1(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1) & (p | p >> 1)
}

fn shape_2(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21) & (p | p >> 21)
}

fn shape_3(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 2) & (p | p >> 2)
}

fn shape_4(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21 & l >> 42) & (p | p >> 42)
}

fn shape_5(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 2 & l >> 3) & (p | p >> 3)
}

fn shape_6(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21 & l >> 42 & l >> 63) & (p | p >> 63)
}

fn shape_7(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 2 & l >> 3 & l >> 4) & (p | p >> 4)
}

fn shape_8(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21 & l >> 42 & l >> 63 & l >> 84) & (p | p >> 84)
}

fn shape_9(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 21 & l >> 22) & (p | p >> 1 | p >> 21 | p >> 22)
}

fn shape_10(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 21 & l >> 22 & l >> 23 & l >> 43) & (p >> 1 | p >> 21 | p >> 23 | p >> 43)
}

fn shape_11(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21 & l >> 22) & (p | p >> 21 | p >> 22)
}

fn shape_12(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 21) & (p | p >> 1 | p >> 21)
}

fn shape_13(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 22) & (p | p >> 1 | p >> 22)
}

fn shape_14(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 21 & l >> 22) & (p >> 1 | p >> 21 | p >> 22)
}

fn shape_15(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 21 & l >> 42) & (p | p >> 1 | p >> 42)
}

fn shape_16(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 22 & l >> 43) & (p | p >> 1 | p >> 43)
}

fn shape_17(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 22 & l >> 42 & l >> 43) & (p >> 1 | p >> 42 | p >> 43)
}

fn shape_18(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21 & l >> 42 & l >> 43) & (p | p >> 42 | p >> 43)
}

fn shape_19(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21 & l >> 22 & l >> 23) & (p | p >> 21 | p >> 23)
}

fn shape_20(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 2 & l >> 21) & (p | p >> 2 | p >> 21)
}

fn shape_21(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 2 & l >> 23) & (p | p >> 2 | p >> 23)
}

fn shape_22(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 2 & l >> 21 & l >> 22 & l >> 23) & (p >> 2 | p >> 21 | p >> 23)
}

fn shape_23(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 2 & l >> 3 & l >> 24) & (p | p >> 3 | p >> 24)
}

fn shape_24(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 2 & l >> 3 & l >> 21) & (p | p >> 3 | p >> 21)
}

fn shape_25(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21 & l >> 22 & l >> 23 & l >> 24) & (p | p >> 21 | p >> 24)
}

fn shape_26(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 3 & l >> 21 & l >> 22 & l >> 23 & l >> 24) & (p >> 3 | p >> 21 | p >> 24)
}

fn shape_27(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 21 & l >> 42 & l >> 63) & (p | p >> 1 | p >> 63)
}

fn shape_28(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21 & l >> 42 & l >> 63 & l >> 64) & (p | p >> 63 | p >> 64)
}

fn shape_29(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 22 & l >> 43 & l >> 64) & (p | p >> 1 | p >> 64)
}

fn shape_30(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 22 & l >> 43 & l >> 63 & l >> 64) & (p >> 1 | p >> 63 | p >> 64)
}

fn shape_31(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 2 & l >> 22 & l >> 43) & (p | p >> 2 | p >> 43)
}

fn shape_32(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 22 & l >> 42 & l >> 43 & l >> 44) & (p >> 1 | p >> 42 | p >> 44)
}

fn shape_33(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21 & l >> 22 & l >> 23 & l >> 42) & (p | p >> 23 | p >> 42)
}

fn shape_34(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 2 & l >> 21 & l >> 22 & l >> 23 & l >> 44) & (p >> 2 | p >> 21 | p >> 44)
}

fn shape_35(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 2 & l >> 22) & (p | p >> 2 | p >> 22)
}

fn shape_36(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 21 & l >> 22 & l >> 23) & (p >> 1 | p >> 21 | p >> 23)
}

fn shape_37(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21 & l >> 22 & l >> 42) & (p | p >> 22 | p >> 42)
}

fn shape_38(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 21 & l >> 22 & l >> 43) & (p >> 1 | p >> 21 | p >> 43)
}

fn shape_39(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 2 & l >> 21 & l >> 22) & (p >> 1 | p >> 2 | p >> 21 | p >> 22)
}

fn shape_40(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 22 & l >> 23) & (p | p >> 1 | p >> 22 | p >> 23)
}

fn shape_41(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 21 & l >> 22 & l >> 42) & (p >> 1 | p >> 21 | p >> 22 | p >> 42)
}

fn shape_42(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21 & l >> 22 & l >> 43) & (p | p >> 21 | p >> 22 | p >> 43)
}

fn shape_43(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21 & l >> 22 & l >> 23 & l >> 44) & (p | p >> 21 | p >> 23 | p >> 44)
}

fn shape_44(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 2 & l >> 21 & l >> 22 & l >> 23 & l >> 42) & (p >> 2 | p >> 21 | p >> 23 | p >> 42)
}

fn shape_45(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 2 & l >> 22 & l >> 42 & l >> 43) & (p >> 1 | p >> 2 | p >> 42 | p >> 43)
}

fn shape_46(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 22 & l >> 43 & l >> 44) & (p | p >> 1 | p >> 43 | p >> 44)
}

fn shape_47(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 2 & l >> 21 & l >> 23) & (p | p >> 2 | p >> 21 | p >> 23)
}

fn shape_48(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 2 & l >> 21 & l >> 22 & l >> 23) & (p | p >> 2 | p >> 21 | p >> 23)
}

fn shape_49(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 21 & l >> 42 & l >> 43) & (p | p >> 1 | p >> 42 | p >> 43)
}

fn shape_50(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 22 & l >> 42 & l >> 43) & (p | p >> 1 | p >> 42 | p >> 43)
}

fn shape_51(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 22 & l >> 23 & l >> 42 & l >> 43) & (p >> 1 | p >> 23 | p >> 42 | p >> 43)
}

fn shape_52(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 21 & l >> 22 & l >> 43 & l >> 44) & (p >> 1 | p >> 21 | p >> 43 | p >> 44)
}

fn shape_53(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 2 & l >> 21 & l >> 22 & l >> 43) & (p >> 1 | p >> 2 | p >> 21 | p >> 43)
}

fn shape_54(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 22 & l >> 23 & l >> 43) & (p | p >> 1 | p >> 23 | p >> 43)
}

fn shape_55(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 2 & l >> 21 & l >> 22 & l >> 23 & l >> 43) & (p >> 2 | p >> 21 | p >> 23 | p >> 43)
}

fn shape_56(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21 & l >> 22 & l >> 23 & l >> 43) & (p | p >> 21 | p >> 23 | p >> 43)
}

fn shape_57(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 21 & l >> 22 & l >> 23 & l >> 44) & (p >> 1 | p >> 21 | p >> 23 | p >> 44)
}

fn shape_58(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 21 & l >> 22 & l >> 23 & l >> 42) & (p >> 1 | p >> 21 | p >> 23 | p >> 42)
}

fn shape_59(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21 & l >> 22 & l >> 43 & l >> 44) & (p | p >> 21 | p >> 22 | p >> 43 | p >> 44)
}

fn shape_60(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 2 & l >> 22 & l >> 23 & l >> 42 & l >> 43)
        & (p >> 2 | p >> 22 | p >> 23 | p >> 42 | p >> 43)
}
fn shape_61(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 22 & l >> 23 & l >> 44) & (p | p >> 1 | p >> 22 | p >> 23 | p >> 44)
}

fn shape_62(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 2 & l >> 21 & l >> 22 & l >> 42)
        & (p >> 1 | p >> 2 | p >> 21 | p >> 22 | p >> 42)
}
fn shape_63(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 22 & l >> 42 & l >> 43 & l >> 63) & (p >> 1 | p >> 42 | p >> 43 | p >> 63)
}

fn shape_64(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21 & l >> 42 & l >> 43 & l >> 64) & (p | p >> 42 | p >> 43 | p >> 64)
}

fn shape_65(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 21 & l >> 22 & l >> 42 & l >> 63) & (p >> 1 | p >> 21 | p >> 22 | p >> 63)
}

fn shape_66(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21 & l >> 22 & l >> 43 & l >> 64) & (p | p >> 21 | p >> 22 | p >> 64)
}

fn shape_67(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 2 & l >> 3 & l >> 21 & l >> 22 & l >> 23) & (p >> 2 | p >> 3 | p >> 21 | p >> 23)
}

fn shape_68(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 2 & l >> 23 & l >> 24) & (p | p >> 2 | p >> 23 | p >> 24)
}

fn shape_69(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 22 & l >> 23 & l >> 24) & (p | p >> 1 | p >> 22 | p >> 24)
}

fn shape_70(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 2 & l >> 3 & l >> 21 & l >> 22) & (p >> 1 | p >> 3 | p >> 21 | p >> 22)
}

fn shape_71(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 2 & l >> 21 & l >> 42) & (p | p >> 2 | p >> 42)
}

fn shape_72(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 2 & l >> 23 & l >> 42 & l >> 43 & l >> 44) & (p >> 2 | p >> 42 | p >> 44)
}

fn shape_73(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 2 & l >> 23 & l >> 44) & (p | p >> 2 | p >> 44)
}

fn shape_74(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21 & l >> 42 & l >> 43 & l >> 44) & (p | p >> 42 | p >> 44)
}

fn shape_75(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 21 & l >> 22 & l >> 42) & (p | p >> 1 | p >> 22 | p >> 42)
}

fn shape_76(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 21 & l >> 22 & l >> 43) & (p | p >> 1 | p >> 21 | p >> 43)
}

fn shape_77(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 21 & l >> 22 & l >> 23) & (p | p >> 1 | p >> 21 | p >> 23)
}

fn shape_78(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 2 & l >> 21 & l >> 22) & (p | p >> 2 | p >> 21 | p >> 22)
}

fn shape_79(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 2 & l >> 22 & l >> 23) & (p | p >> 2 | p >> 22 | p >> 23)
}

fn shape_80(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 2 & l >> 21 & l >> 22 & l >> 23) & (p >> 1 | p >> 2 | p >> 21 | p >> 23)
}

fn shape_81(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 21 & l >> 22 & l >> 42 & l >> 43) & (p >> 1 | p >> 21 | p >> 42 | p >> 43)
}

fn shape_82(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21 & l >> 22 & l >> 42 & l >> 43) & (p | p >> 22 | p >> 42 | p >> 43)
}

fn shape_83(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21 & l >> 22 & l >> 42 & l >> 63) & (p | p >> 22 | p >> 63)
}

fn shape_84(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 21 & l >> 42 & l >> 43 & l >> 63) & (p | p >> 43 | p >> 63)
}

fn shape_85(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 22 & l >> 42 & l >> 43 & l >> 64) & (p >> 1 | p >> 42 | p >> 64)
}

fn shape_86(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 21 & l >> 22 & l >> 43 & l >> 64) & (p >> 1 | p >> 21 | p >> 64)
}

fn shape_87(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 2 & l >> 3 & l >> 23) & (p | p >> 3 | p >> 23)
}

fn shape_88(l: Bitboard, p: Bitboard) -> Bitboard {
    (l & l >> 1 & l >> 2 & l >> 3 & l >> 22) & (p | p >> 3 | p >> 22)
}

fn shape_89(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 2 & l >> 21 & l >> 22 & l >> 23 & l >> 24) & (p >> 2 | p >> 21 | p >> 24)
}

fn shape_90(l: Bitboard, p: Bitboard) -> Bitboard {
    (l >> 1 & l >> 21 & l >> 22 & l >> 23 & l >> 24) & (p >> 1 | p >> 21 | p >> 24)
}

const SHAPE_FUNCTIONS: [ShapeFunction; 91] = [
    shape_0, shape_1, shape_2, shape_3, shape_4, shape_5, shape_6, shape_7, shape_8, shape_9,
    shape_10, shape_11, shape_12, shape_13, shape_14, shape_15, shape_16, shape_17, shape_18,
    shape_19, shape_20, shape_21, shape_22, shape_23, shape_24, shape_25, shape_26, shape_27,
    shape_28, shape_29, shape_30, shape_31, shape_32, shape_33, shape_34, shape_35, shape_36,
    shape_37, shape_38, shape_39, shape_40, shape_41, shape_42, shape_43, shape_44, shape_45,
    shape_46, shape_47, shape_48, shape_49, shape_50, shape_51, shape_52, shape_53, shape_54,
    shape_55, shape_56, shape_57, shape_58, shape_59, shape_60, shape_61, shape_62, shape_63,
    shape_64, shape_65, shape_66, shape_67, shape_68, shape_69, shape_70, shape_71, shape_72,
    shape_73, shape_74, shape_75, shape_76, shape_77, shape_78, shape_79, shape_80, shape_81,
    shape_82, shape_83, shape_84, shape_85, shape_86, shape_87, shape_88, shape_89, shape_90,
];

const PENTOMINO_SHAPES: [usize; 63] = [
    7, 8, 10, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 43, 44, 45, 46, 47, 48, 49, 50, 51,
    52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75,
    76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90,
];

#[cfg(test)]
mod test {
    use super::random_action;
    use game_sdk::{Action, ActionList, GameState};
    use rand::{rngs::SmallRng, SeedableRng};

    #[test]
    fn test_random_actions() {
        let mut al = ActionList::default();
        let mut rng = SmallRng::from_entropy();
        for _ in 0..300 {
            let mut state = GameState::random();
            let mut action = Action::SKIP;
            state.get_possible_actions(&mut al);
            for _ in 0..100 {
                action = random_action(&state, &mut rng, false);
                let mut is_legal = false;
                for i in 0..al.size {
                    if action == al[i] {
                        is_legal = true;
                        break;
                    }
                }
                if !is_legal {
                    panic!("Invalid action");
                }
            }
            while al.size > 0 {
                action = random_action(&state, &mut rng, false);
                for i in 0..al.size {
                    if al[i] == action {
                        al.swap(i, al.size - 1);
                        al.size -= 1;
                        break;
                    }
                }
            }
            state.do_action(action);
        }
    }
}
