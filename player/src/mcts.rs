use super::float_stuff::{ln, sqrt};
use super::neural_network::{state_to_vector, BoardRotation, NeuralNetwork};
use super::playout::playout;
use game_sdk::{Action, ActionList, Bitboard, GameState, Player};
use rand::{rngs::SmallRng, SeedableRng};
use std::time::Instant;

const C: f32 = 0.0;
const C_BASE: f32 = 9000.0;
const C_FACTOR: f32 = 30.0;
const VISITS_BEFORE_EXPANSION: usize = 50;
const B_SQUARED: f32 = 0.7;
const FPU_R: f32 = 0.1;

pub struct RaveTable {
    actions: Vec<(f32, f32)>,
}

impl RaveTable {
    pub fn get_values(&self, action: Action, color: usize) -> (f32, f32) {
        let index = if action.is_set() {
            let destination = action.get_destination() as usize;
            let shape = action.get_shape() as usize;
            (shape * 418 + destination) * 4 + color
        } else {
            153828 + color
        };
        *self.actions.get(index).unwrap()
    }

    pub fn add_value(&mut self, action: Action, color: usize, value: f32) {
        let index = if action.is_set() {
            let destination = action.get_destination() as usize;
            let shape = action.get_shape() as usize;
            (shape * 418 + destination) * 4 + color
        } else {
            153828 + color
        };
        let entry = self.actions.get_mut(index).unwrap();
        entry.0 += value;
        entry.1 += 1.;
    }

    pub fn get_actions(&self) -> &Vec<(f32, f32)> {
        &self.actions
    }
}

