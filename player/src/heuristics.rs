use super::mcts::Node;
use game_sdk::{Action, ActionList, Bitboard, GameState, Player};
use game_sdk::{START_FIELDS, VALID_FIELDS};

const LEAK_FACTOR: f32 = 1.0;
const BLOCKABLE_PLACEMENT_FIELDS_FACTOR: f32 = 1.2;
const PIECE_SIZE_FACTOR: f32 = 1.0;
const OPPONENT_BLOCKABLE_PLACEMENT_FIELDS_FACTOR: f32 = 0.8;
const BLOCKED_OPPONENT_LEAKS: f32 = 1.5;
const NEW_PLACEMENT_FIELDS_FACTOR: f32 = 1.1;
const SECOND_COLOR_BLOCK_FACTOR: f32 = -0.3;

pub struct Heuristic {
    state: GameState,
    leaks: [Bitboard; 4],
    placement_fields: [Bitboard; 4],
    blockable_placement_fields: [Bitboard; 4],
}

impl Heuristic {
    pub fn for_state(state: &GameState) -> Self {
        let occupied = state.get_occupied_fields();
        let mut placement_fields: [Bitboard; 4] = [Bitboard::empty(); 4];
        for color in 0..4 {
            let current_color_fields = state.board[color];
            let other_color_fields = occupied & !current_color_fields;
            let legal_fields = !(occupied | current_color_fields.neighbours()) & VALID_FIELDS;
            placement_fields[color] = if state.ply > 3 {
                current_color_fields.diagonal_neighbours() & legal_fields
            } else {
                START_FIELDS & !other_color_fields
            };
        }
        let mut reachable_fields: [Bitboard; 4] = [Bitboard::empty(); 4];
        for color in 0..4 {
            let mut reachable = placement_fields[color];
            let unreachable = state.board[color].neighbours() | occupied;
            for _ in 0..4 {
                reachable |= reachable.neighbours() & !unreachable;
            }
            reachable_fields[color] = reachable;
        }
        let mut blockable_placement_fields: [Bitboard; 4] = [Bitboard::empty(); 4];
        for color in 0..4 {
            blockable_placement_fields[color] = reachable_fields[color]
                & (placement_fields[(color + 1) & 0b11] | placement_fields[(color + 3) & 0b11]);
        }
        let mut leaks: [Bitboard; 4] = [Bitboard::empty(); 4];
        for color in 0..4 {
            leaks[color] = reachable_fields[color]
                & (placement_fields[color]
                    & occupied.neighbours()
                    & !(occupied | state.board[color].neighbours()))
                .diagonal_neighbours()
                & occupied.neighbours();
        }
        Heuristic {
            state: state.clone(),
            leaks,
            placement_fields,
            blockable_placement_fields,
        }
    }

    /*pub fn evaluate_action(&self, action: Action) -> f32 {
        if action.is_skip() {
            return 0.0;
        }
        let color = self.state.get_current_color();
        let next_opponent_color = (color + 1) & 0b11;
        let second_color = (color + 2) & 0b11;
        let last_opponent_color = (color + 3) & 0b11;
        let piece = Bitboard::with_piece(action.get_destination(), action.get_shape() as usize);
        let mut value;
        value = (piece & self.leaks[color]).count_ones() as f32 * LEAK_FACTOR;
        value += (piece & self.leaks[next_opponent_color] | self.leaks[last_opponent_color])
            .diagonal_neighbours()
            .count_ones() as f32
            * BLOCKED_OPPONENT_LEAKS;

        value += piece.count_ones() as f32 * PIECE_SIZE_FACTOR;

        value += (piece & self.blockable_placement_fields[color]).count_ones() as f32
            * BLOCKABLE_PLACEMENT_FIELDS_FACTOR;
        value += (piece
            & (self.blockable_placement_fields[next_opponent_color]
                | self.blockable_placement_fields[last_opponent_color])
                .diagonal_neighbours())
        .count_ones() as f32
            * OPPONENT_BLOCKABLE_PLACEMENT_FIELDS_FACTOR;

        let new_placement_fields =
            piece.diagonal_neighbours() & !(piece | self.state.board[color]).neighbours();
        value += new_placement_fields.count_ones() as f32 * NEW_PLACEMENT_FIELDS_FACTOR;
        value += (piece & self.placement_fields[second_color]).count_ones() as f32
            * SECOND_COLOR_BLOCK_FACTOR;

        //print!("{}, ", value);
        value
    }*/

    pub fn expand_node(&self, al: &mut ActionList, node: &mut Node) {
        let color = self.state.get_current_color();
        let node_value = node.get_value();
        let next_opponent_color = (color + 1) & 0b11;
        let second_color = (color + 2) & 0b11;
        let last_opponent_color = (color + 3) & 0b11;
        for i in 0..al.size {
            let action = al[i];
            let piece = Bitboard::with_piece(action.get_destination(), action.get_shape() as usize);

            let mut heuristic_value = (piece & self.leaks[color]).count_ones() as f32 * LEAK_FACTOR;
            heuristic_value += (piece & self.leaks[next_opponent_color]
                | self.leaks[last_opponent_color])
                .diagonal_neighbours()
                .count_ones() as f32
                * BLOCKED_OPPONENT_LEAKS;

            heuristic_value += piece.count_ones() as f32 * PIECE_SIZE_FACTOR;

            heuristic_value += (piece & self.blockable_placement_fields[color]).count_ones() as f32
                * BLOCKABLE_PLACEMENT_FIELDS_FACTOR;
            heuristic_value += (piece
                & (self.blockable_placement_fields[next_opponent_color]
                    | self.blockable_placement_fields[last_opponent_color])
                    .diagonal_neighbours())
            .count_ones() as f32
                * OPPONENT_BLOCKABLE_PLACEMENT_FIELDS_FACTOR;

            let new_placement_fields =
                piece.diagonal_neighbours() & !(piece | self.state.board[color]).neighbours();
            heuristic_value +=
                new_placement_fields.count_ones() as f32 * NEW_PLACEMENT_FIELDS_FACTOR;
            heuristic_value += (piece & self.placement_fields[second_color]).count_ones() as f32
                * SECOND_COLOR_BLOCK_FACTOR;

            node.children.push(Node {
                children: Vec::new(),
                action,
                n: 10.,
                q: (1. - node_value + heuristic_value / 40.) * 10.,
            })
        }
    }
}

pub struct HeuristicPlayer {
    al: ActionList,
}

impl Player for HeuristicPlayer {
    fn on_move_request(&mut self, state: &GameState) -> Action {
        let heuristic = Heuristic::for_state(state);
        let mut node = Node::empty();
        state.get_possible_actions(&mut self.al);
        if self.al[0].is_skip() {
            return self.al[0];
        }
        node.children = Vec::with_capacity(self.al.size);
        heuristic.expand_node(&mut self.al, &mut node);
        let mut best_action = self.al[0];
        let mut best_value = std::f32::NEG_INFINITY;
        for child_node in node.children.iter() {
            let heuristic_value = child_node.get_value();
            if heuristic_value > best_value {
                best_value = heuristic_value;
                best_action = child_node.action;
            }
        }
        //let c = state.get_current_color();
        //println!("{}", c);
        //state.display_board(h.blockable_placement_fields[c]);
        //state.display_board(h.leaks[c]);
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
