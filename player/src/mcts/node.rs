use super::float_stuff::{ln, sqrt};
use super::heuristics;
use super::playout::{playout, result_to_value};
use super::rave::RaveTable;
use game_sdk::{Action, ActionList, GameState};
use rand::rngs::SmallRng;

const C: f32 = 0.0;
const C_BASE: f32 = 220.0;
const C_FACTOR: f32 = std::f32::consts::SQRT_2;
const B_SQUARED: f32 = 0.8;
const FPU_R: f32 = 0.1;

pub struct Node {
    pub children: Vec<Node>, // Vector that contains all child nodes
    pub action: Action,      // Action that leads to this node
    pub n: f32,              // Visits
    pub q: f32,              // Sum of all evaluations
}

impl Node {
    pub fn empty() -> Self {
        Self {
            children: Vec::new(),
            action: Action::SKIP,
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
        let (rave_n, rave_q) = rave_table.get_values(self.action, color);
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
        if state.ply < 32 && !al[0].is_skip() {
            // Use heuristics to expand the node
            heuristics::expand_node(self, state, al);
        } else {
            // Expand the node without heuristics
            for i in 0..al.size {
                self.children.push(Node {
                    children: Vec::new(),
                    action: al[i],
                    n: 0.,
                    q: 0.,
                })
            }
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
                #[allow(clippy::float_cmp)]
                if self.n == 1. {
                    self.expand(state, al);
                }
                let result = playout(&mut state.clone(), rng, rave_table);
                delta = if state.ply % 2 == 0 {
                    1. - result
                } else {
                    result
                };
            } else if self.n == 0. {
                let result = state.game_result() * state.get_team();
                self.q = result_to_value(result);
                self.n = 1.;
                delta = self.q;
            } else {
                delta = self.q / self.n;
            }
            self.backpropagate(delta);
            return 1. - delta;
        }
        let next_child =
            self.child_with_max_uct_value(state.get_current_color(), rave_table, is_root);
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
                child_value -= 0.05; // Encourage the player to keep the Monomino if there are other actions with a similar value.
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
            Action::SKIP
        } else {
            self.best_child().action
        }
    }
}