impl Default for RaveTable {
    fn default() -> Self {
        let actions: Vec<(f32, f32)> = vec![(0., 0.); 153832];
        Self { actions }
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
            action: Action::skip(),
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

    fn get_uct_value(
        &self,
        parent_n: f32,
        c: f32,
        color: usize,
        rave_table: &RaveTable,
        fpu_base: f32,
        is_root: bool,
    ) -> f32 {
        if is_root {
            return if self.n > 0. {
                self.q / self.n + c * sqrt(ln(parent_n) / self.n)
            } else {
                std::f32::INFINITY
            };
        }
        let (rave_q, rave_n) = rave_table.get_values(self.action, color);
        let beta = (rave_n / (rave_n + self.n + 4. * B_SQUARED * rave_n * self.n)).min(1.);
        if self.n > 0. {
            (1. - beta) * self.q / self.n
                + beta * rave_q / rave_n
                + self.q / self.n
                + c * sqrt(ln(parent_n) / self.n)
        } else {
            beta * rave_q / rave_n + (1. - beta) * fpu_base + c * sqrt(ln(parent_n))
        }
    }

    fn child_with_max_uct_value(
        &mut self,
        color: usize,
        rave_table: &RaveTable,
        is_root: bool,
    ) -> &mut Node {
        let c_adjusted = C + C_FACTOR * ln((1. + self.n + C_BASE) / C_BASE);
        let fpu_base = (self.n - self.q) / self.n - FPU_R;
        let mut best_child = 0;
        let mut best_value = std::f32::NEG_INFINITY;
        for (i, child) in self.children.iter().enumerate() {
            let value =
                child.get_uct_value(self.n, c_adjusted, color, rave_table, fpu_base, is_root);
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

    fn expand(&mut self, state: &GameState, al: &mut ActionList) {
        state.get_possible_actions(al);
        self.children = Vec::with_capacity(al.size);
        for i in 0..al.size {
            self.children.push(Node {
                children: Vec::new(),
                action: al[i],
                n: 0.,
                q: 0.,
            });
        }
    }

    pub fn iteration(
        &mut self,
        al: &mut ActionList,
        state: &mut GameState,
        rng: &mut SmallRng,
        rave_table: &mut RaveTable,
        is_root: bool,
    ) -> f32 {
        let delta;
        if self.children.is_empty() {
            if !state.is_game_over() {
                if self.n as usize % VISITS_BEFORE_EXPANSION == 1 {
                    self.expand(state, al);
                }
                let result = playout(&mut state.clone(), rng, rave_table);
                delta = if state.ply % 2 == 0 {
                    1. - result
                } else {
                    result
                };
            } else if self.n == 0. {
                let result = state.game_result();
                self.q = match result * state.get_team() {
                    r if r > 0 => 0.999 + (result.abs() as f32) / 100_000.,
                    r if r < 0 => 0.001 - (result.abs() as f32) / 100_000.,
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
        let next_child =
            self.child_with_max_uct_value(state.get_current_color() as usize, &rave_table, is_root);
        state.do_action(next_child.action);
        delta = next_child.iteration(al, state, rng, rave_table, false);
        self.backpropagate(delta);
        1. - delta
    }

    pub fn pv(&mut self, state: &mut GameState, al: &mut ActionList) {
        if self.children.is_empty() {
            return;
        }
        let child = self.best_child();
        let action = child.action;
        al.push(action);
        state.do_action(action);
        child.pv(state, al);
    }

    pub fn best_child(&mut self) -> &mut Node {
        let value = 1. - self.get_value();
        let mut best_child: usize = 0;
        let mut best_value = std::f32::NEG_INFINITY;
        for (i, child) in self.children.iter().enumerate() {
            let mut child_value = child.get_value();
            if value > 0.99 && child.action.is_set() && child.action.get_shape() == 0 {
                child_value -= 0.05;
            }
            if child_value > best_value {
                best_value = child_value;
                best_child = i;
            }
        }
        &mut self.children[best_child]
    }

    pub fn best_action(&mut self) -> Action {
        if self.children.is_empty() {
            Action::skip()
        } else {
            self.best_child().action
        }
    }

    pub fn search_seeding(
        &mut self,
        state: &GameState,
        al: &mut ActionList,
        neural_network: &mut NeuralNetwork,
    ) {
        if self.children.is_empty() {
            self.expand(state, al);
        }
        let mut state = state.clone();
        let r = BoardRotation::rotate_state(&mut state);
        let input = state_to_vector(&state, al);
        let output = neural_network.feed_forward(input);
        for child in self.children.iter_mut() {
            if child.action.is_skip() {
                continue;
            }
            let mut value: f32 = 0.;
            let destination = child.action.get_destination();
            let shape = child.action.get_shape() as usize;
            let mut action_board = Bitboard::with_piece(destination, shape);
            action_board = r.rotate_bitboard(action_board);
            while action_board.not_zero() {
                let field_index = action_board.trailing_zeros();
                action_board.flip_bit(field_index);
                let x = field_index % 21;
                let y = (field_index - x) / 21;
                value += output[(x + y * 20) as usize];
            }
            for _ in 0..3 {
                child.backpropagate(value / 5.);
            }
        }
    }

    pub fn get_children(&self) -> &Vec<Node> {
        &self.children
    }
}

pub struct Mcts {
    root_node: Node,
    root_state: GameState,
    time_limit: Option<i64>,
    iteration_limit: Option<usize>,
    rave_table: RaveTable,
    neural_network: Option<NeuralNetwork>,
}

impl Mcts {
    pub fn set_iteration_limit(&mut self, iteration_limit: Option<usize>) {
        self.iteration_limit = iteration_limit;
    }

    pub fn set_time_limit(&mut self, time_limit: Option<i64>) {
        self.time_limit = time_limit;
    }

    pub fn set_neural_network(&mut self, neural_network: Option<NeuralNetwork>) {
        self.neural_network = neural_network;
    }

    pub fn get_action_value_pairs(&self) -> Vec<(Action, f32)> {
        let mut ret: Vec<(Action, f32)> = Vec::with_capacity(1300);
        for child in self.root_node.get_children().iter() {
            ret.push((child.action, child.get_value()));
        }
        ret
    }

    pub fn get_value(&self) -> f32 {
        1. - self.root_node.get_value()
    }

    pub fn get_root_node(&mut self) -> &mut Node {
        &mut self.root_node
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

    fn do_iterations(&mut self, n: usize, rng: &mut SmallRng) {
        let mut al = ActionList::default();
        for _ in 0..n {
            self.root_node.iteration(
                &mut al,
                &mut self.root_state.clone(),
                rng,
                &mut self.rave_table,
                true,
            );
        }
    }

    pub fn search_action(&mut self, state: &GameState) -> Action {
        println!("Searching action using MCTS. Fen: {}", state.to_fen());
        let start_time = Instant::now();
        self.set_root(&state);
        let mut rng = SmallRng::from_entropy();
        let mut pv = ActionList::default();
        let mut iterations_per_ms = 5.;
        let mut iterations: usize = 0;

        if state.ply >= 4 && state.ply < 12 {
            if let Some(neural_network) = &mut self.neural_network {
                print!("Neural Network prediction... ");
                let nn_start_time = Instant::now();
                self.root_node
                    .search_seeding(&self.root_state, &mut pv, neural_network);
                println!("finished after {}ms", nn_start_time.elapsed().as_millis());
            }
        }

        println!("    Left Depth Iterations Value PV");
        let search_start_time = Instant::now();
        loop {
            pv.clear();
            self.root_node.pv(&mut self.root_state.clone(), &mut pv);

            let (next_iterations, stop) = if let Some(time_limit) = self.time_limit {
                let time_left = time_limit - start_time.elapsed().as_millis() as i64;
                println!(
                    "{:6}ms {:5} {:10} {:4.0}% {}",
                    time_left,
                    pv.size,
                    iterations,
                    (1. - self.root_node.get_value()).min(1.0) * 100.,
                    pv
                );
                let next_iterations =
                    ((time_left as f64 / 6.).min(5000.) * iterations_per_ms).max(1.) as usize;
                (next_iterations, time_left < 30)
            } else if let Some(iteration_limit) = self.iteration_limit {
                if iterations >= iteration_limit {
                    (0, true)
                } else {
                    let iterations_left = iteration_limit - iterations;
                    println!(
                        "{:6}it {:5} {:10} {:4.0}% {}",
                        iterations_left,
                        pv.size,
                        iterations,
                        (1. - self.root_node.get_value()).min(1.0) * 100.,
                        pv
                    );
                    let next_iterations = iterations_left as usize / 2;
                    (next_iterations, next_iterations < 100)
                }
            } else {
                println!("Mcts has neither a time limit nor a node limit");
                (0, true)
            };
            if stop {
                break;
            }
            self.do_iterations(next_iterations, &mut rng);
            iterations += next_iterations;
            let elapsed = search_start_time.elapsed().as_micros() as f64;
            if elapsed > 0. {
                iterations_per_ms = iterations as f64 / elapsed * 1000.;
            }
        }
        println!(
            "Search finished after {}ms. Value: {:.0}% PV-Depth: {} Iterations: {} Iterations/s: {:.2} PV: {}",
            start_time.elapsed().as_millis(),
            (1. - self.root_node.get_value()).min(1.0) * 100.,
            pv.size,
            iterations,
            iterations_per_ms * 1000.,
            pv,
        );
        self.root_node.best_action()
    }
}

impl Player for Mcts {
    fn on_move_request(&mut self, state: &GameState) -> Action {
        self.search_action(state)
    }

    fn on_reset(&mut self) {
        self.root_node = Node::empty();
        self.rave_table = RaveTable::default();
    }
}

impl Default for Mcts {
    fn default() -> Self {
        Self {
            root_node: Node::empty(),
            root_state: GameState::default(),
            time_limit: Some(1980),
            iteration_limit: None,
            rave_table: RaveTable::default(),
            neural_network: NeuralNetwork::new("weights"),
        }
    }
}
