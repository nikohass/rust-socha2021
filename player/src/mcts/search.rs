use super::node::Node;
use super::rave::RaveTable;
use game_sdk::{Action, ActionList, GameState, Player};
use rand::{rngs::SmallRng, SeedableRng};
use std::time::Instant;

pub struct Mcts {
    root_node: Node,
    root_state: GameState,
    time_limit: Option<i64>,
    iteration_limit: Option<usize>,
    pub rave_table: RaveTable,
}

impl Mcts {
    pub fn set_iteration_limit(&mut self, iteration_limit: usize) {
        self.time_limit = None;
        self.iteration_limit = Some(iteration_limit);
    }

    pub fn get_action_value_pairs(&self) -> Vec<(Action, f32)> {
        let mut ret: Vec<(Action, f32)> = Vec::with_capacity(1300);
        for child in self.root_node.children.iter() {
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
            let color = self.root_state.get_current_color();
            let last_board = self.root_state.board[color];
            let changed_fields = state.board[color] & !last_board;
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
        println!("    Left Depth Iterations Value PV");
        let start_time = Instant::now();
        self.set_root(&state);
        let mut rng = SmallRng::from_entropy();
        let mut pv = ActionList::default();
        let mut iterations_per_ms = 5.;
        let mut iterations: usize = 0;

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
                panic!("Mcts has neither a time limit nor a node limit");
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

    fn set_time_limit(&mut self, time_limit: u128) {
        self.iteration_limit = None;
        self.time_limit = Some(time_limit as i64);
    }
}

impl Default for Mcts {
    fn default() -> Self {
        Self {
            root_node: Node::empty(),
            root_state: GameState::default(),
            time_limit: Some(1960),
            iteration_limit: None,
            rave_table: RaveTable::default(),
        }
    }
}
