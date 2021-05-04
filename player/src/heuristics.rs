use game_sdk::{Action, ActionList, Bitboard, GameState, Player};
use game_sdk::{START_FIELDS, VALID_FIELDS};
/*
pub struct GameHeuristic {
    pub state: GameState,
    pub occupied: Bitboard,
    pub placement_fields: [Bitboard; 4],
    pub reachable_fields: [Bitboard; 4],
    pub blockable_placement_fields: [Bitboard; 4],
    pub controlled_fields: [Bitboard; 4],
    pub leaks: [Bitboard; 4],
}

impl GameHeuristic {
    pub fn for_state(state: &GameState) -> Self {
        let mut heuristic = Self {
            state: state.clone(),
            occupied: state.get_occupied_fields(),
            placement_fields: [Bitboard::empty(); 4],
            reachable_fields: [Bitboard::empty(); 4],
            blockable_placement_fields: [Bitboard::empty(); 4],
            controlled_fields: [Bitboard::empty(); 4],
            leaks: [Bitboard::empty(); 4],
        };
        heuristic.get_placement_fields();
        heuristic.get_reachable_fields();
        heuristic.get_blockable_placement_fields();
        heuristic.get_controlled_fields();
        heuristic.get_leaks();
        heuristic
    }

    fn get_placement_fields(&mut self) {
        for color in 0..4 {
            let current_color_fields = self.state.board[color];
            let other_color_fields = self.occupied & !current_color_fields;
            let legal_fields = !(self.occupied | current_color_fields.neighbours()) & VALID_FIELDS;
            self.placement_fields[color] = if self.state.ply > 3 {
                current_color_fields.diagonal_neighbours() & legal_fields
            } else {
                START_FIELDS & !other_color_fields
            };
            //println!("{}", self.placement_fields[color]);
        }
    }

    fn get_reachable_fields(&mut self) {
        for color in 0..4 {
            let mut reachable_fields = self.placement_fields[color];
            let neighbours = self.state.board[color].neighbours();
            for _ in 0..4 {
                reachable_fields |= reachable_fields.neighbours() & !(self.occupied | neighbours);
            }
            self.reachable_fields[color] = reachable_fields;
        }
    }

    fn get_blockable_placement_fields(&mut self) {
        for color in 0..4 {
            self.blockable_placement_fields[color] = self.reachable_fields[color]
                & (self.placement_fields[(color + 1) & 0b11]
                    | self.placement_fields[(color + 3) & 0b11]);
        }
    }

    fn get_controlled_fields(&mut self) {
        for color in 0..4 {
            let mut controlled_fields = self.reachable_fields[color];
            for c in 0..4 {
                if c != color {
                    controlled_fields &= !self.reachable_fields[c];
                }
            }
            self.controlled_fields[color] = controlled_fields;
        }
    }

    fn get_leaks(&mut self) {
        for color in 0..4 {
            let mut possible_leaks = self.controlled_fields[color]
                & (self.occupied & !self.state.board[color]).neighbours();
            let mut leaks = Bitboard::empty();
            while possible_leaks.not_zero() {
                let bit_index = possible_leaks.trailing_zeros();
                let mut possible_leak = Bitboard::bit(bit_index);
                possible_leaks ^= possible_leak;
                possible_leak = possible_leak.diagonal_neighbours()
                    & !(self.reachable_fields[color] | self.occupied);
                if possible_leak.not_zero()
                    && (possible_leak.neighbours() & (self.occupied & !self.state.board[color]))
                        .not_zero()
                {
                    leaks.flip_bit(bit_index);
                }
            }
            self.leaks[color] = leaks;
        }
    }

    pub fn evaluate_action(&self, action: Action) -> f32 {
        if action.is_skip() {
            return 0.;
        }
        let color = self.state.get_current_color();
        let piece =
            Bitboard::with_piece(action.get_destination(), action.get_shape() as usize);

        (piece & self.blockable_placement_fields[color]).count_ones() as f32
            + (piece & self.leaks[color]).count_ones() as f32 * 0.65
            + piece.count_ones() as f32 * 1.
            + (piece & self.placement_fields[2]).count_ones() as f32 * 0.5
            + (piece & self.placement_fields[0]).count_ones() as f32 * 0.5

        //+ ((piece | piece.diagonal_neighbours()) & (self.leaks[1] | self.leaks[3])).count_ones() as f32 * -0.05
        //+ (piece & self.leaks[2]).count_ones() as f32 * 0.2
    }
}
*/

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

    pub fn evaluate_action(&self, action: Action) -> f32 {
        if action.is_skip() {
            return 1.0;
        }
        let color = self.state.get_current_color();
        let next_opponent_color = (color + 1) & 0b11;
        let second_color = (color + 2) & 0b11;
        let last_opponent_color = (color + 3) & 0b11;
        let piece = Bitboard::with_piece(action.get_destination(), action.get_shape() as usize);
        let mut value;
        value = (piece & self.leaks[color]).count_ones() as f32 * 0.8;
        value += (piece & self.blockable_placement_fields[color]).count_ones() as f32;
        value += piece.count_ones() as f32;
        value += (piece
            & (self.blockable_placement_fields[next_opponent_color]
                | self.blockable_placement_fields[last_opponent_color])
                .diagonal_neighbours())
        .count_ones() as f32
            * 0.8;
        value += (piece & self.leaks[next_opponent_color] | self.leaks[last_opponent_color])
            .diagonal_neighbours()
            .count_ones() as f32
            * 0.2;
        let new_placement_fields =
            piece.diagonal_neighbours() & !(piece | self.state.board[color]).neighbours();
        value += new_placement_fields.count_ones() as f32 * 0.1;
        value += (piece & (self.leaks[next_opponent_color] | self.leaks[last_opponent_color]))
            .count_ones() as f32
            * 2.5;
        value -= (piece & self.placement_fields[second_color]).count_ones() as f32 * 0.3;
        //print!("{}, ", value);
        value
    }
}

pub struct HeuristicPlayer {
    al: ActionList,
}

impl Player for HeuristicPlayer {
    fn on_move_request(&mut self, state: &GameState) -> Action {
        let h = Heuristic::for_state(state);
        state.get_possible_actions(&mut self.al);
        let mut best_action = self.al[0];
        let mut best_value = std::f32::NEG_INFINITY;
        for i in 0..self.al.size {
            let action = self.al[i];
            if action.is_skip() {
                return action;
            }
            let heuristic_value = h.evaluate_action(action);
            if heuristic_value > best_value {
                best_value = heuristic_value;
                best_action = action;
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
