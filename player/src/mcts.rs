use super::float_stuff::{ln, sqrt};
use game_sdk::{Action, ActionList, Bitboard, GameState, PieceType, Player};
use rand::{rngs::SmallRng, SeedableRng};
use std::time::Instant;

const C: f32 = 0.0;
const C_BASE: f32 = 7000.;
const C_FACTOR: f32 = 38.5;
const VISITS_BEFORE_EXPANSION: usize = 40;

pub fn rollout(state: &GameState, rng: &mut SmallRng) -> f32 {
    let team = state.get_team();
    let mut result = 0;
    let mut state = state.clone();
    while !state.is_game_over() {
        let color_index = state.get_current_color() as usize;
        match state.get_random_possible_action(rng, state.ply < 16, 40) {
            Action::Skip => {
                state.skipped |= 1 << color_index;
                result = state.game_result();
                if (state.has_team_one_skipped() && result < 0)
                    || (state.has_team_two_skipped() && result > 0)
                {
                    break;
                }
            }
            Action::Set(to, shape) => {
                let piece_type = PieceType::from_shape(shape);
                state.pieces_left[piece_type as usize][color_index] = false;
                state.board[color_index] ^= Bitboard::with_piece(to, shape);
                if piece_type == PieceType::Monomino {
                    state.monomino_placed_last |= 1 << color_index;
                } else {
                    state.monomino_placed_last &= !(1 << color_index);
                }
            }
        };
        state.ply += 1;
    }
    match result * team {
        r if r > 0 => 1.,
        r if r < 0 => 0.,
        _ => 0.5,
    }
}

pub struct Node {
    children: Vec<Node>,
    action: Action,
    n: f32,
    q: f32,
}

impl Node {
    pub fn empty() -> Self {
        Self {
            children: Vec::new(),
            action: Action::Skip,
            n: 0.,
            q: 0.,
        }
    }

    #[inline(always)]
    pub fn get_value(&self) -> f32 {
        if self.n > 0. {
            self.q / self.n
        } else {
            std::f32::NEG_INFINITY
        }
    }

    #[inline(always)]
    pub fn best_child(&self) -> &Node {
        let mut best_child: usize = 0;
        let mut best_value = std::f32::NEG_INFINITY;
        for (i, child_node) in self.children.iter().enumerate() {
            let child_value = child_node.get_value();
            if child_value > best_value {
                best_value = child_value;
                best_child = i;
            }
        }
        &self.children[best_child]
    }

    #[inline(always)]
    pub fn best_action(&self) -> Action {
        if self.children.is_empty() {
            Action::Skip
        } else {
            self.best_child().action
        }
    }

    #[inline(always)]
    fn get_uct_value(&self, parent_n: f32, c: f32) -> f32 {
        if self.n > 0. {
            (self.q / self.n) + c * sqrt(ln(parent_n) / self.n)
        } else {
            std::f32::INFINITY
        }
    }

    fn child_with_max_uct_value(&mut self) -> &mut Node {
        let c_adjusted = C + C_FACTOR * ln((1. + self.n + C_BASE) / C_BASE);
        let mut best_child = 0;
        let mut best_value = std::f32::NEG_INFINITY;
        for (i, child) in self.children.iter().enumerate() {
            let value = child.get_uct_value(self.n, c_adjusted);
            if value > best_value {
                best_value = value;
                best_child = i;
            }
        }
        &mut self.children[best_child]
    }

    #[inline(always)]
    fn backpropagate(&mut self, q: f32) {
        self.n += 1.;
        self.q += q;
    }

    fn expand(&mut self, state: &mut GameState, action_list: &mut ActionList) {
        state.get_possible_actions(action_list);
        self.children = Vec::with_capacity(action_list.size);
        for i in 0..action_list.size {
            self.children.push(Node {
                children: Vec::new(),
                action: action_list[i],
                n: 0.,
                q: 0.,
            });
        }
    }

