use super::node::Node;
use game_sdk::{Action, ActionList, Bitboard, GameState, PieceType, Player};
use game_sdk::{START_FIELDS, VALID_FIELDS};

pub const SEARCH_SEEDING_VISITS: f32 = 18.;
pub const N_PARAMS: usize = 11;
pub const DEFAULT_HEURISTIC_PARAMETERS: [f32; N_PARAMS] = [
    0.06481217,
    0.03788412,
    0.0128481835,
    0.034261946,
    0.014906124,
    0.03253359,
    0.02333225,
    0.027018376,
    0.026275534,
    0.01905919,
    0.027598862,
];
pub const BIAS: f32 = 0.052122455;

fn calculate_placement_fields(state: &GameState, occupied: &Bitboard) -> [Bitboard; 4] {
    // Calculate the corners at which each color can place new pieces
    let mut placement_fields: [Bitboard; 4] = [Bitboard::empty(); 4];
    #[allow(clippy::needless_range_loop)]
    for color in 0..4 {
        let current_color_fields = state.board[color];
        let other_colors_fields = *occupied & !current_color_fields;
        let legal_fields = !(*occupied | current_color_fields.neighbours()) & VALID_FIELDS;
        placement_fields[color] = if state.ply > 3 {
            current_color_fields.diagonal_neighbours() & legal_fields
        } else {
            START_FIELDS & !other_colors_fields
        };
    }
    placement_fields
}

fn estimate_reachable_fields(
    state: &GameState,
    placement_fields: &[Bitboard; 4],
    occupied: &Bitboard,
) -> [Bitboard; 4] {
    // Estimate the area that each color can reach
    let mut reachable_fields: [Bitboard; 4] = [Bitboard::empty(); 4];
    for color in 0..4 {
        let mut reachable = placement_fields[color];
        let unreachable = state.board[color].neighbours() | *occupied;
        for _ in 0..4 {
            reachable |= reachable.neighbours() & !unreachable;
        }
        reachable_fields[color] = reachable;
    }
    reachable_fields
}

fn calculate_leaks(
    state: &GameState,
    placement_fields: &[Bitboard; 4],
    reachable_fields: &[Bitboard; 4],
    occupied: &Bitboard,
) -> [Bitboard; 4] {
    // Calculate fields that help the color to 'leak' into areas that it couldn't reach before
    let mut leaks: [Bitboard; 4] = [Bitboard::empty(); 4];
    for color in 0..4 {
        leaks[color] = reachable_fields[color]
            & (placement_fields[color]
                & (*occupied).neighbours()
                & !(*occupied | state.board[color].neighbours()))
            .diagonal_neighbours()
            & (*occupied).neighbours();
    }
    leaks
}

pub fn expand_node(
    node: &mut Node,
    state: &GameState,
    al: &mut ActionList,
    params: &[f32; N_PARAMS],
) {
    let current_color = state.get_current_color();
    let next_opponent_color = (current_color + 1) & 0b11;
    let second_color = (current_color + 2) & 0b11;
    let last_opponent_color = (current_color + 3) & 0b11;
    let occupied = state.get_occupied_fields();

    let placement_fields = calculate_placement_fields(state, &occupied);
    let reachable_fields = estimate_reachable_fields(state, &placement_fields, &occupied);
    let leaks = calculate_leaks(state, &placement_fields, &reachable_fields, &occupied);
    // All placement fields of the opponent colors
    let opponent_placement_fields =
        placement_fields[next_opponent_color] | placement_fields[last_opponent_color];
    // All fields that the opponent can reach in the next round
    let opponent_reachable_fields =
        reachable_fields[next_opponent_color] | reachable_fields[last_opponent_color];

    let k = reachable_fields[current_color]
        & (occupied & !state.board[current_color]).neighbours()
        & !(occupied & !state.board[current_color]).diagonal_neighbours();

    for i in 0..al.size {
        let action = al[i];
        let shape = action.get_shape() as usize;
        let destination = action.get_destination();
        let piece_type = PieceType::from_shape(shape);
        let piece_size = piece_type.piece_size();
        if state.ply < 8 && piece_size < 5 {
            // Ignore small pieces in the first two rounds
            continue;
        }
        let piece = Bitboard::with_piece(destination, shape);
        let mut heuristic_value = piece_size as f32 * params[0];
        // Evaluate leaks
        heuristic_value += (piece & leaks[current_color]).count_ones() as f32 * params[1];
        heuristic_value += (piece
            & leaks[current_color].diagonal_neighbours()
            & !(opponent_reachable_fields | occupied))
            .count_ones() as f32
            * params[2];
        heuristic_value += (piece & leaks[next_opponent_color])
            .diagonal_neighbours()
            .count_ones() as f32
            * params[3];
        heuristic_value += (piece & leaks[last_opponent_color])
            .diagonal_neighbours()
            .count_ones() as f32
            * params[4];
        // Evaluate blocks
        heuristic_value += (piece & opponent_placement_fields).count_ones() as f32 * params[5];
        // Calculate all new placement fields the piece would create
        let new_placement_fields = piece.diagonal_neighbours()
            & !(piece | state.board[current_color]).neighbours()
            & !occupied;
        // Evaluate the new placement fields
        heuristic_value += (new_placement_fields & reachable_fields[next_opponent_color])
            .count_ones() as f32
            * params[6];
        heuristic_value += (new_placement_fields & reachable_fields[last_opponent_color])
            .count_ones() as f32
            * params[7];
        heuristic_value += new_placement_fields.count_ones() as f32 * params[8];
        heuristic_value += (piece & placement_fields[second_color]).count_ones() as f32 * params[9];
        heuristic_value += (piece & k).count_ones() as f32 * params[10];
        node.children.push(Node {
            children: Vec::new(),
            action,
            n: SEARCH_SEEDING_VISITS,
            q: (heuristic_value + BIAS) * SEARCH_SEEDING_VISITS,
        })
    }
}

pub struct HeuristicPlayer {
    al: ActionList,
}

impl Player for HeuristicPlayer {
    fn on_move_request(&mut self, state: &GameState) -> Action {
        let mut node = Node::empty();
        state.get_possible_actions(&mut self.al);
        if self.al[0].is_skip() {
            return Action::SKIP;
        }
        node.children = Vec::with_capacity(self.al.size);
        expand_node(
            &mut node,
            state,
            &mut self.al,
            &DEFAULT_HEURISTIC_PARAMETERS,
        );
        let mut best_action = self.al[0];
        let mut best_value = std::f32::NEG_INFINITY;
        for child_node in node.children.iter() {
            let heuristic_value = child_node.get_value();
            if heuristic_value > best_value {
                best_value = heuristic_value;
                best_action = child_node.action;
            }
        }
        best_action
    }
}

impl Default for HeuristicPlayer {
    fn default() -> Self {
        Self {
            al: ActionList::default(),
        }
    }
}