    pub fn iteration(
        &mut self,
        action_list: &mut ActionList,
        state: &mut GameState,
        rng: &mut SmallRng,
    ) -> f32 {
        let delta;
        if self.children.is_empty() {
            if !state.is_game_over() {
                if self.n as usize % VISITS_BEFORE_EXPANSION == 1 {
                    self.expand(state, action_list);
                }
                delta = rollout(&state, rng);
            } else if self.n == 0. {
                let result = state.game_result() * state.get_team();
                self.q = match result {
                    r if r > 0 => 1.,
                    r if r < 0 => 0.,
                    _ => 0.5,
                };
                self.n = 1.;
                delta = self.q / self.n;
            } else {
                delta = self.q / self.n;
            }
            self.backpropagate(delta);
            return 1. - delta;
        }
        let next_child = self.child_with_max_uct_value();
        state.do_action(next_child.action);
        delta = next_child.iteration(action_list, state, rng);
        self.backpropagate(delta);
        1. - delta
    }

    pub fn principal_variation(&self, state: &mut GameState, action_list: &mut ActionList) {
        if self.children.is_empty() {
            return;
        }
        let child = self.best_child();
        let action = child.action;
        action_list.push(action);
        state.do_action(action);
        child.principal_variation(state, action_list);
    }
}

pub struct MCTS {
    root_node: Node,
    root_state: GameState,
    time_limit: i64,
}

impl MCTS {
    pub fn new(time_limit: u128) -> Self {
        Self {
            root_node: Node::empty(),
            root_state: GameState::default(),
            time_limit: time_limit as i64,
        }
    }

    fn set_root(&mut self, state: &GameState) {
        loop {
            let last_board = self.root_state.board[self.root_state.get_current_color() as usize];
            let changed_fields =
                state.board[self.root_state.get_current_color() as usize] & !last_board;
            let action = Action::from_bitboard(changed_fields);
            let mut found = false;
            for (i, child) in self.root_node.children.iter().enumerate() {
                if child.action == action {
                    self.root_state.do_action(action);
                    self.root_node = self.root_node.children.remove(i);
                    found = true;
                    break;
                }
            }
            if self.root_state.ply == state.ply {
                break;
            }
            if !found {
                self.root_state = state.clone();
                self.root_node = Node::empty();
                break;
            }
        }
        self.root_state = state.clone();
    }

    fn search_nodes(&mut self, n: usize, rng: &mut SmallRng) {
        let mut action_list = ActionList::default();
        for _ in 0..n {
            self.root_node
                .iteration(&mut action_list, &mut self.root_state.clone(), rng);
        }
    }

    fn print_stats(&self, principal_variation: &mut ActionList, time_left: i64) {
        println!(
            "{:6}ms {:6.2} {}",
            time_left,
            1. - self.root_node.get_value(),
            principal_variation
        );
    }

    pub fn search_action(&mut self, state: &GameState) -> (Action, f32) {
        println!("Searching action using MCTS. Fen: {}", state.to_fen());
        let start_time = Instant::now();
        self.set_root(&state);
        let mut rng = SmallRng::from_entropy();
        let mut principal_variation = ActionList::default();
        let mut iterations_per_ms = 0.1;
        let mut searched: usize = 0;
        let mut action_list = ActionList::default();
        self.root_state.get_possible_actions(&mut action_list);
        if action_list[0] == Action::Skip {
            return (Action::Skip, std::f32::NEG_INFINITY);
        }

        println!("    Time  Value PV");
        loop {
            let time_left = self.time_limit - start_time.elapsed().as_millis() as i64;
            principal_variation.clear();
            self.root_node
                .principal_variation(&mut self.root_state.clone(), &mut principal_variation);
            if searched > 0 {
                self.print_stats(&mut principal_variation, time_left);
            }
            if time_left < 80 {
                break;
            }
            let to_search = ((time_left as f64 / 2.) * iterations_per_ms)
                .max(1.)
                .min(1_500_000.) as usize;
            self.search_nodes(to_search, &mut rng);
            searched += to_search;
            iterations_per_ms = searched as f64 / start_time.elapsed().as_millis() as f64;
        }
        self.print_stats(&mut principal_variation, 0);
        println!(
            "Search finished after {}ms. Value: {} PV: {}",
            start_time.elapsed().as_millis(),
            1. - self.root_node.get_value(),
            principal_variation,
        );
        (
            self.root_node.best_action(),
            1. - self.root_node.get_value(),
        )
    }
}

impl Player for MCTS {
    fn on_move_request(&mut self, state: &GameState) -> Action {
        let (action, _) = self.search_action(state);
        action
    }

    fn on_reset(&mut self) {
        self.root_node = Node::empty();
    }
}
